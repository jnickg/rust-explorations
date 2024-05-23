use jnickg_imaging::{
    dims::Dims,
    dyn_matrix::DynMatrix,
    element::Element,
    matrix::Matrix,
};

#[allow(dead_code)]
pub(crate) struct WrappedMatrix<T: Element, const R: usize, const C: usize>(pub Matrix<T, R, C>);
pub(crate) struct WrappedDynMatrix<T: Element>(pub DynMatrix<T>);
pub(crate) struct WrappedDims(pub Dims);
