use crate::{element::Element, matrix::Matrix};
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};

impl<T: Element, const R: usize, const C: usize> IntoResponse for &Matrix<T, R, C> {
    fn into_response(self) -> Response {
        let _status = StatusCode::OK;
        let _obj = Json(vec![[1, 2, 3]]);
        todo!();
    }
}
