use crate::dims::HasDims;
use crate::element::Element;
use crate::matrix::Matrix;
use crate::dyn_matrix::DynMatrix;
use crate::my_traits::{AreNotSame, TheTypes};
use crate::matrix_type::MatrixType;

pub trait FromMat<T: Element, const R: usize, const C: usize> {
    fn from_mat(m: Matrix<T, R, C>) -> Self;
}

impl<T: Element, U: Element, const R: usize, const C: usize> FromMat<T, R, C> for DynMatrix<U>
    where TheTypes<T, U> : AreNotSame,
          T : Into<U>
{
    fn from_mat(matrix: Matrix<T, R, C>) -> Self {
        let mut result = DynMatrix::<U>::zeros(matrix.dims());
        for i in 0..matrix.rows() {
            for j in 0..matrix.cols() {
                result[(i, j)] = matrix[(i, j)].into();
            }
        }
        result
    }
}

impl<T: Element, const R: usize, const C: usize> FromMat<T,R,C> for MatrixType<T, DynMatrix<T>> {
    fn from_mat(m: Matrix<T, R, C>) -> Self {
        DynMatrix::<T>::from_mat(m)
    }
}

impl<U: Element, T: Element, const R: usize, const C: usize> From<Matrix<T,R,C>> for MatrixType<U, DynMatrix<U>> {
    fn from(matrix: Matrix<T,R,C>) -> Self {
        U::from_mat(matrix)
    }
}

// impl<T: Element, U: Element, const R: usize, const C: usize> From<Matrix<T, R, C>> for Matrix<U, R, C> {
//     fn from(matrix: Matrix<T, R, C>) -> Self {
//         Self::from_mat(matrix)
//     }
// }

impl<T: Element, U: Element, const R: usize, const C: usize> FromMat<T, R, C> for Matrix<U, R, C>
    where TheTypes<T, U> : AreNotSame,
          T : Into<U>
{
    fn from_mat(matrix: Matrix<T, R, C>) -> Self {
        let mut result = Matrix::<U, R, C>::zeros();
        for i in 0..matrix.rows() {
            for j in 0..matrix.cols() {
                result[(i, j)] = matrix[(i, j)].into();
            }
        }
        result
    }
}

pub trait FromDynMat<T: Element> {
    fn from_dyn_mat(m: DynMatrix<T>) -> Self;
}

impl<T: Element, U: Element> FromDynMat<T> for DynMatrix<U>
    where TheTypes<T, U> : AreNotSame,
          T : Into<U>
{
    fn from_dyn_mat(matrix: DynMatrix<T>) -> Self {
        let mut result = DynMatrix::<U>::zeros(matrix.dims());
        for i in 0..matrix.rows() {
            for j in 0..matrix.cols() {
                result[(i, j)] = matrix[(i, j)].into();
            }
        }
        result
    }
}
