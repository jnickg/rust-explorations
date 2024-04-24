use crate::{dims::Dims, dyn_matrix::DynMatrix, element::Element};
use serde_json::json;
use utoipa::{openapi::{ArrayBuilder, ObjectBuilder, RefOr, Schema, SchemaType}, ToSchema};

impl<'__s, T: Element> ToSchema<'__s> for DynMatrix<T> {
    fn schema() -> (&'__s str, RefOr<Schema>) {
         (
            "Matrix",
            ArrayBuilder::new()
                .items(
                    ArrayBuilder::new()
                        .items(ObjectBuilder::new().schema_type(SchemaType::Number))
                        .min_items(Some(1))
                        .unique_items(false)
                        .build()
                )
                .min_items(Some(1))
                .unique_items(false)
                // Identity camera matrix
                .example(Some(json!([[1,0,0,0],[0,1,0,0],[0,0,1,0],[0,0,0,0]])))
                .build()
                .into(),
        )
    }
}

impl<'__s> ToSchema<'__s> for Dims {
    fn schema() -> (&'__s str, RefOr<Schema>) {
         (
            "Dims",
            ArrayBuilder::new()
                .items(ObjectBuilder::new().schema_type(SchemaType::Number))
                .min_items(Some(2))
                .max_items(Some(2))
                .unique_items(false)
                .example(Some(json!([[1,0],[0,1]])))
                .build()
                .into(),
        )
    }
}
