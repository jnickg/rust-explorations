use serde::{de::Visitor, ser::SerializeSeq, Deserialize, Deserializer, Serialize};

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

pub struct MatrixVisitor<T: Element, const R: usize, const C: usize> {
    marker: std::marker::PhantomData<T>,
}

impl<'de, T: Element, const R: usize, const C: usize> Visitor<'de> for MatrixVisitor<T, R, C>
where
    T: Deserialize<'de>,
{
    type Value = Matrix<T, R, C>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str(&format!(
            "a 2-dimensional matrix of numeric values with dimensions {}x{}",
            R, C
        ))
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        let mut rows = Vec::with_capacity(R);
        while let Some(row) = seq.next_element::<Vec<T>>()? {
            if row.len() != C {
                return Err(serde::de::Error::custom(
                    "invalid number of columns in matrix",
                ));
            }

            rows.push(row);
        }
        if rows.len() != R {
            return Err(serde::de::Error::custom("invalid number of rows in matrix"));
        }
        Ok(Matrix::<T, R, C>::from_vec(&rows))
    }
}

impl<'de, T: Element, const R: usize, const C: usize> Deserialize<'de> for Matrix<T, R, C>
where
    T: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Matrix<T, R, C>, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_seq(MatrixVisitor::<T, R, C> {
            // This is so yucky
            marker: std::marker::PhantomData,
        })
    }
}

// Needed so that we can tell serde we expect our sequence to always contain nested sequences
pub struct DynMatrixVisitor<T: Element> {
    marker: std::marker::PhantomData<T>,
}

impl<'de, T: Element> Visitor<'de> for DynMatrixVisitor<T>
where
    T: Deserialize<'de>,
{
    type Value = DynMatrix<T>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a 2-dimensional matrix of numeric values")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        let mut rows = Vec::new();
        let mut row_len: usize = 0;
        while let Some(row) = seq.next_element::<Vec<T>>()? {
            match row_len {
                0 => row_len = row.len(),
                _ => {
                    if row.len() != row_len {
                        return Err(serde::de::Error::custom("inconsistent row lengths"));
                    }
                }
            }
            rows.push(row);
        }
        Ok(DynMatrix::from_vec(&rows))
    }
}

impl<'de, T: Element> Deserialize<'de> for DynMatrix<T>
where
    T: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<DynMatrix<T>, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_seq(DynMatrixVisitor::<T> {
            // This is so yucky
            marker: std::marker::PhantomData,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::dyn_matrix::DynMatrix;
    use crate::matrix::Matrix;
    use serde_json;

    #[test]
    fn serialize_matrix() {
        let m = Matrix::from_nested(&[[1.0, 2.0], [3.0, 4.0]]);
        let serialized = serde_json::to_string(&m).unwrap();
        assert_eq!(serialized, "[[1.0,2.0],[3.0,4.0]]");
    }

    #[test]
    fn serialize_dyn_matrix() {
        let m = DynMatrix::from_nested(&[[1.0, 2.0], [3.0, 4.0]]);
        let serialized = serde_json::to_string(&m).unwrap();
        assert_eq!(serialized, "[[1.0,2.0],[3.0,4.0]]");
    }

    #[test]
    fn deserialize_matrix() {
        let serialized_mat = "[[1.0,2.0],[3.0,4.0]]";
        let deserialized = serde_json::from_str::<Matrix<f64, 2, 2>>(serialized_mat).unwrap();
        let expected = Matrix::from_nested(&[[1.0, 2.0], [3.0, 4.0]]);
        assert_eq!(deserialized, expected);
    }

    #[test]
    fn deserialize_matrix_fails_when_row_lengths_are_inconsistent() {
        let serialized_mat = "[[1.0,2.0],[3.0,4.0,5.0]]";
        let deserialized = serde_json::from_str::<Matrix<f64, 2, 2>>(serialized_mat);
        assert!(deserialized.is_err());
    }

    #[test]
    fn deserialize_matrix_fails_when_row_len_does_not_match() {
        // Valid matrix, but not a 3x2 matrix
        let serialized_mat = "[[1.0,2.0],[3.0,4.0]]";
        let deserialized = serde_json::from_str::<Matrix<f64, 3, 2>>(serialized_mat);
        assert!(deserialized.is_err());
    }

    #[test]
    fn deserialize_matrix_fails_when_col_len_does_not_match() {
        // Valid matrix, but not a 2x3 matrix
        let serialized_mat = "[[1.0,2.0],[3.0,4.0]]";
        let deserialized = serde_json::from_str::<Matrix<f64, 2, 3>>(serialized_mat);
        assert!(deserialized.is_err());
    }

    #[test]
    fn deserialize_matrix_constructs_f64_from_int_types() {
        let serialized_mat = "[[1,2],[3,4]]";
        let deserialized = serde_json::from_str::<Matrix<f64, 2, 2>>(serialized_mat).unwrap();
        let expected = Matrix::from_nested(&[[1.0, 2.0], [3.0, 4.0]]);
        assert_eq!(deserialized, expected);
    }

    #[test]
    fn deserialize_matrix_constructs_i32_from_int_literals() {
        let serialized_mat = "[[1,2],[3,4]]";
        let deserialized = serde_json::from_str::<Matrix<i32, 2, 2>>(serialized_mat).unwrap();
        let expected = Matrix::from_nested(&[[1, 2], [3, 4]]);
        assert_eq!(deserialized, expected);
    }

    #[test]
    fn deserialize_matrix_fails_to_construct_i32_from_float_literals() {
        let serialized_mat = "[[1.0,2.0],[3.0,4.0]]";
        let deserialized = serde_json::from_str::<Matrix<i32, 2, 2>>(serialized_mat);
        assert!(deserialized.is_err());
    }

    #[test]
    fn deserialize_dyn_matrix() {
        let serialized_mat = "[[1.0,2.0],[3.0,4.0]]";
        let deserialized = serde_json::from_str::<DynMatrix<f64>>(serialized_mat).unwrap();
        let expected = DynMatrix::from_nested(&[[1.0, 2.0], [3.0, 4.0]]);
        assert_eq!(deserialized, expected);
    }

    #[test]
    fn deserialize_dyn_matrix_fails_when_row_lengths_are_inconsistent() {
        let serialized_mat = "[[1.0,2.0],[3.0,4.0,5.0]]";
        let deserialized = serde_json::from_str::<DynMatrix<f64>>(serialized_mat);
        assert!(deserialized.is_err());
    }

    #[test]
    fn deserialize_dyn_matrix_constructs_f64_from_int_types() {
        let serialized_mat = "[[1,2],[3,4]]";
        let deserialized = serde_json::from_str::<DynMatrix<f64>>(serialized_mat).unwrap();
        let expected = DynMatrix::from_nested(&[[1.0, 2.0], [3.0, 4.0]]);
        assert_eq!(deserialized, expected);
    }

    #[test]
    fn deserialize_dyn_matrix_constructs_i32_from_int_literals() {
        let serialized_mat = "[[1,2],[3,4]]";
        let deserialized = serde_json::from_str::<DynMatrix<i32>>(serialized_mat).unwrap();
        let expected = DynMatrix::from_nested(&[[1, 2], [3, 4]]);
        assert_eq!(deserialized, expected);
    }

    #[test]
    fn deserialize_dyn_matrix_fails_to_construct_i32_from_float_literals() {
        let serialized_mat = "[[1.0,2.0],[3.0,4.0]]";
        let deserialized = serde_json::from_str::<DynMatrix<i32>>(serialized_mat);
        assert!(deserialized.is_err());
    }
}
