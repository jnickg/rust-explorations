use image::{DynamicImage, GenericImageView};

use crate::dims::{Cols, Dims, HasDims, Rows};
use crate::dyn_matrix::DynMatrix;

pub struct IprImage(pub DynamicImage);

pub trait HasImageProcessingRoutines {
    fn convolve_in_place(&mut self, k: DynMatrix<f64>) -> Result<(), &'static str>;
    fn generate_image_pyramid(&self) -> Result<Vec<DynamicImage>, &'static str>;
}

impl HasImageProcessingRoutines for IprImage {
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
            i = i.resize(i.width() / 2, i.height() / 2, image::imageops::FilterType::Gaussian);
            pyramid.push(i.clone());
        }

        Ok(pyramid)
    }
}
