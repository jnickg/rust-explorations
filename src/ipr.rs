use std::io::Cursor;

use image::{DynamicImage, GenericImageView, ImageFormat};

use crate::dims::{Cols, Dims, HasDims, Rows};
use crate::dyn_matrix::DynMatrix;

pub struct IprImage<'a>(pub &'a DynamicImage);

pub struct ImageTiles {
    pub original_width: u32,
    pub original_height: u32,
    pub tiles: Vec<DynamicImage>,
    pub tile_width: u32,
    pub tile_height: u32,
    pub count_across: u32,
    pub count_down: u32,
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
        let mut pyramid = Vec::new();
        pyramid.push(self.0.clone());

        let mut i = self.0.clone();
        while i.width() > 1 && i.height() > 1 {
            i = i.resize(
                i.width() / 2,
                i.height() / 2,
                image::imageops::FilterType::Gaussian,
            );
            pyramid.push(i.clone());
        }

        Ok(pyramid)
    }

    fn make_tiles(&self, tile_width: u32, tile_height: u32) -> Result<ImageTiles, &'static str> {
        let i = &self.0;
        let (width, height) = i.dimensions();

        let count_across = width / tile_width;
        let count_down = height / tile_height;

        let mut tile_buffers = Vec::new();
        for y in 0..count_down {
            for x in 0..count_across {
                let actual_width = if x == count_across - 1 {
                    width % tile_width
                } else {
                    tile_width
                };
                let actual_height = if y == count_down - 1 {
                    height % tile_height
                } else {
                    tile_height
                };
                let tile = i
                    .view(x * tile_width, y * tile_height, actual_width, actual_height)
                    .to_image();
                tile_buffers.push(tile);
            }
        }

        let tiles = tile_buffers
            .iter()
            .map(|t| DynamicImage::ImageRgba8(t.clone()))
            .collect();

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
