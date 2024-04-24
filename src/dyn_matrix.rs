use std::fmt::Display;
use std::ops::{Add, AddAssign, Index, IndexMut, Mul, MulAssign, Sub, SubAssign};

use crate::dims::{Cols, Dims, HasDims, Rows};
use crate::element::Element;
use crate::matrix::Matrix;
// use crate::my_traits::{AreNotSame, IsTrue, Multiplied, TheTypes, Values, AreEqual};

/// A matrix of elements of type `T`, with `R` rows and `C` columns.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DynMatrix<T: Element> {
    /// The elements of this matrix
    els: Vec<Vec<T>>,
}

impl<T: Element> DynMatrix<T> {
    /// Create a new matrix with all elements set to zero.
    pub fn zeros<D>(dims: D) -> Self
    where
        D: Into<Dims>,
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
    where
        D: Into<Dims>,
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
    where
        D: Into<Dims>,
    {
        let Dims(Rows(r), Cols(c)) = dims.into();
        let num_els = r * c;
        assert_eq!(data.len(), num_els);
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

    pub fn from_vec(data: &[Vec<T>]) -> Self {
        let mut matrix = Self::zeros((data.len(), data[0].len()));
        for (i, row) in data.iter().enumerate() {
            for (j, el) in row.iter().enumerate() {
                matrix.els[i][j] = *el;
            }
        }
        matrix
    }

    pub fn identity<D>(dims: D) -> Self
    where
        D: Into<Dims>,
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

pub struct DynMatrixIterator<'a, T: Element> {
    matrix: &'a DynMatrix<T>,
    row: usize,
}

impl<'a, T: Element> Iterator for DynMatrixIterator<'a, T> {
    type Item = Vec<T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.row < self.matrix.rows() {
            let result = &self.matrix.els[self.row];
            self.row += 1;
            Some(result.to_vec())
        } else {
            None
        }
    }
}

pub struct DynMatrixIntoIterator<T: Element> {
    matrix: DynMatrix<T>,
    row: usize,
}

impl<T: Element> Iterator for DynMatrixIntoIterator<T> {
    type Item = Vec<T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.row < self.matrix.rows() {
            let result = &self.matrix.els[self.row];
            self.row += 1;
            Some(result.to_vec())
        } else {
            None
        }
    }
}

impl<T: Element> IntoIterator for DynMatrix<T> {
    type Item = Vec<T>;
    type IntoIter = DynMatrixIntoIterator<T>;

    fn into_iter(self) -> Self::IntoIter {
        DynMatrixIntoIterator {
            matrix: self,
            row: 0,
        }
    }
}

impl<'a, T: Element> IntoIterator for &'a DynMatrix<T> {
    type Item = Vec<T>;
    type IntoIter = DynMatrixIterator<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        DynMatrixIterator {
            matrix: self,
            row: 0,
        }
    }
}

impl<T: Element> Index<(usize, usize)> for DynMatrix<T> {
    type Output = T;

    fn index(&self, (x, y): (usize, usize)) -> &Self::Output {
        &self.els[x][y]
    }
}

impl<T: Element> Index<usize> for DynMatrix<T> {
    type Output = [T];

    fn index(&self, x: usize) -> &Self::Output {
        &self.els[x]
    }
}

impl<T: Element> IndexMut<usize> for DynMatrix<T> {
    fn index_mut(&mut self, x: usize) -> &mut Self::Output {
        &mut self.els[x]
    }
}

impl<T: Element> IndexMut<(usize, usize)> for DynMatrix<T> {
    fn index_mut(&mut self, (x, y): (usize, usize)) -> &mut Self::Output {
        &mut self.els[x][y]
    }
}

#[auto_impl_ops::auto_ops]
impl<T: Element> AddAssign<&DynMatrix<T>> for DynMatrix<T>
where
    for<'x> &'x T: Add<Output = T>,
{
    fn add_assign(&mut self, other: &Self) {
        assert_eq!(self.rows(), other.rows());
        assert_eq!(self.cols(), other.cols());
        for i in 0..self.rows() {
            for j in 0..self.cols() {
                self[(i, j)] += other[(i, j)];
            }
        }
    }
}

#[auto_impl_ops::auto_ops]
impl<T: Element> AddAssign<&T> for DynMatrix<T>
where
    for<'x> &'x T: Add<Output = T>,
{
    fn add_assign(&mut self, other: &T) {
        for i in 0..self.rows() {
            for j in 0..self.cols() {
                self[(i, j)] += *other;
            }
        }
    }
}

#[auto_impl_ops::auto_ops]
impl<T: Element> SubAssign<&DynMatrix<T>> for DynMatrix<T>
where
    for<'x> &'x T: Sub<Output = T>,
{
    fn sub_assign(&mut self, other: &Self) {
        assert_eq!(self.rows(), other.rows());
        assert_eq!(self.cols(), other.cols());
        for i in 0..self.rows() {
            for j in 0..self.cols() {
                self[(i, j)] -= other[(i, j)];
            }
        }
    }
}

#[auto_impl_ops::auto_ops]
impl<T: Element> SubAssign<&T> for DynMatrix<T>
where
    for<'x> &'x T: Sub<Output = T>,
{
    fn sub_assign(&mut self, other: &T) {
        for i in 0..self.rows() {
            for j in 0..self.cols() {
                self[(i, j)] -= *other;
            }
        }
    }
}

#[auto_impl_ops::auto_ops]
impl<'a, T: Element> MulAssign<&'a DynMatrix<T>> for DynMatrix<T>
where
    T: Element + Sized + for<'x> MulAssign<&'x T>,
{
    fn mul_assign(&mut self, other: &DynMatrix<T>) {
        assert_eq!(self.cols(), other.rows());
        let mut result = DynMatrix::zeros((self.rows(), other.cols()));
        for i in 0..self.rows() {
            for j in 0..other.cols() {
                for k in 0..self.cols() {
                    result[(i, j)] += self[(i, k)] * other[(k, j)];
                }
            }
        }
        *self = result;
    }
}

#[auto_impl_ops::auto_ops]
impl<'a, T: Element> MulAssign<&'a T> for DynMatrix<T>
where
    T: Element + Sized + for<'x> MulAssign<&'x T>,
{
    fn mul_assign(&mut self, other: &T) {
        for i in 0..self.rows() {
            for j in 0..self.cols() {
                self[(i, j)] *= other;
            }
        }
    }
}

impl<T: Element> From<DynMatrix<T>> for Vec<Vec<T>> {
    fn from(matrix: DynMatrix<T>) -> Self {
        matrix.els
    }
}

impl<T: Element> From<&DynMatrix<T>> for String {
    fn from(matrix: &DynMatrix<T>) -> Self {
        serde_json::to_string(matrix).unwrap()
    }
}

impl<T: Element> Display for DynMatrix<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let json: String = self.into();
        write!(f, "{}", json)
    }
}

impl<T: Element, const R: usize, const C: usize> From<Matrix<T, R, C>> for DynMatrix<T> {
    fn from(matrix: Matrix<T, R, C>) -> Self {
        let mut result = DynMatrix::zeros((R, C));
        for i in 0..R {
            for j in 0..C {
                result[(i, j)] = matrix[(i, j)];
            }
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use crate::from_mat::FromDynMat;

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
        let matrix1 = DynMatrix::<u8>::from_flat(&[1, 2, 3, 4], (2, 2));
        let matrix2 = DynMatrix::<u8>::from_flat(&[5, 6, 7, 8], (2, 2));
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
    fn from_other_element_type() {
        let matrix = DynMatrix::<u8>::from_flat(&[1, 2, 3, 4], (2, 2));
        let result = DynMatrix::<u16>::from_dyn_mat(matrix);
        assert_eq!(result[(0, 0)], 1);
        assert_eq!(result[(0, 1)], 2);
        assert_eq!(result[(1, 0)], 3);
        assert_eq!(result[(1, 1)], 4);
    }
}
