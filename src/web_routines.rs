
use futures_util::AsyncReadExt;
use image::{DynamicImage, ImageFormat};
use mongodb::{bson::{doc, Bson, Document}, Collection};
use uuid::Uuid;
use rayon::prelude::*;

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
pub async fn generate_tiles_for_pyramid(app_state: AppState, pyramid_uuid: Uuid) -> Result<(), &'static str> {
    let mut dest_format = ImageFormat::Png;
    let pyramid_images: Vec<DynamicImage> = {
        let app =  &mut app_state.read().await;
        let db = app.db.as_ref().ok_or("Database not connected")?;
        let pyramids_collection: Collection<Document> = db.collection("pyramids");
        // Update document so "tiles" field says "processing" and update the db
        match pyramids_collection.update_one(
            doc! { "uuid": pyramid_uuid.to_string() },
            doc! { "$set": { "tiles": "processing" } },
            None,
        ).await {
            Ok(_) => (),
            Err(_) => return Err("Error updating pyramid"),
        };
        // Now get a handle to the document and return it from the scope block
        let pyramid_doc = match pyramids_collection.find_one(doc! { "uuid": pyramid_uuid.to_string() }, None).await {
            Ok(Some(doc)) => doc,
            Ok(None) => return Err("Pyramid not found"),
            Err(_) => return Err("Error fetching pyramid"),
        };

        let mime_type = match pyramid_doc.get("mime_type") {
            Some(m) => m.as_str().unwrap(),
            None => return Err("Failed to determine mime type")
        };
        dest_format = ImageFormat::from_mime_type(mime_type).unwrap();

        // Grab each of the image files from GridFS
        let image_ids: &Vec<Bson> = match pyramid_doc.get_array("image_files") {
            Ok(arr) => arr,
            _ => return Err("Error fetching image files"),
        };

        let bucket = db.gridfs_bucket(None);

        futures::future::join_all(
            image_ids.iter().map(|id| async {
                let mut image_bytes = Vec::new();
                let mut image_stream = bucket.open_download_stream(id.clone()).await.unwrap();
                match image_stream.read_to_end(&mut image_bytes).await {
                    Ok(_) => (),
                    Err(_) => {
                        todo!();
                    }
                };
                image::load_from_memory_with_format(&image_bytes, dest_format).unwrap()
        })).await
    };

    // Now that we've grabbed all the images in the pyramid and updated the doc, actually create
    // the tiles for each pyramid level, then encode them to the destination format and brotli
    // compress them. Use Rayon to process each pyramid level separately when breaking into tiles,
    // then use Rayon to process each tile separately to encode/compress. Then we collect them into
    // a 3D array where the first dimension is the pyramid level, the second dimension is the list
    // of tiles, and the third is the bytes for that given tile
    let compressed_level_tiles: Vec<Vec<Vec<u8>>> = pyramid_images.par_iter().map(|i: &DynamicImage| -> Vec<Vec<u8>> {
        let image = IprImage(i);
        let tiles = image.make_tiles(512, 512).unwrap();
        let compressed_tiles: Vec<Vec<u8>> = tiles.tiles.par_iter().map(|t: &DynamicImage| -> Vec<u8> {
            let tile = IprImage(t);
            tile.compress_brotli(10, 24, Some(dest_format)).unwrap()
        }).collect();
        compressed_tiles
    }).collect();

    // Kick off more Rayon work so that for each Pyramid level & tile, we write that object to
    // GridFS and update the pyramid doc to have a handle to the GridFS object ID
    todo!();
}