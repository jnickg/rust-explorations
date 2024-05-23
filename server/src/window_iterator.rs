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
    x1: isize,
    x2: isize,
    y1: isize,
    y2: isize,
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

#[allow(dead_code)]
pub struct ImageBufferWindowBuilder<'a, T> {
    image: ImageDescriptor<'a, T>,
    stride: Option<StrideDescriptor>,
    roi: Option<RoiDescriptor>,
    default: Option<&'a T>,
}

impl<'a, T> ImageBufferWindowBuilder<'a, T> {
    #[allow(dead_code)]
    pub fn with_stride(mut self, per_element: usize, per_row: usize) -> Self {
        self.stride = Some(StrideDescriptor {
            per_element,
            per_row,
        });
        self
    }

    #[allow(dead_code)]
    pub fn with_roi(mut self, x1: isize, x2: isize, y1: isize, y2: isize) -> Self {
        self.roi = Some(RoiDescriptor { x1, x2, y1, y2 });
        self
    }

    #[allow(dead_code)]
    pub fn with_max_roi(mut self) -> Self {
        self.roi = Some(RoiDescriptor {
            x1: 0,
            x2: (self.image.width - 1).try_into().unwrap(),
            y1: 0,
            y2: (self.image.height - 1).try_into().unwrap(),
        });
        self
    }

    #[allow(dead_code)]
    pub fn shift_roi(mut self, dx: isize, dy: isize) -> Self {
        self.roi = self.roi.map(|roi| RoiDescriptor {
            x1: roi.x1 + dx,
            x2: roi.x2 + dx,
            y1: roi.y1 + dy,
            y2: roi.y2 + dy,
        });
        self
    }

    #[allow(dead_code)]
    pub fn with_default(mut self, default: &'a T) -> Self {
        self.default = Some(default);
        self
    }

    #[allow(dead_code)]
    pub fn build(self) -> ImageBufferWindow<'a, T> {
        let roi = self.roi.unwrap();
        let dist_from_x1_to_x2: usize = (roi.x2 - roi.x1).try_into().unwrap();
        let total_els: usize = ((roi.y2 - roi.y1 + 1) * (roi.x2 - roi.x1 + 1))
            .try_into()
            .unwrap();
        ImageBufferWindow {
            image: self.image,
            stride: self.stride.unwrap(),
            roi,
            default: self.default.unwrap(),
            dist_from_x1_to_x2,
            counter: 0,
            total_els,
        }
    }
}

impl<'a, T> ImageBufferWindow<'a, T> {
    #[allow(clippy::new_ret_no_self)]
    #[allow(dead_code)]
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
where
    T: Copy,
{
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.window.counter >= self.window.total_els {
            return None;
        }

        let counter = self.window.counter;
        self.window.counter += 1;

        let roi_x: isize = (counter % (self.window.dist_from_x1_to_x2 + 1)
            * self.window.stride.per_element)
            .try_into()
            .unwrap();
        let roi_y: isize = (counter / (self.window.dist_from_x1_to_x2 + 1)
            * self.window.stride.per_row)
            .try_into()
            .unwrap();

        let x: isize = self.window.roi.x1 + roi_x;
        let y: isize = self.window.roi.y1 + roi_y;
        if x < 0 || y < 0 {
            return Some(self.window.default);
        }

        let x: usize = x.try_into().unwrap();
        let y: usize = y.try_into().unwrap();
        if x >= self.window.image.width || y >= self.window.image.height {
            return Some(self.window.default);
        }

        let idx: usize = y * self.window.image.width + x;
        Some(&self.window.image.data[idx])
    }
}

impl<'a, T> IntoIterator for ImageBufferWindow<'a, T>
where
    T: Copy,
{
    type Item = &'a T;
    type IntoIter = ImageBufferWindowIterator<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        ImageBufferWindowIterator { window: self }
    }
}

// pub fn convolve<'a, T, U: Element>(image: ImageDescriptor<'a, T> , kernel: DynMatrix<U>) -> DynMatrix<f32>
// where
//     T: Copy + Into<f32> + Default,
//     U: Copy + Into<f32>,
// {
//     let mut result = DynMatrix::zeros(crate::dims::Dims(crate::dims::Rows(image.height), crate::dims::Cols(image.width)));
//     let mut kernel_sum: f32 = 0.0;
//     for row in kernel {
//         for el in row.iter() {
//             kernel_sum += (*el).into();
//         }
//     }

//     let mut windows: Vec<ImageBufferWindow<T>> = Vec::new();
//     for row in 0..kernel.rows() {
//         for col in 0..kernel.cols() {
//             let dx: isize = col.try_into().unwrap();
//             let dy: isize = row.try_into().unwrap();
//             let window = ImageBufferWindow::new(&image.data, image.width, image.height)
//                 .with_stride(1, 1)
//                 .with_max_roi()
//                 .shift_roi(dx, dy)
//                 .with_default(&T::default())
//                 .build();
//             windows.push(window);
//         }
//     }

//     for (y, x) in result.iter_mut() {
//         let mut sum: f32 = 0.0;
//         for (w, k) in windows.iter().zip(kernel.iter()) {
//             let w: f32 = (*w).into_iter().zip(k.iter()).fold(0.0, |acc, (w, k)| {
//                 acc + (*w).into() * (*k).into()
//             });
//             sum += w;
//         }
//         *x = sum / kernel_sum;
//     }

//     result
// }

#[cfg(test)]
mod tests {
    use super::*;
    extern crate test;
    use itertools::izip;
    use std::iter::zip;
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

        #[rustfmt::skip]
        let expected_vals = { vec![
            33, 34, 35, 36,
            43, 44, 45, 46,
            53, 54, 55, 56,
            63, 64, 65, 66,
        ] };
        for (i, v) in window.into_iter().enumerate() {
            println!("value in window: {v}");
            assert_eq!(*v, expected_vals[i]);
        }
    }

    #[test]
    fn out_of_bounds_returns_default_x() {
        let data: Vec<u8> = (0..100).collect();
        let window = ImageBufferWindow::new(&data, 10, 10)
            .with_stride(1, 1)
            .with_roi(10, 10, 0, 0)
            .with_default(&255)
            .build();

        let expected_vals = vec![255];
        for (i, v) in window.into_iter().enumerate() {
            println!("oob val x: {v}");
            assert_eq!(*v, expected_vals[i]);
        }
    }

    #[test]
    fn out_of_bounds_returns_default_y() {
        let data: Vec<u8> = (0..100).collect();
        let window = ImageBufferWindow::new(&data, 10, 10)
            .with_stride(1, 1)
            .with_roi(0, 0, 10, 10)
            .with_default(&255)
            .build();

        let expected_vals = vec![255];
        for (i, v) in window.into_iter().enumerate() {
            println!("oob val y: {v}");
            assert_eq!(*v, expected_vals[i]);
        }
    }

    #[test]
    fn out_of_bounds_returns_default_neg_x() {
        let data: Vec<u8> = (0..100).collect();
        let window = ImageBufferWindow::new(&data, 10, 10)
            .with_stride(1, 1)
            .with_roi(-1, -1, 0, 0)
            .with_default(&255)
            .build();

        let expected_vals = vec![255];
        for (i, v) in window.into_iter().enumerate() {
            println!("oob val -x: {v}");
            assert_eq!(*v, expected_vals[i]);
        }
    }

    #[test]
    fn out_of_bounds_returns_default_neg_y() {
        let data: Vec<u8> = (0..100).collect();
        let window = ImageBufferWindow::new(&data, 10, 10)
            .with_stride(1, 1)
            .with_roi(0, 0, -1, -1)
            .with_default(&255)
            .build();

        let expected_vals = vec![255];
        for (i, v) in window.into_iter().enumerate() {
            println!("oob val -y: {v}");
            assert_eq!(*v, expected_vals[i]);
        }
    }

    #[test]
    fn convolve_with_many_iterators() {
        /*
        IMAGE:

        00, 01, 02, 03, 04,
        05, 06, 07, 08, 09,
        10, 11, 12, 13, 14,
        15, 16, 17, 18, 19,
        20, 21, 22, 23, 24,

        */

        let data: Vec<u8> = (0..25).collect();

        #[rustfmt::skip]
        let shifts = { vec![
            (-1, -1), (0, -1), (1, -1),
            (-1,  0), (0,  0), (1,  0),
            (-1,  1), (0,  1), (1,  1),
        ] };
        let windows: Vec<ImageBufferWindow<u8>> = shifts
            .iter()
            .map(|(dx, dy)| {
                ImageBufferWindow::new(&data, 5, 5)
                    .with_stride(1, 1)
                    .with_max_roi()
                    .shift_roi(*dx, *dy)
                    .with_default(&0)
                    .build()
            })
            .collect();

        let mut results = [0f32; 25];
        let box_3x3: Vec<i16> = vec![1, 1, 1, 1, 1, 1, 1, 1, 1];
        let box_3x3: Vec<f32> = box_3x3
            .iter()
            .map(|x| {
                let normalized: f32 = (*x).try_into().unwrap();
                normalized / 9f32
            })
            .collect();

        // Below we transpose `windows` so that, instead of a vector of iterable windows, we have
        // an iterable collection where each element N is a tuple of all the "Nth" index of each of
        // the windows
        let transposed_windows = izip!(
            windows[0].into_iter(),
            windows[1].into_iter(),
            windows[2].into_iter(),
            windows[3].into_iter(),
            windows[4].into_iter(),
            windows[5].into_iter(),
            windows[6].into_iter(),
            windows[7].into_iter(),
            windows[8].into_iter()
        );

        for (i, ws) in transposed_windows.enumerate() {
            let mut sum: f32 = 0.0;
            let neighborhood = &[
                &ws.0, &ws.1, &ws.2, &ws.3, &ws.4, &ws.5, &ws.6, &ws.7, &ws.8,
            ];
            for (w, g) in zip(neighborhood.iter(), box_3x3.iter()) {
                let w: f32 = (***w).try_into().unwrap();
                sum += w * g;
            }
            results[i] = sum;
        }

        #[rustfmt::skip]
        let expected_results: Vec<f32> = vec![
             1.3333334,  2.3333335, 3.0,  3.6666667, 2.6666667,
             3.666667,   6.0000005, 7.0,  8.0,       5.666667,
             7.0,       11.000001, 12.0, 13.0,       9.0,
            10.333333,  16.0,      17.0, 17.999998, 12.333334,
             8.0,       12.333334, 13.0, 13.666667,  9.333334,
        ];

        for (e, v) in zip(expected_results, results) {
            assert_eq!(v, e);
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
