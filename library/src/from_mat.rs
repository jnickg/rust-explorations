use crate::dims::HasDims;
use crate::dyn_matrix::DynMatrix;
use crate::element::Element;
use crate::matrix::Matrix;

pub trait FromMat<T: Element, const R: usize, const C: usize> {
    fn from_mat(m: Matrix<T, R, C>) -> Self;
}

impl<T: Element, U: Element, const R: usize, const C: usize> FromMat<T, R, C> for DynMatrix<U>
where
    T: Into<U>,
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

impl<T: Element, U: Element, const R: usize, const C: usize> FromMat<T, R, C> for Matrix<U, R, C>
where
    T: Into<U>,
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
where
    T: Into<U>,
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
