use num::Unsigned;
use std::ops::{Index, IndexMut};

pub trait PixelComponent: Unsigned + Clone + Default {}
impl<T> PixelComponent for T where T: Unsigned + Clone + Default {}

pub struct XIndex(pub u32);
pub struct YIndex(pub u32);
pub struct CIndex(pub u32);

pub struct ImageIndex(XIndex, YIndex, CIndex);

impl From<ImageIndex> for (u32, u32, u32) {
    fn from(value: ImageIndex) -> Self {
        let ImageIndex(XIndex(x), YIndex(y), CIndex(c)) = value;
        (x, y, c)
    }
}

/// An Image type that stores pixel data in a contiguous array of unsigned integral elements.
#[derive(Debug, Clone)]
pub struct MyImage<T: PixelComponent> {
    /// The image data, stored as a contiguous array of values.
    data: Vec<T>,

    /// Image width, in pixels
    width: u32,

    /// Image height, in pixels
    height: u32,

    /// Number of components per pixel. Each component is one element
    components_per_pixel: u32,
}

impl<T: PixelComponent> MyImage<T> {
    /// Create a new image with the given dimensions and number of components per pixel.
    pub fn new(width: u32, height: u32, components_per_pixel: u32) -> Self {
        let data = vec![T::zero(); (width * height * components_per_pixel) as usize];
        Self {
            data,
            width,
            height,
            components_per_pixel,
        }
    }

    /// Get the width of the image, in pixels.
    pub fn width(&self) -> u32 {
        self.width
    }

    /// Get the height of the image, in pixels.
    pub fn height(&self) -> u32 {
        self.height
    }

    /// Get the number of components per pixel.
    pub fn components_per_pixel(&self) -> u32 {
        self.components_per_pixel
    }

    /// Get a reference to the image data.
    pub fn data(&self) -> &[T] {
        &self.data
    }

    /// Get a mutable reference to the image data.
    pub fn data_mut(&mut self) -> &mut [T] {
        &mut self.data
    }
}

impl<T: PixelComponent> Index<u32> for MyImage<T> {
    type Output = [T];

    fn index(&self, index: u32) -> &Self::Output {
        let index = index * self.components_per_pixel;
        &self.data[index as usize..(index + self.components_per_pixel) as usize]
    }
}

impl<T: PixelComponent> Index<ImageIndex> for MyImage<T> {
    type Output = T;

    fn index(&self, ImageIndex(XIndex(x), YIndex(y), CIndex(c)): ImageIndex) -> &Self::Output {
        assert!(c < self.components_per_pixel);
        let index = (y * self.width + x) * self.components_per_pixel + c;
        &self.data[index as usize]
    }
}

impl<T: PixelComponent> Index<(u32, u32)> for MyImage<T> {
    type Output = [T];

    fn index(&self, (x, y): (u32, u32)) -> &Self::Output {
        let index = (y * self.width + x) * self.components_per_pixel;
        &self.data[index as usize..(index + self.components_per_pixel) as usize]
    }
}


impl<T: PixelComponent> Index<(u32, u32, u32)> for MyImage<T> {
    type Output = T;

    fn index(&self, (x, y, c): (u32, u32, u32)) -> &Self::Output {
        assert!(c < self.components_per_pixel);
        let index = (y * self.width + x) * self.components_per_pixel + c;
        &self.data[index as usize]
    }
}

impl<T: PixelComponent> IndexMut<u32> for MyImage<T> {
    fn index_mut(&mut self, index: u32) -> &mut Self::Output {
        let index = index * self.components_per_pixel;
        &mut self.data[index as usize..(index + self.components_per_pixel) as usize]
    }
}

impl<T: PixelComponent> IndexMut<(u32, u32)> for MyImage<T> {
    fn index_mut(&mut self, (x, y): (u32, u32)) -> &mut Self::Output {
        let index = (y * self.width + x) * self.components_per_pixel;
        &mut self.data[index as usize..(index + self.components_per_pixel) as usize]
    }
}

impl<T: PixelComponent> IndexMut<(u32, u32, u32)> for MyImage<T> {
    fn index_mut(&mut self, (x, y, c): (u32, u32, u32)) -> &mut Self::Output {
        assert!(c < self.components_per_pixel);
        let index = (y * self.width + x) * self.components_per_pixel + c;
        &mut self.data[index as usize]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_image_indexing() {
        let mut image = MyImage::<u8>::new(2, 2, 3);
        image[(0, 0, 0)] = 255;
        image[(0, 0, 1)] = 128;
        image[(0, 0, 2)] = 0;
        image[(1, 1, 0)] = 0;
        image[(1, 1, 1)] = 128;
        image[(1, 1, 2)] = 255;

        assert_eq!(image[(0, 0, 0)], 255);
        assert_eq!(image[(0, 0, 1)], 128);
        assert_eq!(image[(0, 0, 2)], 0);
        assert_eq!(image[(1, 1, 0)], 0);
        assert_eq!(image[(1, 1, 1)], 128);
        assert_eq!(image[(1, 1, 2)], 255);
    }
}
