pub struct Rows(pub usize);
pub struct Cols(pub usize);
pub struct Dims(pub Rows, pub Cols);

impl From<(usize, usize)> for Dims {
    fn from((r, c): (usize, usize)) -> Self {
        Dims(Rows(r), Cols(c))
    }
}

pub trait HasDims {
    fn rows(&self) -> usize;
    fn cols(&self) -> usize;
    fn dims(&self) -> Dims;
}