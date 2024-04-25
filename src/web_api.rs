use axum::body::Body;
use image::{io::Reader as ImageReader, DynamicImage, ImageFormat};
use std::{collections::HashMap, io::Cursor};

use askama::Template;
use jnickg_imaging::{
    dims::{Dims, HasDims},
    dyn_matrix::DynMatrix,
};
use utoipa::OpenApi;

use crate::*;

macro_rules! debug_print {
    ($($e:expr),+) => {
        {
            #[cfg(debug_assertions)]
            {
                println!($($e),+)
            }
            #[cfg(not(debug_assertions))]
            {}
        }
    };
}

#[derive(OpenApi)]
#[openapi(
    paths(
        get_something,
        put_something,
        delete_something,
        post_something,
        post_something_with_id,
        post_image,
        get_image,
        post_matrix_with_name,
        get_matrix,
        put_matrix,
        delete_matrix,
        post_matrix_add,
        post_matrix_subtract,
        post_matrix_multiply,
        get_matrix_dims
    ),
    components(
        schemas(
            DynMatrix<f64>,
            Dims
        )
    ),
    tags(
        (name = "jnickg_imaging", description = "Toy Image Processing API")
    )
)]
pub struct Documentation;

#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexTemplate<'a> {
    matrices: &'a HashMap<String, DynMatrix<f64>>,
    images: &'a HashMap<String, DynamicImage>,
    stylesheet: &'static str,
}

impl<'a> IndexTemplate<'a> {
    fn new(
        matrices: &'a HashMap<String, DynMatrix<f64>>,
        images: &'a HashMap<String, DynamicImage>,
    ) -> Self {
        Self {
            matrices,
            images,
            stylesheet: "/style.css",
        }
    }
}

pub async fn get_index(State(app_state): AppState) -> Response {
    let app = &mut app_state.read().await;
    (
        StatusCode::OK,
        IndexTemplate::new(&app.matrices, &app.images),
    )
        .into_response()
}

pub async fn get_hello() -> Response {
    (StatusCode::OK, "Hello, World!").into_response()
}

#[utoipa::path(
    get,
    path = "/api/v1/something/{id}",
    responses(
        (status = StatusCode::OK, description = "Shows something with that ID", body = ()),
        (status = StatusCode::NOT_FOUND, description = "No such something", body = ())
    )
)]
pub async fn get_something(State(app_state): AppState, Path(id): Path<u32>) -> Response {
    let app = &mut app_state.read().await;
    match app.somethings.get(&id) {
        Some(_) => (
            StatusCode::OK,
            format!("Getting something with id {}\n", id),
        )
            .into_response(),
        None => (
            StatusCode::NOT_FOUND,
            format!("Something with id {} not found.\n", id),
        )
            .into_response(),
    }
}

#[utoipa::path(
    put,
    path = "/api/v1/something/{id}",
    responses(
        (status = StatusCode::OK, description = "Updated the given something", body = ()),
        (status = StatusCode::CREATED, description = "Created something with the given ID", body = ())
    )
)]
pub async fn put_something(State(app_state): AppState, Path(id): Path<u32>) -> Response {
    let app = &mut app_state.write().await;
    if app.somethings.contains(&id) {
        return (
            StatusCode::OK,
            format!("Updating something with id {}\n", id),
        )
            .into_response();
    }
    let somethings = &mut app.somethings;
    somethings.insert(id);
    (
        StatusCode::CREATED,
        format!("Creating something with id {}\n", id),
    )
        .into_response()
}

#[utoipa::path(
    delete,
    path = "/api/v1/something/{id}",
    responses(
        (status = StatusCode::OK, description = "Deleted the given something", body = ()),
        (status = StatusCode::NOT_FOUND, description = "No something with the given ID to delete", body = ())
    )
)]
pub async fn delete_something(State(app_state): AppState, Path(id): Path<u32>) -> Response {
    let app = &mut app_state.write().await;
    if app.somethings.contains(&id) {
        app.somethings.remove(&id);
        return (
            StatusCode::OK,
            format!("Deleting something with id {}\n", id),
        )
            .into_response();
    }
    (
        StatusCode::NOT_FOUND,
        format!("Something with id {} not found.\n", id),
    )
        .into_response()
}

#[utoipa::path(
    post,
    path = "/api/v1/something",
    responses(
        (status = StatusCode::CREATED, description = "Created new something with the returned ID", body = str)
    )
)]
pub async fn post_something(State(app_state): AppState) -> Response {
    let app = &mut app_state.write().await;
    // Probably wiser to use an incrementing counter that skips past any IDs manually added
    // but this is a good excuse to exercise using a loop to return a value
    let mut rng = rand::thread_rng();
    let id = loop {
        let random_id = rng.gen::<u32>();
        if app.somethings.contains(&random_id) {
            continue;
        }
        break random_id;
    };
    app.somethings.insert(id);
    (
        StatusCode::CREATED,
        format!("Posting something with id {}\n", id),
    )
        .into_response()
}

#[utoipa::path(
    post,
    path = "/api/v1/something/{id}",
    responses(
        (status = StatusCode::CREATED, description = "Created new something with the returned ID", body = str),
        (status = StatusCode::CONFLICT, description = "Something with that ID already exists", body = ())
    )
)]
pub async fn post_something_with_id(State(app_state): AppState, Path(id): Path<u32>) -> Response {
    let app = &mut app_state.write().await;
    let inserted = app.somethings.insert(id);
    match inserted {
        true => (
            StatusCode::CREATED,
            format!("Posting something with id {}\n", id),
        )
            .into_response(),
        false => (
            StatusCode::CONFLICT,
            format!("Something with id {} already exists.\n", id),
        )
            .into_response(),
    }
}

#[utoipa::path(
    post,
    path = "/api/v1/api/v1/matrix/{name}",
    request_body(
        content = DynMatrix<f64>,
        content_type = "application/json"
    ),
    responses(
        (status = StatusCode::CREATED, description = "Added matrix with the given name", body = str),
        (status = StatusCode::INTERNAL_SERVER_ERROR, description = "Failed parse matrix from request body", body = ()),
        (status = StatusCode::CONFLICT, description = "Cannot POST new matrix with existing name. If this is intentional, use PUT", body = ())
    )
)]
pub async fn post_matrix_with_name(
    State(app_state): AppState,
    Path(name): Path<String>,
    request: Request,
) -> Response {
    let mat_from_req = DynMatrix::<f64>::from_request(request, &app_state).await;
    match mat_from_req {
        Ok(new_mat) => {
            let app = &mut app_state.write().await;
            match app.matrices.contains_key(&name) {
                true => (
                    StatusCode::CONFLICT,
                    "Cannot POST new matrix with existing name. If this is intentional, use PUT",
                )
                    .into_response(),
                false => {
                    app.matrices.insert(name.clone(), new_mat.clone());
                    (StatusCode::CREATED, format!("Matrix {} received.\n", name)).into_response()
                }
            }
        }
        Err(_) => {
            debug_print!("Failed to deserialize matrix name from string: {}", name);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to read matrix from request.\n",
            )
                .into_response()
        }
    }
}

#[utoipa::path(
    get,
    path = "/api/v1/matrix/{name}",
    responses(
        (status = StatusCode::OK, description = "Returns matrix with the given name", body = MatrixSchema<f64>),
        (status = StatusCode::NOT_FOUND, description = "Unable to find matrix withthe given name", body = ()),
    )
)]
pub async fn get_matrix(State(app_state): AppState, Path(name): Path<String>) -> Response {
    let app = &mut app_state.read().await;
    match app.matrices.get(&name) {
        Some(mat) => (StatusCode::OK, mat.clone()).into_response(),
        None => (
            StatusCode::NOT_FOUND,
            format!("Matrix {} not found.\n", name),
        )
            .into_response(),
    }
}

#[utoipa::path(
    get,
    path = "/api/v1/matrix/{name}/dims",
    responses(
        (status = StatusCode::OK, description = "Returns dimensions of the matrix with the given name", body = Dims),
        (status = StatusCode::NOT_FOUND, description = "Unable to find matrix withthe given name", body = ()),
    )
)]
pub async fn get_matrix_dims(State(app_state): AppState, Path(name): Path<String>) -> Response {
    let app = &mut app_state.read().await;
    match app.matrices.get(&name) {
        Some(mat) => {
            let dims = mat.dims();
            (StatusCode::OK, dims).into_response()
        }
        None => (
            StatusCode::NOT_FOUND,
            format!("Matrix {} not found.\n", name),
        )
            .into_response(),
    }
}

#[utoipa::path(
    put,
    path = "/api/v1/matrix/{name}",
    request_body(
        content = DynMatrix<f64>,
        content_type = "application/json"
    ),
    responses(
        (status = StatusCode::OK, description = "Updated matrix with the given name", body = DynMatrix<f64>),
        (status = StatusCode::CREATED, description = "Created matrix with the given name", body = DynMatrix<f64>),
        (status = StatusCode::NOT_FOUND, description = "Unable to find matrix withthe given name", body = ()),
    )
)]
pub async fn put_matrix(
    State(app_state): AppState,
    Path(name): Path<String>,
    request: Request,
) -> Response {
    let mat_from_req = DynMatrix::<f64>::from_request(request, &app_state).await;
    match mat_from_req {
        Ok(new_mat) => {
            let app = &mut app_state.write().await;
            match app.matrices.contains_key(&name) {
                true => {
                    app.matrices.insert(name.clone(), new_mat.clone());
                    (StatusCode::OK, new_mat).into_response()
                }
                false => {
                    app.matrices.insert(name.clone(), new_mat.clone());
                    (StatusCode::CREATED, new_mat).into_response()
                }
            }
        }
        Err(_) => {
            debug_print!("Failed to deserialize matrix name from string: {}", name);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to read matrix from request.\n",
            )
                .into_response()
        }
    }
}

#[utoipa::path(
    delete,
    path = "/api/v1/matrix/{name}",
    responses(
        (status = StatusCode::OK, description = "Deleted matrix with the given name and returned it", body = DynMatrix<f64>),
        (status = StatusCode::NOT_FOUND, description = "Unable to find matrix withthe given name", body = ()),
    )
)]
pub async fn delete_matrix(State(app_state): AppState, Path(name): Path<String>) -> Response {
    let app = &mut app_state.write().await;
    match app.matrices.remove(&name) {
        Some(mat) => (StatusCode::OK, mat).into_response(),
        None => (
            StatusCode::NOT_FOUND,
            format!("Matrix {} not found.\n", name),
        )
            .into_response(),
    }
}

#[utoipa::path(
    post,
    path = "/api/v1/matrix/multiply/{name1}/{name2}",
    responses(
        (status = StatusCode::OK, description = "Computation completed and result is returned in JSON format", body = DynMatrix<f64>),
        (status = StatusCode::INTERNAL_SERVER_ERROR, description = "Invalid matrix multiplication", body = ()),
    )
)]
pub async fn post_matrix_multiply(
    State(app_state): AppState,
    Path((name1, name2)): Path<(String, String)>,
) -> Response {
    let app = &mut app_state.write().await;
    let mat1 = app.matrices.get(&name1).unwrap();
    let mat2 = app.matrices.get(&name2).unwrap();
    let result = mat1 * mat2;
    // Return result in body
    (StatusCode::OK, result.clone()).into_response()
}

#[utoipa::path(
    post,
    path = "/api/v1/matrix/add/{name1}/{name2}",
    responses(
        (status = StatusCode::OK, description = "Computation completed and result is returned in JSON format", body = DynMatrix<f64>),
        (status = StatusCode::INTERNAL_SERVER_ERROR, description = "Invalid matrix addition (check matrix dimensions)", body = ()),
    )
)]
pub async fn post_matrix_add(
    State(app_state): AppState,
    Path((name1, name2)): Path<(String, String)>,
) -> Response {
    let app = &mut app_state.write().await;
    let mat1 = app.matrices.get(&name1).unwrap();
    let mat2 = app.matrices.get(&name2).unwrap();
    let result = mat1 + mat2;
    // Return result in body
    (StatusCode::OK, result.clone()).into_response()
}

#[utoipa::path(
    post,
    path = "/api/v1/matrix/subtract/{name1}/{name2}",
    responses(
        (status = StatusCode::OK, description = "Computation completed and result is returned in JSON format", body = DynMatrix<f64>),
        (status = StatusCode::INTERNAL_SERVER_ERROR, description = "Invalid matrix subtraction (check matrix dimensions)", body = ()),
    )
)]
pub async fn post_matrix_subtract(
    State(app_state): AppState,
    Path((name1, name2)): Path<(String, String)>,
) -> Response {
    let app = &mut app_state.write().await;
    let mat1 = app.matrices.get(&name1).unwrap();
    let mat2 = app.matrices.get(&name2).unwrap();
    let result = mat1 - mat2;
    // Return result in body
    (StatusCode::OK, result.clone()).into_response()
}

#[utoipa::path(
    post,
    path = "/api/v1/image",
    request_body(
        content = Bytes,
    ),
    responses(
        (status = StatusCode::CREATED, description = "Added the image with the returned ID", body = ()),
        (status = StatusCode::INTERNAL_SERVER_ERROR, description = "Failed to read image from request", body = ()),
        (status = StatusCode::BAD_REQUEST, description = "Unable to handle request. Please pass an image body and specify content type.", body = ()),
        (status = StatusCode::NOT_ACCEPTABLE, description = "Unsupported image format.", body = ())
    )
)]
pub async fn post_image(State(app_state): AppState, request: Request) -> Response {
    let content_disposition_hdr = request.headers().get("Content-Disposition");
    let image_name: String = if content_disposition_hdr.is_some() {
        let content_disposition = content_disposition_hdr.unwrap().to_str().unwrap();
        let parts: Vec<&str> = content_disposition.split(';').collect();
        let name_part = parts.iter().find(|&p| p.starts_with("filename"));
        if name_part.is_none() {
            return (
                StatusCode::BAD_REQUEST,
                "Unable to handle request. Please pass an image body and specify content type.\n",
            )
                .into_response();
        }
        let name_part = name_part.unwrap();
        let name_parts: Vec<&str> = name_part.split('=').collect();
        let name = name_parts.get(1).unwrap();
        name.to_string()
    } else {
        let app = &mut app_state.write().await;
        let new_name = format!("image_{}", app.image_counter);
        app.image_counter += 1;
        new_name
    };
    debug_print!("Attempting to add new image with name {}", image_name);

    let content_type_hdr = request.headers().get("Content-Type");
    if content_type_hdr.is_none() {
        return (
            StatusCode::BAD_REQUEST,
            "Unable to handle request. Please pass an image body and specify content type.\n",
        )
            .into_response();
    }

    let mime_type = content_type_hdr.unwrap().to_str().unwrap();
    let format: ImageFormat = match ImageFormat::from_mime_type(mime_type) {
        Some(fmt) => fmt,
        None => {
            return (
                StatusCode::NOT_ACCEPTABLE,
                format!("The given MIME Type \"{}\" is not supported", mime_type),
            )
                .into_response()
        }
    };
    debug_print!("Detected MIME Type: \"{}\"", mime_type);

    let bytes = match Bytes::from_request(request, &app_state).await {
        Ok(b) => b.to_vec(),
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to read image data from request body.\n",
            )
                .into_response()
        }
    };
    debug_print!("Extracted image data with byte length: {}", bytes.len());

    #[allow(clippy::let_and_return)]
    let result: Response = match ImageReader::with_format(Cursor::new(bytes), format).decode() {
        Ok(new_image) => {
            let app = &mut app_state.write().await;
            app.images.insert(image_name.clone(), new_image);
            (
                StatusCode::CREATED,
                format!("Image added with name {}.", image_name),
            )
                .into_response()
        }
        Err(_e) => {
            debug_print!("Error: {}", _e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!(
                    "Failed to decode image of type {} from request body",
                    format.extensions_str()[0]
                ),
            )
                .into_response();
        }
    };

    result
}

#[utoipa::path(
    get,
    path = "/api/v1/image/{name}",
    request_body(
        content = Bytes,
    ),
    responses(
        (status = StatusCode::OK, description = "Returned the image of the given name", body = Vec<u8>),
        (status = StatusCode::NOT_FOUND, description = "No such image available", body = ()),
    )
)]
pub async fn get_image(State(app_state): AppState, Path(name): Path<String>, request: Request) -> Response {
    let app = &mut app_state.read().await;
    match app.images.get(&name) {
        Some(image) => {
            // We try to adhere to user request, but default to PNG if anything goes wrong
            let dest_format = match request.headers().get("Accept") {
                Some(accept_hdr) => {
                    let accept = accept_hdr.to_str().unwrap();
                    match ImageFormat::from_mime_type(accept) {
                        Some(fmt) => fmt,
                        None => ImageFormat::Png,
                    }
                }
                None => ImageFormat::Png,
            };
            let mut data = Vec::new();
            let mut cursor = Cursor::new(&mut data);
            match image.write_to(&mut cursor, dest_format) {
                Ok(_) => Response::builder()
                    .status(StatusCode::OK)
                    .header("Content-Type", dest_format.to_mime_type())
                    .body(Body::from(data))
                    .unwrap(),
                Err(_) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Failed to write image data to response body.\n",
                )
                    .into_response(),
            }
        }
        None => (
            StatusCode::NOT_FOUND,
            format!("Image {} not found.\n", name),
        )
            .into_response(),
    }
}
