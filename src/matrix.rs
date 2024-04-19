use num::{Zero, Num};
use std::ops::{Index, IndexMut, Mul, Add, Sub};

pub trait Element: Num + Clone + Default + Copy + Zero {}
impl<T> Element for T where T: Num + Clone + Default + Copy + Zero{}

/// A matrix of elements of type `T`, with `R` rows and `C` columns.
pub struct Matrix<T: Element, const R: usize, const C: usize> {
    /// The elements of this matrix
    els: [[T; C]; R],
}

impl<T: Element, const R: usize, const C: usize> Matrix<T, R, C> {
    /// Create a new matrix with all elements set to zero.
    pub fn new() -> Self {
        Self {
            els: [[T::zero(); C]; R],
        }
    }

    /// Create a new matrix of the given size from a flat array
    pub fn new_from_flat(data: &[T]) -> Self {
        let mut matrix = Self::new();
        let data_sz = data.len();
        assert_eq!(data_sz, R * C, "Data size does not match matrix size");
        
        for i in 0..R {
            for j in 0..C {
                matrix.els[i][j] = data[i * C + j];
            }
        }
        matrix
    }

    /// Create a new matrix of the given size from a nested array
    pub fn new_from_nested(data: &[[T; C]; R]) -> Self {
        Self {
            els: data.clone(),
        }
    }

    pub fn new_identity() -> Self {
        let mut matrix = Self::new();
        for i in 0..R {
            matrix.els[i][i] = T::one();
        }
        matrix
    }

    pub fn transpose(&self) -> Matrix<T, C, R> {
        let mut result = Matrix::<T, C, R>::new();
        for i in 0..R {
            for j in 0..C {
                result[(j, i)] = self[(i, j)];
            }
        }
        result
    }
}

impl<T: Element, const R: usize, const C: usize> Index<(usize, usize)> for Matrix<T, R, C> {
    type Output = T;

    fn index(&self, (x, y): (usize, usize)) -> &Self::Output {
        &self.els[x][y]
    }
}

impl<T: Element, const R: usize, const C: usize> IndexMut<(usize, usize)> for Matrix<T, R, C> {
    fn index_mut(&mut self, (x, y): (usize, usize)) -> &mut Self::Output {
        &mut self.els[x][y]
    }
}

impl<T: Element, const R: usize, const C: usize> Add for Matrix<T, R, C> {
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        let mut result = Matrix::<T, R, C>::new();
        for i in 0..R {
            for j in 0..C {
                result[(i, j)] = self[(i, j)] + other[(i, j)];
            }
        }
        result
    }
}

impl<T: Element, const R: usize, const C: usize> Sub for Matrix<T, R, C> {
    type Output = Self;

    fn sub(self, other: Self) -> Self::Output {
        let mut result = Matrix::<T, R, C>::new();
        for i in 0..R {
            for j in 0..C {
                result[(i, j)] = self[(i, j)] - other[(i, j)];
            }
        }
        result
    }
}

trait DotProduct<T: Element, const R2: usize, const C2: usize> {
    type Output: ?Sized;

    fn dot_product(&self, m2: Matrix<T, R2, C2>) -> Self::Output;
}

impl<T: Element, const R1: usize, const C1: usize, const R2: usize, const C2: usize> DotProduct<T, R2, C2> for Matrix<T, R1, C1> {
    type Output = Matrix<T, R1, C2>;

    fn dot_product(&self, m2: Matrix<T, R2, C2>) -> Self::Output {
        let mut result = Matrix::<T, R1, C2>::new();
        for i in 0..R1 {
            for j in 0..C2 {
                for k in 0..C1 {
                    result[(i, j)] = result[(i, j)] + self[(i, k)] * m2[(k, j)];
                }
            }
        }
        result
    }
}

// ????????
// impl<T: Element, const R1: usize, const C1: usize, const R2: usize, const C2: usize> Mul for Matrix<T, R1, C1> {
//     type Output = Matrix<T, R1, C2>;

//     fn mul(&self, m2: Matrix<T, R2, C2>) -> Self::Output {
//         let mut result = Matrix::<T, R1, C2>::new();
//         for i in 0..R1 {
//             for j in 0..C2 {
//                 for k in 0..C1 {
//                     result[(i, j)] = result[(i, j)] + self[(i, k)] * m2[(k, j)];
//                 }
//             }
//         }
//         result
//     }
// }


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new() {
        let matrix = Matrix::<u8, 2, 2>::new();
        assert_eq!(matrix[(0, 0)], 0);
        assert_eq!(matrix[(0, 1)], 0);
        assert_eq!(matrix[(1, 0)], 0);
        assert_eq!(matrix[(1, 1)], 0);
    }

    #[test]
    fn new_from_flat() {
        let matrix = Matrix::<u8, 2, 2>::new_from_flat(&[1, 2, 3, 4]);
        assert_eq!(matrix[(0, 0)], 1);
        assert_eq!(matrix[(0, 1)], 2);
        assert_eq!(matrix[(1, 0)], 3);
        assert_eq!(matrix[(1, 1)], 4);
    }

    #[test]
    fn new_from_nested() {
        let matrix = Matrix::<u8, 2, 2>::new_from_nested(&[[1, 2], [3, 4]]);
        assert_eq!(matrix[(0, 0)], 1);
        assert_eq!(matrix[(0, 1)], 2);
        assert_eq!(matrix[(1, 0)], 3);
        assert_eq!(matrix[(1, 1)], 4);
    }

    #[test]
    fn new_identity() {
        let matrix = Matrix::<u8, 2, 2>::new_identity();
        assert_eq!(matrix[(0, 0)], 1);
        assert_eq!(matrix[(0, 1)], 0);
        assert_eq!(matrix[(1, 0)], 0);
        assert_eq!(matrix[(1, 1)], 1);
    }


    #[test]
    fn indexing() {
        let mut matrix = Matrix::<u8, 2, 2>::new();
        matrix[(0, 0)] = 255;
        matrix[(0, 1)] = 128;
        matrix[(1, 0)] = 0;
        matrix[(1, 1)] = 128;
        assert_eq!(matrix[(0, 0)], 255);
        assert_eq!(matrix[(0, 1)], 128);
        assert_eq!(matrix[(1, 0)], 0);
        assert_eq!(matrix[(1, 1)], 128);
    }

    #[test]
    fn transpose_2x2() {
        let matrix = Matrix::<u8, 2, 2>::new_from_flat(&[1, 2, 3, 4]);
        let result = matrix.transpose();
        assert_eq!(result[(0, 0)], 1);
        assert_eq!(result[(0, 1)], 3);
        assert_eq!(result[(1, 0)], 2);
        assert_eq!(result[(1, 1)], 4);
    }

    #[test]
    fn transpose_3x2() {
        let matrix = Matrix::<u8, 3, 2>::new_from_flat(&[1, 2, 3, 4, 5, 6]);
        let result = matrix.transpose();
        assert_eq!(result[(0, 0)], 1);
        assert_eq!(result[(0, 1)], 3);
        assert_eq!(result[(0, 2)], 5);
        assert_eq!(result[(1, 0)], 2);
        assert_eq!(result[(1, 1)], 4);
        assert_eq!(result[(1, 2)], 6);
    }

    #[test]
    fn add() {
        let matrix1 = Matrix::<u8, 2, 2>::new_from_flat(&[1, 2, 3, 4]);
        let matrix2 = Matrix::<u8, 2, 2>::new_from_flat(&[5, 6, 7, 8]);
        let result = matrix1 + matrix2;
        assert_eq!(result[(0, 0)], 6);
        assert_eq!(result[(0, 1)], 8);
        assert_eq!(result[(1, 0)], 10);
        assert_eq!(result[(1, 1)], 12);
    }

    #[test]
    fn sub() {
        let matrix1 = Matrix::<i8, 2, 2>::new_from_flat(&[1, 2, 3, 4]);
        let matrix2 = Matrix::<i8, 2, 2>::new_from_flat(&[5, 6, 7, 8]);
        let result = matrix2 - matrix1;
        assert_eq!(result[(0, 0)], 4);
        assert_eq!(result[(0, 1)], 4);
        assert_eq!(result[(1, 0)], 4);
        assert_eq!(result[(1, 1)], 4);
    }

    #[test]
    fn dot_product_fn() {
        let matrix1 = Matrix::<u8, 2, 2>::new_from_flat(&[1, 2, 3, 4]);
        let matrix2 = Matrix::<u8, 2, 2>::new_from_flat(&[5, 6, 7, 8]);
        let result = matrix1.dot_product(matrix2);
        assert_eq!(result[(0, 0)], 19);
        assert_eq!(result[(0, 1)], 22);
        assert_eq!(result[(1, 0)], 43);
        assert_eq!(result[(1, 1)], 50);
    }

    #[test]
    fn dot_product_fn_1x2_dot_2x1() {
        let matrix1 = Matrix::<u8, 1, 2>::new_from_flat(&[1, 2]);
        let matrix2 = Matrix::<u8, 2, 1>::new_from_flat(&[3, 4]);
        let result = matrix1.dot_product(matrix2);
        assert_eq!(result[(0, 0)], 11);
    }

    #[test]
    fn dot_product_fn_1x2_dot_1x1_fails() {
        // TODO This should FAIL TO COMPILE, not throw a runtime error
        let matrix1 = Matrix::<u8, 1, 2>::new_from_flat(&[1, 2]);
        let matrix2 = Matrix::<u8, 1, 1>::new_from_flat(&[3]);
        let result = matrix1.dot_product(matrix2);
        assert_eq!(result[(0, 0)], 3);
    }

    // #[test]
    // fn multiply() {
    //     let matrix1 = Matrix::<u8, 2, 2>::new_from_flat(&[1, 2, 3, 4]);
    //     let matrix2 = Matrix::<u8, 2, 2>::new_from_flat(&[5, 6, 7, 8]);
    //     let result = matrix1 * matrix2;
    //     assert_eq!(result[(0, 0)], 19);
    //     assert_eq!(result[(0, 1)], 22);
    //     assert_eq!(result[(1, 0)], 43);
    //     assert_eq!(result[(1, 1)], 50);
    // }
}
