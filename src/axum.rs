use axum::{
    async_trait,
    extract::{FromRequest, Request},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::de::DeserializeOwned;

use crate::{dims::{Cols, Dims, Rows}, dyn_matrix::DynMatrix, element::Element, matrix::Matrix};

impl<T: Element, const R: usize, const C: usize> IntoResponse for &Matrix<T, R, C> {
    fn into_response(self) -> Response {
        let _status = StatusCode::OK;
        let _obj = Json(vec![[1, 2, 3]]);
        todo!();
    }
}

impl<T: Element> IntoResponse for DynMatrix<T> {
    fn into_response(self) -> Response {
        (StatusCode::OK, Json(&self)).into_response()
    }
}

#[async_trait]
impl<T: Element, S> FromRequest<S> for DynMatrix<T>
where
    S: Send + Sync,
    T: DeserializeOwned,
{
    type Rejection = ();

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let Json(matrix) = Json::<DynMatrix<T>>::from_request(req, state)
            .await
            .map_err(|_| ())?;
        Ok(matrix)
    }
}

impl IntoResponse for Dims {
    fn into_response(self) -> Response {
        let Dims(Rows(r), Cols(c)) = self;
        (StatusCode::OK, Json(&(r,c))).into_response()
    }
}