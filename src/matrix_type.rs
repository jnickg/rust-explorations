use std::ops::{Index, IndexMut};

use crate::{dims::{Cols, Dims, HasDims, Rows}, element::Element};

pub trait Indexible<T: Element>: HasDims {
    fn at(&self, d: Dims) -> T;
    fn at_mut(&mut self, d: Dims) -> &mut T;
}

impl<T: Element> Index<(usize, usize)> for dyn Indexible<T> {
    type Output = T;

    fn index(&self, (r, c): (usize, usize)) -> &Self::Output {
        &self.at(Dims(Rows(r), Cols(c)))
    }
}

impl<T: Element> IndexMut<(usize, usize)> for dyn Indexible<T> {
    fn index_mut(&mut self, (r, c): (usize, usize)) -> &mut Self::Output {
        &mut self.at(Dims(Rows(r), Cols(c)))
    }
}

impl<T: Element> Index<Dims> for dyn Indexible<T> {
    type Output = T;

    fn index(&self, d: Dims) -> &Self::Output {
        &self.at(d)
    }
}

impl<T: Element> IndexMut<Dims> for dyn Indexible<T> {
    fn index_mut(&mut self, d: Dims) -> &mut Self::Output {
        &mut self.at_mut(d)
    }
}

pub trait IsMatrix<T: Element> : Indexible<T> {
    fn zeros<D>(dims: D) -> Self where D: Into<Dims>;
    fn zeros_like(m: &Self) -> Self;
    fn ones<D>(dims: D) -> Self where D: Into<Dims>;
    fn ones_like(m: &Self) -> Self;
}

pub struct MatrixType<E: Element, T: IsMatrix<E>>(T);
