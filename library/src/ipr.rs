use image::{DynamicImage, GenericImageView, ImageFormat};
use std::io::Cursor;

use crate::dims::{Cols, Dims, HasDims, Rows};
use crate::dyn_matrix::DynMatrix;

pub struct IprImage<'a>(pub &'a DynamicImage);

#[derive(Debug, Default)]
pub struct ImageTiles {
    pub original_width: u32,
    pub original_height: u32,
    pub tiles: Vec<DynamicImage>,
    pub tile_width: u32,
    pub tile_height: u32,
    pub count_across: u32,
    pub count_down: u32,
}

impl Clone for ImageTiles {
    fn clone(&self) -> Self {
        ImageTiles {
            original_width: self.original_width,
            original_height: self.original_height,
            tiles: self.tiles.clone(),
            tile_width: self.tile_width,
            tile_height: self.tile_height,
            count_across: self.count_across,
            count_down: self.count_down,
        }
    }
}

pub trait HasImageProcessingRoutines {
    fn convolve_in_place(&mut self, k: DynMatrix<f64>) -> Result<(), &'static str>;
    fn generate_image_pyramid(&self) -> Result<Vec<DynamicImage>, &'static str>;
    fn make_tiles(&self, tile_width: u32, tile_height: u32) -> Result<ImageTiles, &'static str>;
    fn compress_brotli(
        &self,
        brotli_level: u32,
        brotli_lg_window_size: u32,
        fmt: Option<ImageFormat>,
    ) -> Result<Vec<u8>, &'static str>;
}

impl<'a> HasImageProcessingRoutines for IprImage<'a> {
    fn convolve_in_place(&mut self, k: DynMatrix<f64>) -> Result<(), &'static str> {
        let Dims(Rows(r), Cols(c)) = k.dims();
        if r != c {
            return Err("Kernel matrix must be square in shape!");
        }
        if r % 2 == 0 {
            return Err("Kernel matrix must have an odd number of rows and columns!");
        }

        let i = &self.0;
        let (_width, _height) = i.dimensions();

        todo!("Iterate through image pixels and convolve neighborhood. Lose outer data");
    }

    fn generate_image_pyramid(&self) -> Result<Vec<DynamicImage>, &'static str> {
        use image_pyramid::*;

        let params = ImagePyramidParams {
            pyramid_type: ImagePyramidType::Lowpass,
            scale_factor: 0.5,
            smoothing_type: SmoothingType::Gaussian,
        };

        let pyramid = ImagePyramid::create(self.0, Some(&params))?;

        Ok(pyramid.levels)
    }

    /// Splits this image into tiles of the given dimensions or smaller.
    ///
    /// The number of tiles returned is the product of the following:
    ///     - The ceiling of `self.width() / tile_width`
    ///     - The ceiling of `self.height() / tile_height`
    ///
    /// That is, if the instance's dimensions are (1000,1000) and the tile dimensions are (300,300)
    /// then there will be sixteen tiles, as illustrated below. (9 full-size tiles, 3 tiles along
    /// the bottom with a height of 100, 3 tiles along the right side with a width of 100, and one
    /// tile in the bottom-right corner with dimensions (100,100)).
    ///
    /// ```text
    ///           300px       300px       300px   100px
    ///       ┌───────────┬───────────┬──────────┬─────┐
    ///       │           │           │          │     │
    ///  300px│           │           │          │     │
    ///       │           │           │          │     │
    ///       │           │           │          │     │
    ///       ├───────────┼───────────┼──────────┼─────┤
    ///       │           │           │          │     │
    ///  300px│           │           │          │     │
    ///       │           │           │          │     │
    ///       │           │           │          │     │
    ///       ├───────────┼───────────┼──────────┼─────┤
    ///       │           │           │          │     │
    ///  300px│           │           │          │     │
    ///       │           │           │          │     │
    ///       │           │           │          │     │
    ///       ├───────────┼───────────┼──────────┼─────┤
    ///  100px│           │           │          │     │
    ///       │           │           │          │     │
    ///       └───────────┴───────────┴──────────┴─────┘
    /// ``````
    fn make_tiles(&self, tile_width: u32, tile_height: u32) -> Result<ImageTiles, &'static str> {
        let i = &self.0;
        let (width, height) = i.dimensions();

        let count_across = (width + tile_width - 1) / tile_width;
        let count_down = (height + tile_height - 1) / tile_height;

        let mut tiles = Vec::<DynamicImage>::new();
        // let mut tiles = Vec::new();
        for y in 0..count_down {
            for x in 0..count_across {
                let actual_tile_width = if x == count_across - 1 {
                    width - x * tile_width
                } else {
                    tile_width
                };
                let actual_tile_height = if y == count_down - 1 {
                    height - y * tile_height
                } else {
                    tile_height
                };

                let tile = i.crop_imm(
                    x * tile_width,
                    y * tile_height,
                    actual_tile_width,
                    actual_tile_height,
                );
                tiles.push(tile);
            }
        }

        Ok(ImageTiles {
            original_height: i.height(),
            original_width: i.width(),
            tiles,
            tile_width,
            tile_height,
            count_across,
            count_down,
        })
    }

    fn compress_brotli(
        &self,
        brotli_level: u32,
        brotli_lg_window_size: u32,
        fmt: Option<ImageFormat>,
    ) -> Result<Vec<u8>, &'static str> {
        let i = &self.0;
        let dest_fmt = fmt.unwrap_or(ImageFormat::Png);

        let mut data = Vec::new();
        let mut cursor = Cursor::new(&mut data);
        i.write_to(&mut cursor, dest_fmt).unwrap();

        let brotli_params = brotli::enc::BrotliEncoderParams {
            quality: match brotli_level.try_into() {
                Ok(v) => v,
                Err(_) => return Err("Brotli level must be between 0 and 11"),
            },
            lgwin: match brotli_lg_window_size.try_into() {
                Ok(v) => v,
                Err(_) => return Err("Brotli lg_window_size must be between 10 and 24"),
            },
            // This is a neat feature :-)
            ..Default::default()
        };

        let mut compressed_data = Vec::new();
        brotli::BrotliCompress(
            &mut Cursor::new(&mut data),
            &mut compressed_data,
            &brotli_params,
        )
        .unwrap();

        Ok(compressed_data)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use super::*;
    use rayon::prelude::*;
    extern crate test;
    use test::Bencher;
    use test_case::test_case;

    #[test_case("test_files/elden_ring.jpg")]
    #[test_case("test_files/totk.bmp")]
    #[test_case("test_files/totk.jpg")]
    #[test_case("test_files/totk.png")]
    #[test_case("test_files/totk.webp")]
    fn compress_brotli(i_path: &str) {
        let i = image::open(i_path).unwrap();
        let i = IprImage(&i);

        let original_data_format = ImageFormat::from_path(i_path).unwrap();
        let compressed_buf = match i.compress_brotli(10, 24, Some(original_data_format)) {
            Ok(v) => v,
            Err(_) => panic!("You fool! You cannot compress such a thing."),
        };

        let mut original_buf = Vec::<u8>::new();
        let mut cursor = Cursor::new(&mut original_buf);
        match i.0.write_to(&mut cursor, original_data_format) {
            Ok(_) => (),
            Err(_) => panic!("You fool! You cannot write such a thing."),
        };

        assert!(compressed_buf.len() < original_buf.len());
    }

    #[test_case("test_files/elden_ring.jpg")]
    #[test_case("test_files/totk.bmp")]
    #[test_case("test_files/totk.jpg")]
    #[test_case("test_files/totk.png")]
    #[test_case("test_files/totk.webp")]
    fn generate_image_pyramid(i_path: &str) {
        let i = image::open(i_path).unwrap();
        let i = IprImage(&i);

        let pyramid = match i.generate_image_pyramid() {
            Ok(v) => v,
            Err(_) => panic!("You fool! You cannot construct the pyramids with such a thing."),
        };

        // Compute expected pyramid levels using the image dimensions. We compute the full image
        // pyramid down to a single-pixel
        let expected_pyramid_levels = (0..)
            .map(|n| 2u32.pow(n))
            .take_while(|&n| n <= i.0.width() && n <= i.0.height())
            .collect::<Vec<u32>>();

        assert_eq!(pyramid.len(), expected_pyramid_levels.len());
    }

    #[bench]
    fn bench_generate_image_pyramid(b: &mut Bencher) {
        let i_path = "test_files/totk.png";
        let i = image::open(i_path).unwrap();
        let i = IprImage(&i);

        let bench_loop = move || {
            let pyramid = match i.generate_image_pyramid() {
                Ok(v) => v,
                Err(_) => panic!("You fool! You cannot construct the pyramids with such a thing."),
            };
            test::black_box(pyramid);
        };
        b.iter(bench_loop)
    }

    #[test_case("test_files/elden_ring.jpg")]
    #[test_case("test_files/totk.bmp")]
    #[test_case("test_files/totk.jpg")]
    #[test_case("test_files/totk.png")]
    #[test_case("test_files/totk.webp")]
    fn make_tiles(i_path: &str) {
        let i = image::open(i_path).unwrap();
        let i = IprImage(&i);

        let tiles = match i.make_tiles(256, 256) {
            Ok(v) => v,
            Err(_) => panic!("You fool! You cannot construct the tiles with such a thing."),
        };

        // See documentation for make_tiles for how this is computed
        let expected_num_tiles: usize = (((tiles.original_width + tiles.tile_width - 1)
            / tiles.tile_width)
            * ((tiles.original_height + tiles.tile_height - 1) / tiles.tile_height))
            .try_into()
            .unwrap();

        assert_eq!(tiles.tiles.len(), expected_num_tiles);
    }

    #[test_case("test_files/elden_ring.jpg")]
    #[test_case("test_files/totk.bmp")]
    #[test_case("test_files/totk.jpg")]
    #[test_case("test_files/totk.png")]
    #[test_case("test_files/totk.webp")]
    fn generate_image_pyramid_then_make_tiles_then_compress_brotli(i_path: &str) {
        let i = image::open(i_path).unwrap();
        let i = IprImage(&i);

        let pyramid = match i.generate_image_pyramid() {
            Ok(v) => v,
            Err(_) => panic!("You fool! You cannot construct the pyramids with such a thing."),
        };

        let dest_format = ImageFormat::Png;

        let pyramid_level_tiles = vec![ImageTiles::default(); pyramid.len()];
        let locking_pyramid_level_tiles = Arc::new(Mutex::new(pyramid_level_tiles));
        let compressed_level_tiles: Vec<Vec<Vec<u8>>> = pyramid
            .par_iter()
            .enumerate()
            .map(|(idx, i): (usize, &DynamicImage)| -> Vec<Vec<u8>> {
                let image = IprImage(i);
                let tiles = image.make_tiles(512, 512).unwrap();
                assert_ne!(tiles.tiles.len(), 0);
                let compressed_tiles: Vec<Vec<u8>> = tiles
                    .tiles
                    .par_iter()
                    .map(|t: &DynamicImage| -> Vec<u8> {
                        let tile = IprImage(t);
                        tile.compress_brotli(10, 24, Some(dest_format)).unwrap()
                    })
                    .collect();
                let plt = &mut locking_pyramid_level_tiles.lock().unwrap();
                plt[idx] = tiles;
                compressed_tiles
            })
            .collect();

        assert_eq!(compressed_level_tiles.len(), pyramid.len());
    }
}
