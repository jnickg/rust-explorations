use std::ops::{Add, Index, IndexMut, Mul, Sub};
use serde::{Serialize, Deserialize};
use utoipa::ToSchema;

use crate::dims::{Dims, Rows, Cols, HasDims};
use crate::element::Element;
use crate::matrix_type::{Indexible, IsMatrix};
// use crate::my_traits::{AreNotSame, IsTrue, Multiplied, TheTypes, Values, AreEqual};


/// A matrix of elements of type `T`, with `R` rows and `C` columns.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DynMatrix<T: Element> {
    /// The elements of this matrix
    #[schema(example = json!([[1,0],[0,1]]))]
    els: Vec<Vec<T>>,
}

impl<T: Element> DynMatrix<T> {
    /// Create a new matrix with all elements set to zero.
    pub fn zeros<D>(dims: D) -> Self
        where D: Into<Dims>
    {
        let Dims(Rows(r), Cols(c)) = dims.into();
        Self {
            els: vec![vec![T::zero(); c]; r],
        }
    }

    pub fn zeros_like(m: &Self) -> Self {
        Self {
            els: vec![vec![T::zero(); m.cols()]; m.rows()],
        }
    }

    pub fn ones<D>(dims: D) -> Self
        where D: Into<Dims>
    {
        let Dims(Rows(r), Cols(c)) = dims.into();
        Self {
            els: vec![vec![T::one(); c]; r],
        }
    }


    pub fn ones_like(m: &Self) -> Self {
        Self {
            els: vec![vec![T::one(); m.cols()]; m.rows()],
        }
    }

    /// Create a new matrix of the given size from a flat array
    pub fn from_flat<D>(data: &[T], dims: D) -> Self
        where D: Into<Dims>
    {
        let Dims(Rows(r), Cols(c)) = dims.into();
        let mut matrix = Self::zeros((r, c));
        for i in 0..r {
            for j in 0..c {
                matrix.els[i][j] = data[i * c + j];
            }
        }
        matrix
    
    }

    /// Create a new matrix of the given size from a nested array
    pub fn from_nested<const R: usize, const C: usize>(data: &[[T; C]; R]) -> Self {
        let mut matrix = Self::zeros((R, C));
        for (i, row) in data.iter().enumerate().take(R) {
            for (j, el) in row.iter().enumerate().take(C) {
                matrix.els[i][j] = *el;
            }
        }
        matrix
    
    }

    pub fn identity<D>(dims: D) -> Self
        where D: Into<Dims>
    {
        let Dims(Rows(r), Cols(c)) = dims.into();
        let mut matrix = Self::zeros((r, c));
        for i in 0..r {
            matrix.els[i][i] = T::one();
        }
        matrix
    }

    pub fn identity_like(m: &Self) -> Self {
        let mut matrix = Self::zeros(m.dims());
        for i in 0..m.rows() {
            matrix.els[i][i] = T::one();
        }
        matrix
    
    }

    pub fn transpose(&self) -> Self {
        let mut result = Self::zeros(Dims(Rows(self.cols()), Cols(self.rows())));
        for i in 0..self.rows() {
            for j in 0..self.cols() {
                result.els[j][i] = self.els[i][j];
            }
        }
        result
    }
}

impl<T: Element> HasDims for DynMatrix<T> {
    fn rows(&self) -> usize {
        self.els.len()
    }

    fn cols(&self) -> usize {
        self.els[0].len()
    }

    fn dims(&self) -> Dims {
        (self.rows(), self.cols()).into()
    }
}

impl<T: Element> Indexible<T> for DynMatrix<T> {
    fn at(&self, d: Dims) -> T {
        let Dims(Rows(r), Cols(c)) = d;
        self.els[r][c]
    }

    fn at_mut(&mut self, d: Dims) -> &mut T {
        let Dims(Rows(r), Cols(c)) = d;
        &mut self.els[r][c]
    }
}

impl<T: Element> IsMatrix<T> for DynMatrix<T> {
    fn zeros<D>(dims: D) -> Self
        where D: Into<Dims>
    {
        Self::zeros(dims)
    }

    fn zeros_like(m: &Self) -> Self {
        Self::zeros_like(m)
    }

    fn ones<D>(dims: D) -> Self
        where D: Into<Dims>
    {
        Self::ones(dims)
    }

    fn ones_like(m: &Self) -> Self {
        Self::ones_like(m)
    }
}


impl<T: Element> Add for DynMatrix<T> {
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        assert_eq!(self.rows(), other.rows());
        assert_eq!(self.cols(), other.cols());
        let mut result = DynMatrix::<T>::zeros(self.dims());
        for i in 0..self.rows() {
            for j in 0..self.cols() {
                result[(i, j)] = self[(i, j)] + other[(i, j)];
            }
        }
        result
    }
}

impl<T: Element> Sub for DynMatrix<T> {
    type Output = Self;

    fn sub(self, other: Self) -> Self::Output {
        let mut result = DynMatrix::zeros_like(&self);
        for i in 0..self.rows() {
            for j in 0..self.cols() {
                result[(i, j)] = self[(i, j)] - other[(i, j)];
            }
        }
        result
    }
}

impl<T: Element> Mul for DynMatrix<T> {
    type Output = Self;

    fn mul(self, other: DynMatrix<T>) -> Self::Output {
        assert_eq!(self.cols(), other.rows());
        let mut result = DynMatrix::zeros((self.rows(), other.cols()));
        for i in 0..self.rows() {
            for j in 0..other.cols() {
                for k in 0..self.cols() {
                    result[(i, j)] += self[(i, k)] * other[(k, j)];
                }
            }
        }
        result
    }
}

impl<T: Element> Mul<T> for DynMatrix<T> {
    type Output = Self;

    fn mul(self, scalar: T) -> Self::Output {
        let mut result = DynMatrix::zeros_like(&self);
        for i in 0..self.rows() {
            for j in 0..self.cols() {
                result[(i, j)] = self[(i, j)] * scalar;
            }
        }
        result
    }
}

impl<T: Element> From<DynMatrix<T>> for Vec<Vec<T>> {
    fn from(matrix: DynMatrix<T>) -> Self {
        matrix.els
    }
}

#[cfg(test)]
mod tests {
    use crate::{from_mat::{FromDynMat, FromMat}, matrix::Matrix};

    use super::*;

    #[test]
    fn zeros() {
        let matrix = DynMatrix::<u8>::zeros((2, 2));
        
        assert_eq!(matrix[(0, 0)], 0);
        assert_eq!(matrix[(0, 1)], 0);
        assert_eq!(matrix[(1, 0)], 0);
        assert_eq!(matrix[(1, 1)], 0);
    }

    #[test]
    fn from_flat() {
        let matrix = DynMatrix::from_flat(&[1, 2, 3, 4], (2, 2));
        assert_eq!(matrix[(0, 0)], 1);
        assert_eq!(matrix[(0, 1)], 2);
        assert_eq!(matrix[(1, 0)], 3);
        assert_eq!(matrix[(1, 1)], 4);
    }

    #[test]
    fn from() {
        let matrix = DynMatrix::from_nested(&[[1, 2], [3, 4]]);
        assert_eq!(matrix[(0, 0)], 1);
        assert_eq!(matrix[(0, 1)], 2);
        assert_eq!(matrix[(1, 0)], 3);
        assert_eq!(matrix[(1, 1)], 4);
    }

    #[test]
    fn identity() {
        let matrix = DynMatrix::<u8>::identity((2, 2));
        assert_eq!(matrix[(0, 0)], 1);
        assert_eq!(matrix[(0, 1)], 0);
        assert_eq!(matrix[(1, 0)], 0);
        assert_eq!(matrix[(1, 1)], 1);
    }


    #[test]
    fn indexing() {
        let mut matrix = DynMatrix::<u8>::zeros((2, 2));
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
        let matrix = DynMatrix::<u8>::from_flat(&[1, 2, 3, 4], (2, 2));
        let result = matrix.transpose();
        assert_eq!(result[(0, 0)], 1);
        assert_eq!(result[(0, 1)], 3);
        assert_eq!(result[(1, 0)], 2);
        assert_eq!(result[(1, 1)], 4);
    }

    #[test]
    fn transpose_3x2() {
        let matrix = DynMatrix::<u8>::from_flat(&[1, 2, 3, 4, 5, 6], (3, 2));
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
        let matrix1 = DynMatrix::<u8>::from_flat(&[1, 2, 3, 4], (2,2));
        let matrix2 = DynMatrix::<u8>::from_flat(&[5, 6, 7, 8], (2,2));
        let result = matrix1 + matrix2;
        assert_eq!(result[(0, 0)], 6);
        assert_eq!(result[(0, 1)], 8);
        assert_eq!(result[(1, 0)], 10);
        assert_eq!(result[(1, 1)], 12);
    }

    #[test]
    fn sub() {
        let matrix1 = DynMatrix::<i8>::from_flat(&[1, 2, 3, 4], (2, 2));
        let matrix2 = DynMatrix::<i8>::from_flat(&[5, 6, 7, 8], (2, 2));
        let result = matrix2 - matrix1;
        assert_eq!(result[(0, 0)], 4);
        assert_eq!(result[(0, 1)], 4);
        assert_eq!(result[(1, 0)], 4);
        assert_eq!(result[(1, 1)], 4);
    }

    #[test]
    fn mul_matrix() {
        let matrix1 = DynMatrix::from_flat(&[1, 2, 3, 4], (2, 2));
        let matrix2 = DynMatrix::from_flat(&[5, 6, 7, 8], (2, 2));
        let result = matrix1 * matrix2;
        assert_eq!(result[(0, 0)], 19);
        assert_eq!(result[(0, 1)], 22);
        assert_eq!(result[(1, 0)], 43);
        assert_eq!(result[(1, 1)], 50);
    }

    #[test]
    fn mul_by_identity_like_yields_same() {
        let matrix = DynMatrix::from_flat(&[1, 2, 3, 4], (2, 2));
        let identity = DynMatrix::identity_like(&matrix);
        let result = matrix * identity;
        assert_eq!(result[(0, 0)], 1);
        assert_eq!(result[(0, 1)], 2);
        assert_eq!(result[(1, 0)], 3);
        assert_eq!(result[(1, 1)], 4);
    }

    #[test]
    fn mul_scalar() {
        let matrix = DynMatrix::<u8>::from_flat(&[1, 2, 3, 4], (2, 2));
        let result = matrix * 2;
        assert_eq!(result[(0, 0)], 2);
        assert_eq!(result[(0, 1)], 4);
        assert_eq!(result[(1, 0)], 6);
        assert_eq!(result[(1, 1)], 8);
    }

    #[test]
    fn from_other_element_type_dyn_mat() {
        let matrix = DynMatrix::<u8>::from_flat(&[1, 2, 3, 4], (2, 2));
        let result = DynMatrix::<u16>::from_dyn_mat(matrix);
        assert_eq!(result[(0, 0)], 1);
        assert_eq!(result[(0, 1)], 2);
        assert_eq!(result[(1, 0)], 3);
        assert_eq!(result[(1, 1)], 4);
    }

    #[test]
    fn from_other_element_type_mat() {
        let matrix = Matrix::<u8, 2, 2>::from_flat(&[1, 2, 3, 4]);
        let result = DynMatrix::<u16>::from_mat(matrix);
        assert_eq!(result[(0, 0)], 1);
        assert_eq!(result[(0, 1)], 2);
        assert_eq!(result[(1, 0)], 3);
        assert_eq!(result[(1, 1)], 4);
    }
}
