use axum::{
    async_trait,
    extract::{FromRequest, Request},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::de::DeserializeOwned;

use jnickg_imaging::{
    dims::{Cols, Dims, Rows},
    dyn_matrix::DynMatrix,
    element::Element,
};

use crate::wrappers::*;

impl<T: Element, const R: usize, const C: usize> IntoResponse for WrappedMatrix<T, R, C> {
    fn into_response(self) -> Response {
        let _status = StatusCode::OK;
        let _obj = Json(vec![[1, 2, 3]]);
        todo!();
    }
}

impl<T: Element> IntoResponse for WrappedDynMatrix<T> {
    fn into_response(self) -> Response {
        let Self(mat) = self;
        (StatusCode::OK, Json(&mat)).into_response()
    }
}

#[async_trait]
impl<T: Element, S> FromRequest<S> for WrappedDynMatrix<T>
where
    S: Send + Sync,
    T: DeserializeOwned,
{
    type Rejection = ();

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let Json(matrix) = Json::<DynMatrix<T>>::from_request(req, state)
            .await
            .map_err(|_| ())?;
        Ok(Self(matrix))
    }
}

impl IntoResponse for WrappedDims {
    fn into_response(self) -> Response {
        let WrappedDims(Dims(Rows(r), Cols(c))) = self;
        (StatusCode::OK, Json(&(r, c))).into_response()
    }
}
