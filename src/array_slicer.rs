#[derive(Clone, Copy)]
pub struct ImageDescriptor<'a, T> {
    data: &'a Vec<T>,
    width: usize,
    height: usize,
}

#[derive(Clone, Copy)]
pub struct StrideDescriptor {
    /// How far to stride when iterating horizontally
    per_element: usize,

    /// How far to stride when iterating vertically
    per_row: usize,
}

/// Inclusive, so an ROI of x1=0, x2=0, y1=0, y2=0 windows into a single point
#[derive(Clone, Copy)]
pub struct RoiDescriptor {
    x1: usize,
    x2: usize,
    y1: usize,
    y2: usize,
}

#[derive(Clone, Copy)]
pub struct ImageBufferWindow<'a, T> {
    image: ImageDescriptor<'a, T>,
    stride: StrideDescriptor,
    roi: RoiDescriptor,
    default: &'a T,
    dist_from_x1_to_x2: usize,
    counter: usize,
    total_els: usize,
}

pub struct ImageBufferWindowBuilder<'a, T> {
    image: ImageDescriptor<'a, T>,
    stride: Option<StrideDescriptor>,
    roi: Option<RoiDescriptor>,
    default: Option<&'a T>,
}

impl<'a, T> ImageBufferWindowBuilder<'a, T> {
    pub fn with_stride(mut self, per_element: usize, per_row: usize) -> Self {
        self.stride = Some(StrideDescriptor {
            per_element,
            per_row,
        });
        self
    }

    pub fn with_roi(mut self, x1: usize, x2: usize, y1: usize, y2: usize) -> Self {
        self.roi = Some(RoiDescriptor {
            x1,
            x2,
            y1,
            y2,
        });
        self
    }

    pub fn with_max_roi(mut self) -> Self {
        self.roi = Some(RoiDescriptor {
            x1: 0,
            x2: self.image.width - 1,
            y1: 0,
            y2: self.image.height - 1,
        });
        self
    }

    pub fn with_default(mut self, default: &'a T) -> Self {
        self.default = Some(default);
        self
    }

    pub fn build(self) -> ImageBufferWindow<'a, T> {
        let roi = self.roi.unwrap();
        let dist_from_x1_to_x2 = roi.x2 - roi.x1;
        let total_els = (roi.y2 - roi.y1 + 1) * (roi.x2 - roi.x1 + 1);
        ImageBufferWindow {
            image: self.image,
            stride: self.stride.unwrap(),
            roi,
            default: self.default.unwrap(),
            dist_from_x1_to_x2,
            counter: 0,
            total_els
        }
    }
}

impl<'a, T> ImageBufferWindow<'a, T> {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(data: &'a Vec<T>, width: usize, height: usize) -> ImageBufferWindowBuilder<'a, T> {
        ImageBufferWindowBuilder {
            image: ImageDescriptor {
                data,
                width,
                height,
            },
            stride: None,
            roi: None,
            default: None,
        }
    }
}

pub struct ImageBufferWindowIterator<'a, T> {
    window: ImageBufferWindow<'a, T>,
}

impl<'a, T> Iterator for ImageBufferWindowIterator<'a, T>
    where T: Copy
{
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.window.counter >= self.window.total_els {
            return None;
        }

        let counter = self.window.counter;
        self.window.counter += 1;

        let roi_x = counter % (self.window.dist_from_x1_to_x2 + 1) * self.window.stride.per_element;
        let roi_y = counter / (self.window.dist_from_x1_to_x2 + 1) * self.window.stride.per_row;
        let x = self.window.roi.x1 + roi_x;
        let y = self.window.roi.y1 + roi_y;

        if x >= self.window.image.width || y >= self.window.image.height {
            return Some(self.window.default);
        }

        let idx = y * self.window.image.width + x;
        Some(&self.window.image.data[idx])
    }
}

impl<'a, T> IntoIterator for ImageBufferWindow<'a, T>
    where T: Copy
{
    type Item = &'a T;
    type IntoIter = ImageBufferWindowIterator<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        ImageBufferWindowIterator {
            window: self,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    extern crate test;
    use test::Bencher;

/*
IMAGE: 
    |  0   1   2   3   4   5   6   7   8   9
    +---------------------------------------
  0 | 00, 01, 02, 03, 04, 05, 06, 07, 08, 09
  1 | 10, 11, 12, 13, 14, 15, 16, 17, 18, 19
  2 | 20, 21, 22, 23, 24, 25, 26, 27, 28, 29
  3 | 30, 31, 32, 33, 34, 35, 36, 37, 38, 39
  4 | 40, 41, 42, 43, 44, 45, 46, 47, 48, 49
  5 | 50, 51, 52, 53, 54, 55, 56, 57, 58, 59
  6 | 60, 61, 62, 63, 64, 65, 66, 67, 68, 69
  7 | 70, 71, 72, 73, 74, 75, 76, 77, 78, 79
  8 | 80, 81, 82, 83, 84, 85, 86, 87, 88, 89
  9 | 90, 91, 92, 93, 94, 95, 96, 97, 98, 99

ROI: 
33, 34, 35, 36
43, 44, 45, 46
53, 54, 55, 56
63, 64, 65, 66
*/

    #[test]
    fn iterate_over_window() {
        let data: Vec<u8> = (0..100).collect();
        let window = ImageBufferWindow::new(&data, 10, 10)
            .with_stride(1, 1)
            .with_roi(3, 6, 3, 6)
            .with_default(&0)
            .build();

        let expected_vals = vec![33, 34, 35, 36, 43, 44, 45, 46, 53, 54, 55, 56, 63, 64, 65, 66];
        for (i, v) in window.into_iter().enumerate() {
            println!("Value: {v}");
            assert_eq!(*v, expected_vals[i]);
        }
    }

    #[test]
    fn out_of_bounds_returns_default() {
        let data: Vec<u8> = (0..100).collect();
        let window = ImageBufferWindow::new(&data, 10, 10)
            .with_stride(1, 1)
            .with_roi(10, 10, 0, 0)
            .with_default(&255)
            .build();

        let expected_vals = vec![255];
        for (i, v) in window.into_iter().enumerate() {
            println!("Value: {v}");
            assert_eq!(*v, expected_vals[i]);
        }
    }

    #[bench]
    fn bench_iterate_over_window(b: &mut Bencher) {
        let data: Vec<u8> = vec![0; 1000000];
        let window = ImageBufferWindow::new(&data, 1000, 1000)
            .with_stride(1, 1)
            .with_max_roi()
            .with_default(&0)
            .build();

        let bench_loop = move || {
            for v in window.into_iter() {
                test::black_box(v);
            }
        };
        b.iter(bench_loop);
    }

}