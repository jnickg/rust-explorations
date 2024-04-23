use std::ops::{Add, Index, IndexMut, Mul, Sub};
use utoipa::ToSchema;
use crate::{dims::{Dims, HasDims}, element::Element};

/// A matrix of elements of type `T`, with `R` rows and `C` columns.
#[derive(Debug, Clone, ToSchema, PartialEq, Eq)]
pub struct Matrix<T: Element, const R: usize, const C: usize> {
    /// The elements of this matrix
    els: [[T; C]; R],
}

impl<T: Element, const R: usize, const C: usize> Matrix<T, R, C> {
    /// Create a new matrix with all elements set to zero.
    pub fn zeros() -> Self {
        Self {
            els: [[T::zero(); C]; R],
        }
    }

    pub fn zeros_like(_: &Self) -> Self {
        Self {
            els: [[T::zero(); C]; R],
        }
    }

    pub fn ones_like(_: &Self) -> Self {
        Self {
            els: [[T::one(); C]; R],
        }
    }

    /// Create a new matrix of the given size from a flat array
    pub fn from_flat(data: &[T]) -> Self {
        let mut matrix = Self::zeros();
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
    pub fn from_nested(data: &[[T; C]; R]) -> Self {
        Self {
            els: *data,
        }
    }

    pub fn from_vec(data: &[Vec<T>]) -> Self {
        let mut matrix = Self::zeros();
        let data_sz = data.len() * data[0].len(); // Outer array can never be empty
        assert_eq!(data_sz, R * C, "Data size does not match matrix size");
        for (i, row) in data.iter().enumerate().take(R) {
            for (j, el) in row.iter().enumerate().take(C) {
                matrix.els[i][j] = *el;
            }
        }
        matrix
    }

    pub fn identity() -> Self {
        let mut matrix = Self::zeros();
        for i in 0..R {
            matrix.els[i][i] = T::one();
        }
        matrix
    }

    pub fn identity_like(_: &Self) -> Self {
        let mut matrix = Self::zeros();
        for i in 0..R {
            matrix.els[i][i] = T::one();
        }
        matrix
    }

    pub fn transpose(&self) -> Matrix<T, C, R> {
        let mut result = Matrix::<T, C, R>::zeros();
        for i in 0..R {
            for j in 0..C {
                result[(j, i)] = self[(i, j)];
            }
        }
        result
    }
}

impl<T: Element, const R: usize, const C: usize> HasDims for Matrix<T, R, C> {
    fn rows(&self) -> usize {
        R
    }

    fn cols(&self) -> usize {
        C
    }

    fn dims(&self) -> Dims {
        (self.rows(), self.cols()).into()
    }
}

pub struct MatrixIterator<'a, T: Element, const R: usize, const C: usize> {
    matrix: &'a Matrix<T, R, C>,
    row: usize,
}

impl<'a, T: Element, const R: usize, const C: usize> Iterator for MatrixIterator<'a, T, R, C> {
    type Item = [T; C];

    fn next(&mut self) -> Option<Self::Item> {
        if self.row < R {
            let result = self.matrix.els[self.row];
            self.row += 1;
            Some(result)
        } else {
            None
        }
    
    }
}

pub struct MatrixIntoIterator<T: Element, const R: usize, const C: usize> {
    matrix: Matrix<T, R, C>,
    row: usize,
}

impl<T: Element, const R: usize, const C: usize> Iterator for MatrixIntoIterator<T, R, C> {
    type Item = [T; C];

    fn next(&mut self) -> Option<Self::Item> {
        if self.row < R {
            let result = self.matrix.els[self.row];
            self.row += 1;
            Some(result)
        } else {
            None
        }
    }
}

impl<T: Element, const R: usize, const C: usize> IntoIterator for Matrix<T, R, C> {
    type Item = [T; C];
    type IntoIter = MatrixIntoIterator<T, R, C>;

    fn into_iter(self) -> Self::IntoIter {
        MatrixIntoIterator {
            matrix: self,
            row: 0,
        }
    }
}

impl<'a, T: Element, const R: usize, const C: usize> IntoIterator for &'a Matrix<T, R, C> {
    type Item = [T; C];
    type IntoIter = MatrixIterator<'a, T, R, C>;

    fn into_iter(self) -> Self::IntoIter {
        MatrixIterator {
            matrix: self,
            row: 0,
        }
    }
}

impl<T: Element, const R: usize, const C: usize> Index<usize> for Matrix<T, R, C> {
    type Output = [T; C];

    fn index(&self, r: usize) -> &Self::Output {
        &self.els[r]
    }
}

impl<T: Element, const R: usize, const C: usize> IndexMut<usize> for Matrix<T, R, C> {
    fn index_mut(&mut self, r: usize) -> &mut Self::Output {
        &mut self.els[r]
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
        let mut result = Matrix::<T, R, C>::zeros();
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
        let mut result = Matrix::<T, R, C>::zeros();
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

impl<T: Element, const R1: usize, const I: usize, const C2: usize> DotProduct<T, I, C2> for Matrix<T, R1, I> {
    type Output = Matrix<T, R1, C2>;

    fn dot_product(&self, m2: Matrix<T, I, C2>) -> Self::Output {
        let mut result = Matrix::<T, R1, C2>::zeros();
        for i in 0..R1 {
            for j in 0..C2 {
                for k in 0..I {
                    result[(i, j)] += self[(i, k)] * m2[(k, j)];
                }
            }
        }
        result
    }
}

impl<T: Element, const R1: usize, const I: usize, const C2: usize> Mul<Matrix<T, I, C2>> for Matrix<T, R1, I> {
    type Output = Matrix<T, R1, C2>;

    fn mul(self, m2: Matrix<T, I, C2>) -> Self::Output {
        let mut result = Matrix::<T, R1, C2>::zeros();
        for i in 0..R1 {
            for j in 0..C2 {
                for k in 0..I {
                    result[(i, j)] += self[(i, k)] * m2[(k, j)];
                }
            }
        }
        result
    }
}

impl<T: Element, const R: usize, const C: usize> Mul<T> for Matrix<T, R, C> {
    type Output = Matrix<T, R, C>;

    fn mul(self, scalar: T) -> Self::Output {
        let mut result = Matrix::<T, R, C>::zeros();
        for i in 0..R {
            for j in 0..C {
                result[(i, j)] = self[(i, j)] * scalar;
            }
        }
        result
    }
}

impl<T: Element, const R: usize, const C: usize> From<Matrix<T, R, C>> for [[T; C]; R] {
    fn from(matrix: Matrix<T, R, C>) -> Self {
        matrix.els
    }
}

#[cfg(test)]
mod tests {
    use crate::from_mat::FromMat;
    use super::*;

    #[test]
    fn zeros() {
        let matrix = Matrix::<u8, 2, 2>::zeros();
        assert_eq!(matrix[(0, 0)], 0);
        assert_eq!(matrix[(0, 1)], 0);
        assert_eq!(matrix[(1, 0)], 0);
        assert_eq!(matrix[(1, 1)], 0);
    }

    #[test]
    fn from_flat() {
        let matrix = Matrix::<u8, 2, 2>::from_flat(&[1, 2, 3, 4]);
        assert_eq!(matrix[(0, 0)], 1);
        assert_eq!(matrix[(0, 1)], 2);
        assert_eq!(matrix[(1, 0)], 3);
        assert_eq!(matrix[(1, 1)], 4);
    }

    #[test]
    fn from() {
        let matrix = Matrix::from_nested(&[[1, 2], [3, 4]]);
        assert_eq!(matrix[(0, 0)], 1);
        assert_eq!(matrix[(0, 1)], 2);
        assert_eq!(matrix[(1, 0)], 3);
        assert_eq!(matrix[(1, 1)], 4);
    }

    #[test]
    fn identity() {
        let matrix = Matrix::<u8, 2, 2>::identity();
        assert_eq!(matrix[(0, 0)], 1);
        assert_eq!(matrix[(0, 1)], 0);
        assert_eq!(matrix[(1, 0)], 0);
        assert_eq!(matrix[(1, 1)], 1);
    }


    #[test]
    fn indexing() {
        let mut matrix = Matrix::<u8, 2, 2>::zeros();
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
        let matrix = Matrix::<u8, 2, 2>::from_flat(&[1, 2, 3, 4]);
        let result = matrix.transpose();
        assert_eq!(result[(0, 0)], 1);
        assert_eq!(result[(0, 1)], 3);
        assert_eq!(result[(1, 0)], 2);
        assert_eq!(result[(1, 1)], 4);
    }

    #[test]
    fn transpose_3x2() {
        let matrix = Matrix::<u8, 3, 2>::from_flat(&[1, 2, 3, 4, 5, 6]);
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
        let matrix1 = Matrix::<u8, 2, 2>::from_flat(&[1, 2, 3, 4]);
        let matrix2 = Matrix::<u8, 2, 2>::from_flat(&[5, 6, 7, 8]);
        let result = matrix1 + matrix2;
        assert_eq!(result[(0, 0)], 6);
        assert_eq!(result[(0, 1)], 8);
        assert_eq!(result[(1, 0)], 10);
        assert_eq!(result[(1, 1)], 12);
    }

    #[test]
    fn sub() {
        let matrix1 = Matrix::<i8, 2, 2>::from_flat(&[1, 2, 3, 4]);
        let matrix2 = Matrix::<i8, 2, 2>::from_flat(&[5, 6, 7, 8]);
        let result = matrix2 - matrix1;
        assert_eq!(result[(0, 0)], 4);
        assert_eq!(result[(0, 1)], 4);
        assert_eq!(result[(1, 0)], 4);
        assert_eq!(result[(1, 1)], 4);
    }

    #[test]
    fn dot_product_fn() {
        let matrix1 = Matrix::<u8, 2, 2>::from_flat(&[1, 2, 3, 4]);
        let matrix2 = Matrix::<u8, 2, 2>::from_flat(&[5, 6, 7, 8]);
        let result = matrix1.dot_product(matrix2);
        assert_eq!(result[(0, 0)], 19);
        assert_eq!(result[(0, 1)], 22);
        assert_eq!(result[(1, 0)], 43);
        assert_eq!(result[(1, 1)], 50);
    }

    #[test]
    fn dot_product_fn_1x2_dot_2x1() {
        let matrix1 = Matrix::<u8, 1, 2>::from_flat(&[1, 2]);
        let matrix2 = Matrix::<u8, 2, 1>::from_flat(&[3, 4]);
        let result = matrix1.dot_product(matrix2);
        assert_eq!(result[(0, 0)], 11);
    }

    #[test]
    fn mul_matrix() {
        let matrix1 = Matrix::<u8, 2, 2>::from_flat(&[1, 2, 3, 4]);
        let matrix2 = Matrix::<u8, 2, 2>::from_flat(&[5, 6, 7, 8]);
        let result = matrix1 * matrix2;
        assert_eq!(result[(0, 0)], 19);
        assert_eq!(result[(0, 1)], 22);
        assert_eq!(result[(1, 0)], 43);
        assert_eq!(result[(1, 1)], 50);
    }

    #[test]
    fn mul_by_identity_like_yields_same() {
        let matrix = Matrix::<u8, 2, 2>::from_flat(&[1, 2, 3, 4]);
        let identity = Matrix::<u8, 2, 2>::identity_like(&matrix);
        let result = matrix * identity;
        assert_eq!(result[(0, 0)], 1);
        assert_eq!(result[(0, 1)], 2);
        assert_eq!(result[(1, 0)], 3);
        assert_eq!(result[(1, 1)], 4);
    }

    #[test]
    fn mul_scalar() {
        let matrix = Matrix::<u8, 2, 2>::from_flat(&[1, 2, 3, 4]);
        let result = matrix * 2;
        assert_eq!(result[(0, 0)], 2);
        assert_eq!(result[(0, 1)], 4);
        assert_eq!(result[(1, 0)], 6);
        assert_eq!(result[(1, 1)], 8);
    }

    #[test]
    fn from_other_element_type() {
        let matrix = Matrix::<u8, 2, 2>::from_flat(&[1, 2, 3, 4]);
        let result = Matrix::<u16, 2, 2>::from_mat(matrix);
        assert_eq!(result[(0, 0)], 1);
        assert_eq!(result[(0, 1)], 2);
        assert_eq!(result[(1, 0)], 3);
        assert_eq!(result[(1, 1)], 4);
    }


}
