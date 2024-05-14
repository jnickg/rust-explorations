use std::sync::Mutex;

use futures::AsyncWriteExt;
use futures_util::AsyncReadExt;
use image::{DynamicImage, ImageFormat};
use mongodb::{
    bson::{doc, Bson, Document}, options::GridFsBucketOptions, Collection
};
use rayon::prelude::*;
use uuid::Uuid;

use crate::*;

use jnickg_imaging::ipr::{HasImageProcessingRoutines, IprImage};

/// Generate tiles for a pyramid
///
/// With the given image pyramid document, this function represents a background task that takes
/// the pyramid, and generates tiles for each level of the pyramid. The tiles are 512x512 pixels.
///  0. Updates the pyramid doc such that "tiles" field is now "processing" and releases doc lock
///  1. Breaks each image into tiles of 512x512 pixels
///  2. Encodes the tile as a PNG and Brotli compresses the PNG data
///  3. Updates the pyramid doc such that "tiles" field is now "done", when ALL tiles are done
///  4. Updates the pyramid doc such that "tiles" field is now "failed" if any tile fails
pub async fn generate_tiles_for_pyramid(
    app_state: AppState,
    pyramid_uuid: Uuid,
) -> Result<(), &'static str> {
    let (dest_format, pyramid_images): (ImageFormat, Vec<DynamicImage>) = {
        let app = &mut app_state.read().await;
        let db = app.db.as_ref().ok_or("Database not connected")?;
        let pyramids_collection: Collection<Document> = db.collection("pyramids");
        // Update document so "tiles" field says "processing" and update the db
        match pyramids_collection
            .update_one(
                doc! { "uuid": pyramid_uuid.to_string() },
                doc! { "$set": { "tiles": "processing" } },
                None,
            )
            .await
        {
            Ok(_) => (),
            Err(_) => return Err("Error updating pyramid"),
        };
        // Now get a handle to the document and return it from the scope block
        let pyramid_doc = match pyramids_collection
            .find_one(doc! { "uuid": pyramid_uuid.to_string() }, None)
            .await
        {
            Ok(Some(doc)) => doc,
            Ok(None) => return Err("Pyramid not found"),
            Err(_) => return Err("Error fetching pyramid"),
        };

        let mime_type = match pyramid_doc.get("mime_type") {
            Some(m) => m.as_str().unwrap(),
            None => return Err("Failed to determine mime type"),
        };
        let dest_format = ImageFormat::from_mime_type(mime_type).unwrap();

        // Grab each of the image files from GridFS
        let image_ids: &Vec<Bson> = match pyramid_doc.get_array("image_files") {
            Ok(arr) => arr,
            _ => return Err("Error fetching image files"),
        };

        let bucket = db.gridfs_bucket(None);

        let images = futures::future::join_all(image_ids.iter().map(|id| async {
            let mut image_bytes = Vec::new();
            let mut image_stream = bucket.open_download_stream(id.clone()).await.unwrap();
            match image_stream.read_to_end(&mut image_bytes).await {
                Ok(_) => (),
                Err(_) => {
                    todo!();
                }
            };
            image::load_from_memory_with_format(&image_bytes, dest_format).unwrap()
        }))
        .await;

        (dest_format, images)
    };

    // Now that we've grabbed all the images in the pyramid and updated the doc, actually create
    // the tiles for each pyramid level, then encode them to the destination format and brotli
    // compress them. Use Rayon to process each pyramid level separately when breaking into tiles,
    // then use Rayon to process each tile separately to encode/compress. Then we collect them into
    // a 3D array where the first dimension is the pyramid level, the second dimension is the list
    // of tiles, and the third is the bytes for that given tile
    let pyramid_level_tiles = Vec::with_capacity(pyramid_images.len());
    let locking_pyramid_level_tiles = Mutex::new(pyramid_level_tiles);
    let compressed_level_tiles: Vec<Vec<Vec<u8>>> = pyramid_images
        .par_iter()
        .enumerate()
        .map(|(idx, i) : (usize, &DynamicImage)| -> Vec<Vec<u8>> {
            let image = IprImage(i);
            let tiles = image.make_tiles(512, 512).unwrap();
            let compressed_tiles: Vec<Vec<u8>> = tiles
                .tiles
                .par_iter()
                .map(|t: &DynamicImage| -> Vec<u8> {
                    let tile = IprImage(t);
                    tile.compress_brotli(10, 24, Some(dest_format)).unwrap()
                })
                .collect();
            let mut plt = locking_pyramid_level_tiles.lock().unwrap();
            plt.insert(idx, tiles);
            compressed_tiles
        })
        .collect();

    // We don't need the mutex any more, to slurp the vec back out
    let pyramid_level_tiles = locking_pyramid_level_tiles.lock().unwrap();

    // For each Pyramid level & tile, we write that object to GridFS and return a doc describing
    // the tile (x/y loc, w/h, index. In the outer layer, aggregate all Bson::Documents into a
    // single array doc containing all the tile docs for that pyramid level, as well as some
    // metadata about that pyramid level (index, w/h)
    let app = &mut app_state.write().await;
    let db = app.db.as_ref().unwrap();
    let bucket = db.gridfs_bucket(GridFsBucketOptions::builder().bucket_name("tiles".to_string()).build());
    let mut level_docs = Vec::new();
    for (pyramid_level, level_tiles) in compressed_level_tiles.iter().enumerate() {
        let mut tile_docs = Vec::new();
        for (t_idx, tile) in level_tiles.iter().enumerate() {
            let mut upload_stream = bucket.open_upload_stream("tile.data", None);
            match upload_stream.write_all(&tile).await {
                Ok(_) => (),
                Err(_) => return Err("Error writing tile to GridFS"),
            }
            let tile_obj_id = upload_stream.id();
            let level_tiles = &pyramid_level_tiles[pyramid_level];
            let tile_image = &level_tiles.tiles[t_idx];

            // Based on tile size, original dimensions, and tile index, determine our x/y;
            let t_idx: u32 = t_idx.try_into().unwrap();
            let x = (t_idx % level_tiles.count_across) * level_tiles.tile_width;
            let y = (t_idx / level_tiles.count_across) * level_tiles.tile_height;

            tile_docs.push(doc!{
                "x": x,
                "y": y,
                "width": tile_image.width(),
                "height": tile_image.height(),
                "index": t_idx,
                "tile_id": tile_obj_id.to_string()
            });
        }
        // Now that we have all the tile docs for this pyramid level, we need to add some
        // metadata about the pyramid level itself
        let pyramid_level_u32: u32 = pyramid_level.try_into().unwrap(); // How annoying
        level_docs.push(doc!{
            "index": pyramid_level_u32,
            "width": pyramid_images[pyramid_level].width(),
            "height": pyramid_images[pyramid_level].height(),
            "tiles": tile_docs
        });
    }

    let pyramids_collection: Collection<Document> = db.collection("pyramids");
    // Update document so "tiles" field contains all the tiles
    match pyramids_collection
        .update_one(
            doc! { "uuid": pyramid_uuid.to_string() },
            doc! { "$set": { "tiles": level_docs } },
            None,
        )
        .await
    {
        Ok(_) => Ok(()),
        Err(_) => return Err("Error updating pyramid"),
    }
}
