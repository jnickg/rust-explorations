use serde::{ser::SerializeSeq, Serialize};

use crate::{dims::HasDims, dyn_matrix::DynMatrix, element::Element, matrix::Matrix};

struct DataArr<'a, T: Element, const SIZE: usize>(&'a [T; SIZE]);

impl<'a, T: Element, const SIZE: usize> Serialize for DataArr<'a, T, SIZE> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(SIZE))?;
        for row in self.0.iter() {
            seq.serialize_element(row)?;
        }
        seq.end()
    }
}

impl<T: Element, const R: usize, const C: usize> Serialize for Matrix<T, R, C> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(R))?;
        for row in self {
            seq.serialize_element(&DataArr(&row))?;
        }
        seq.end()
    }
}

struct DataVec<'a, T: Element>(&'a Vec<T>);

impl<'a, T: Element> Serialize for DataVec<'a, T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(self.0.len()))?;
        for row in self.0.iter() {
            seq.serialize_element(row)?;
        }
        seq.end()
    }
}

impl<T: Element> Serialize for DynMatrix<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(self.rows()))?;
        for row in self {
            seq.serialize_element(&DataVec(&row))?;
        }
        seq.end()
    }
}

// TODO implement Deserialize for DynMatrix.

#[cfg(test)]
mod tests {
    use crate::dyn_matrix::DynMatrix;
    use crate::matrix::Matrix;
    use serde_json;

    #[test]
    fn test_serialize_matrix() {
        let m = Matrix::from_nested(&[[1.0, 2.0], [3.0, 4.0]]);
        let serialized = serde_json::to_string(&m).unwrap();
        assert_eq!(serialized, "[[1.0,2.0],[3.0,4.0]]");
    }

    #[test]
    fn test_serialize_dyn_matrix() {
        let m = DynMatrix::from_nested(&[[1.0, 2.0], [3.0, 4.0]]);
        let serialized = serde_json::to_string(&m).unwrap();
        assert_eq!(serialized, "[[1.0,2.0],[3.0,4.0]]");
    }
}
