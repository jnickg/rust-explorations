use ::axum::body::Body;
use futures_util::{io::AsyncWriteExt, AsyncReadExt, StreamExt};
use image::ImageFormat;
use mongodb::{
    bson::{doc, Document},
    Collection,
};
use std::{collections::HashMap, io::Cursor};

use ::utoipa::OpenApi;
use askama::Template;
use jnickg_imaging::{
    dims::HasDims,
    dyn_matrix::DynMatrix,
    ipr::{self, HasImageProcessingRoutines},
};

use crate::wrappers::*;
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
            WrappedDynMatrix<f64>,
            WrappedDims
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
    images: &'a Vec<String>,
    stylesheet: &'static str,
}

impl<'a> IndexTemplate<'a> {
    fn new(matrices: &'a HashMap<String, DynMatrix<f64>>, images: &'a Vec<String>) -> Self {
        Self {
            matrices,
            images,
            stylesheet: "/style.css",
        }
    }
}

pub async fn get_api_index(State(app_state): AppState) -> Response {
    let app = &mut app_state.read().await;

    // Get handle to gridfs
    let db = app.db.as_ref().unwrap();
    let mut images = Vec::<String>::new();
    let images_coll: Collection<Document> = db.collection("images");
    let mut cursor = match images_coll.find(None, None).await {
        Ok(c) => c,
        Err(_e) => {
            debug_print!("Error: {}", _e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to query image database.\n",
            )
                .into_response();
        }
    };

    // Extract `name`, `image` and `mime_type` fields from each document in images collection.
    // Then, get the GridFS file from the ObjectId defined by `image` and decode it using the
    // ImageFormat associated with the `mime_type`, and push the DynamicImage to `images` using
    // `name` as the key
    while let Some(doc) = cursor.next().await {
        match doc {
            Ok(d) => {
                let name = d.get("name").unwrap().as_str().unwrap();

                images.push(name.to_string());
            }
            Err(_e) => {
                debug_print!("Error: {}", _e);
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Failed to read image document.\n",
                )
                    .into_response();
            }
        }
    }

    (StatusCode::OK, IndexTemplate::new(&app.matrices, &images)).into_response()
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
    let mat_from_req = WrappedDynMatrix::<f64>::from_request(request, &app_state).await;
    match mat_from_req {
        Ok(WrappedDynMatrix(new_mat)) => {
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
        Some(mat) => (StatusCode::OK, WrappedDynMatrix(mat.clone())).into_response(),
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
            (StatusCode::OK, WrappedDims(dims)).into_response()
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
    let mat_from_req = WrappedDynMatrix::<f64>::from_request(request, &app_state).await;
    match mat_from_req {
        Ok(WrappedDynMatrix(new_mat)) => {
            let app = &mut app_state.write().await;
            match app.matrices.contains_key(&name) {
                true => {
                    app.matrices.insert(name.clone(), new_mat.clone());
                    (StatusCode::OK, WrappedDynMatrix(new_mat)).into_response()
                }
                false => {
                    app.matrices.insert(name.clone(), new_mat.clone());
                    (StatusCode::CREATED, WrappedDynMatrix(new_mat)).into_response()
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
        Some(mat) => (StatusCode::OK, WrappedDynMatrix(mat)).into_response(),
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
    (StatusCode::OK, WrappedDynMatrix(result.clone())).into_response()
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
    (StatusCode::OK, WrappedDynMatrix(result.clone())).into_response()
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
    (StatusCode::OK, WrappedDynMatrix(result.clone())).into_response()
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

    let app = &mut app_state.write().await;

    if app.db.is_none() {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to acquire handle to image database.\n",
        )
            .into_response();
    }
    let db = app.db.as_ref().unwrap();

    let bucket = db.gridfs_bucket(None);
    let mut upload_stream = bucket.open_upload_stream(image_name.clone(), None);
    let upload_result = upload_stream.write_all(&bytes).await;
    match upload_result {
        Ok(_) => (),
        Err(_e) => {
            debug_print!("Error: {}", _e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to upload image to database.\n",
            )
                .into_response();
        }
    }
    let image_id = upload_stream.id().clone();

    // Now that we have a handle to the uploaded ID and created a document, close out the
    // upload to latch it.
    match upload_stream.close().await {
        Ok(_) => (),
        Err(_e) => {
            debug_print!("Error: {}", _e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to close upload stream for image.\n",
            )
                .into_response();
        }
    }

    let doc = doc! {
        "name": image_name.clone(),
        "image": image_id,
        "mime_type": format.to_mime_type(),
    };

    match db.collection("images").insert_one(doc, None).await {
        Ok(_) => (),
        Err(_e) => {
            debug_print!("Error: {}", _e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to insert image into database.\n",
            )
                .into_response();
        }
    }

    (
        StatusCode::CREATED,
        format!("Image added with name {}.", image_name),
    )
        .into_response()
}

#[utoipa::path(
    get,
    path = "/api/v1/images",
    responses(
        (status = StatusCode::OK, description = "Returned a JSON list of image documents", body = Json),
    )
)]
pub async fn get_images(State(app_state): AppState) -> Response {
    let app = &mut app_state.read().await;
    if app.db.is_none() {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to acquire handle to image database.\n",
        )
            .into_response();
    }
    let db = app.db.as_ref().unwrap();
    let images: Collection<Document> = db.collection("images");
    let mut found = match images.find(None, None).await {
        Ok(cursor) => cursor,
        Err(_e) => {
            debug_print!("Error: {}", _e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to query image database.\n",
            )
                .into_response();
        }
    };

    let mut image_docs = Vec::new();
    while let Some(doc) = found.next().await {
        match doc {
            Ok(d) => {
                image_docs.push(d);
            }
            Err(_e) => {
                debug_print!("Error: {}", _e);
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Failed to read image document.\n",
                )
                    .into_response();
            }
        }
    }

    let json = match serde_json::to_string(&image_docs) {
        Ok(j) => j,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to serialize pyramid document.\n",
            )
                .into_response();
        }
    };

    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .body(Body::from(json.to_string()))
        .unwrap()
}

pub async fn get_image_from_collection(
    State(app_state): AppState,
    Path(name): Path<String>,
    request: Request,
    collection_name: &str,
) -> Response {
    // If name has an extension, try to discern the desired format from it. But drop the extension
    // for the purpose of image lookup. We try to adhere to user request, but default to PNG if
    // anything goes wrong
    let ext_str = name.split('.').last().unwrap_or("png");
    let default_format = ImageFormat::from_extension(ext_str).unwrap_or(ImageFormat::Png);

    let name_without_ext = name.split('.').next().unwrap_or(name.as_str());
    let app = &mut app_state.read().await;
    if app.db.is_none() {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to acquire handle to image database.\n",
        )
            .into_response();
    }
    let db = app.db.as_ref().unwrap();
    let images: Collection<Document> = db.collection(collection_name);
    let mut found = match images
        .find(
            doc! {
                "name": name_without_ext
            },
            None,
        )
        .await
    {
        Ok(cursor) => cursor,
        Err(_e) => {
            debug_print!("Error: {}", _e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to query image database.\n",
            )
                .into_response();
        }
    };

    // This is jank because there's no good way to count results before iterating through them.
    let image_doc = match found.next().await {
        Some(doc) => match doc {
            Ok(d) => d,
            Err(_e) => {
                debug_print!("Error: {}", _e);
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Failed to read image document.\n",
                )
                    .into_response();
            }
        },
        None => {
            return (
                StatusCode::NOT_FOUND,
                format!("Image {} not found.\n", name),
            )
                .into_response();
        }
    };

    let image_id = image_doc.get("image");
    if image_id.is_none() {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to find image id in database.\n",
        )
            .into_response();
    }
    let image_id = image_id.unwrap();

    let mime_type = match image_doc.get("mime_type") {
        Some(m) => m.as_str().unwrap(),
        None => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to find image MIME type in database.\n",
            )
                .into_response();
        }
    };

    let bucket = db.gridfs_bucket(None);
    let mut image_bytes = Vec::new();
    let mut download_stream = match bucket.open_download_stream(image_id.clone()).await {
        Ok(s) => s,
        Err(_e) => {
            debug_print!("Error: {}", _e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to open download stream for image.\n",
            )
                .into_response();
        }
    };

    match download_stream.read_to_end(&mut image_bytes).await {
        Ok(_) => (),
        Err(_e) => {
            debug_print!("Error: {}", _e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to read image data from database.\n",
            )
                .into_response();
        }
    };

    // If a header is specified, prefer to honor that over what might be in the request URL
    let dest_format = match request.headers().get("Accept") {
        Some(accept_hdr) => {
            let accept = accept_hdr.to_str().unwrap();
            match ImageFormat::from_mime_type(accept) {
                Some(fmt) => fmt,
                None => default_format,
            }
        }
        None => default_format,
    };

    let is_brotli: bool = image_doc.get_bool("brotli").unwrap_or(false);

    image_bytes = if dest_format == default_format {
        image_bytes
    } else {
        let data_to_re_encode = if is_brotli {
            let mut decompressed = Vec::new();
            match brotli::BrotliDecompress(
                &mut Cursor::new(image_bytes),
                &mut Cursor::new(&mut decompressed),
            ) {
                Ok(_) => (),
                Err(_) => {
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "Failed to decompress image data.\n",
                    )
                        .into_response();
                }
            }
            decompressed
        } else {
            image_bytes
        };

        let image = match image::load_from_memory_with_format(
            &data_to_re_encode,
            ImageFormat::from_mime_type(mime_type).unwrap(),
        ) {
            Ok(img) => img,
            Err(_e) => {
                debug_print!("Error: {}", _e);
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Failed to load image from memory.\n",
                )
                    .into_response();
            }
        };

        let mut re_encoded_data = Vec::new();
        let mut cursor = Cursor::new(&mut re_encoded_data);
        match image.write_to(&mut cursor, dest_format) {
            Ok(_) => re_encoded_data,
            Err(_) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Failed to write image data to response body.\n",
                )
                    .into_response()
            }
        }
    };

    let mut builder = Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", dest_format.to_mime_type());
    if is_brotli {
        builder = builder.header("Content-Encoding", "br");
    }

    builder.body(Body::from(image_bytes)).unwrap()
}

pub async fn put_image_in_collection(
    State(app_state): AppState,
    Path(image_name): Path<String>,
    request: Request,
    collection_name: &str,
) -> Response {
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

    let app = &mut app_state.write().await;

    if app.db.is_none() {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to acquire handle to image database.\n",
        )
            .into_response();
    }
    let db = app.db.as_ref().unwrap();

    // Check if there is an existing document in the image collection with the given name. If there
    // is, get the `image` ObjectId for the GridFS file. Delete both the document and the GridFS file

    let image_collection: Collection<Document> = db.collection(collection_name);
    let existing_image = match image_collection
        .find_one(
            doc! {
                "name": image_name.clone()
            },
            None,
        )
        .await
    {
        Ok(d) => d,
        Err(_e) => {
            debug_print!("Error: {}", _e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to query image database.\n",
            )
                .into_response();
        }
    };

    let mut deleted_old: bool = false;
    if existing_image.is_some() {
        deleted_old = true;
        let existing_image = existing_image.unwrap();
        let image_id = existing_image.get("image").unwrap();
        let bucket = db.gridfs_bucket(None);
        match bucket.delete(image_id.clone()).await {
            Ok(_) => (),
            Err(_e) => {
                debug_print!("Error: {}", _e);
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Failed to delete image from database.\n",
                )
                    .into_response();
            }
        }
        match image_collection
            .delete_one(
                doc! {
                    "name": image_name.clone()
                },
                None,
            )
            .await
        {
            Ok(_) => (),
            Err(_e) => {
                debug_print!("Error: {}", _e);
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Failed to delete image document from database.\n",
                )
                    .into_response();
            }
        }
    }

    // Now, write the image to GridFS and get the ID of the new upload
    let bucket = db.gridfs_bucket(None);
    let mut upload_stream = bucket.open_upload_stream(image_name.clone(), None);
    let upload_result = upload_stream.write_all(&bytes).await;
    match upload_result {
        Ok(_) => (),
        Err(_e) => {
            debug_print!("Error: {}", _e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to upload image to database.\n",
            )
                .into_response();
        }
    }

    let image_id = upload_stream.id();
    let images = db.collection(collection_name);
    let doc = doc! {
        "name": image_name.clone(),
        "image": image_id,
        "mime_type": format.to_mime_type(),
    };

    // Now that we have a handle to the uploaded ID and created a document, close out the
    // upload to latch it.
    match upload_stream.close().await {
        Ok(_) => (),
        Err(_e) => {
            debug_print!("Error: {}", _e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to close upload stream for image.\n",
            )
                .into_response();
        }
    }

    match images.insert_one(doc, None).await {
        Ok(_) => (),
        Err(_e) => {
            debug_print!("Error: {}", _e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to insert image into database.\n",
            )
                .into_response();
        }
    }

    match deleted_old {
        true => (
            StatusCode::OK,
            format!("Image {} updated.\n", image_name.clone()),
        )
            .into_response(),
        false => (
            StatusCode::CREATED,
            format!("Image added with name {}.", image_name.clone()),
        )
            .into_response(),
    }
}

pub async fn delete_image_from_collection(
    State(app_state): AppState,
    Path(image_name): Path<String>,
    collection_name: &str,
) -> Response {
    let app = &mut app_state.write().await;
    if app.db.is_none() {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to acquire handle to image database.\n",
        )
            .into_response();
    }
    let db = app.db.as_ref().unwrap();
    let images: Collection<Document> = db.collection(collection_name);
    let existing_image = match images
        .find_one(
            doc! {
                "name": image_name.clone()
            },
            None,
        )
        .await
    {
        Ok(d) => d,
        Err(_e) => {
            debug_print!("Error: {}", _e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to query image database.\n",
            )
                .into_response();
        }
    };

    if existing_image.is_none() {
        return (
            StatusCode::NOT_FOUND,
            format!("Image {} not found.\n", image_name),
        )
            .into_response();
    }

    let existing_image = existing_image.unwrap();
    let image_id = existing_image.get("image").unwrap();
    let bucket = db.gridfs_bucket(None);
    match bucket.delete(image_id.clone()).await {
        Ok(_) => (),
        Err(_e) => {
            debug_print!("Error: {}", _e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to delete image from database.\n",
            )
                .into_response();
        }
    }
    match images
        .delete_one(
            doc! {
                "name": image_name.clone()
            },
            None,
        )
        .await
    {
        Ok(_) => (),
        Err(_e) => {
            debug_print!("Error: {}", _e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to delete image document from database.\n",
            )
                .into_response();
        }
    }

    (StatusCode::OK, format!("Image {} deleted.\n", image_name)).into_response()
}

#[utoipa::path(
    post,
    path = "/api/v1/pyramid",
    request_body(
        content = Bytes,
    ),
    responses(
        (status = StatusCode::CREATED, description = "Added the image with the returned ID", body = Json),
        (status = StatusCode::INTERNAL_SERVER_ERROR, description = "Failed to read image from request", body = ()),
        (status = StatusCode::BAD_REQUEST, description = "Unable to handle request. Please pass an image body and specify content type.", body = ()),
        (status = StatusCode::NOT_ACCEPTABLE, description = "Unsupported image format.", body = ())
    )
)]
pub async fn post_pyramid(State(app_state): AppState, request: Request) -> Response {
    let content_disposition_hdr = request.headers().get("Content-Disposition");
    let image_name: String = if content_disposition_hdr.is_some() {
        let content_disposition = content_disposition_hdr.unwrap().to_str().unwrap();
        dbg!(&content_disposition);
        let parts: Vec<&str> = content_disposition.split(';').map(|p| p.trim()).collect();
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
            "Unable to handle request. Please pass an image body and specify a Content-Type.\n",
        )
            .into_response();
    }

    let given_mime_type = content_type_hdr.unwrap().to_str().unwrap();
    let format: ImageFormat = match ImageFormat::from_mime_type(given_mime_type) {
        Some(fmt) => fmt,
        None => {
            return (
                StatusCode::NOT_ACCEPTABLE,
                format!(
                    "The given MIME Type \"{}\" is not supported",
                    given_mime_type
                ),
            )
                .into_response()
        }
    };
    debug_print!("Detected MIME Type: \"{}\"", given_mime_type);

    let bytes = match Bytes::from_request(request, &app_state).await {
        Ok(b) => b.to_vec(),
        Err(_e) => {
            debug_print!("Error: {}", _e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to read image data from request body.\n",
            )
                .into_response();
        }
    };
    debug_print!("Extracted image data with byte length: {}", bytes.len());

    // Decode image using provided information
    let image = match image::load_from_memory_with_format(&bytes, format) {
        Ok(img) => img,
        Err(_e) => {
            debug_print!("Error: {}", _e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to load image from memory.\n",
            )
                .into_response();
        }
    };

    // Generate image pyramid
    let ipr = ipr::IprImage(&image);

    // For now we assume this is fast enough to do as part of the POST handler. Regardless, it is
    // much faster than tiling/compressing, so it should be done as a separate phase so that the
    // steps could be split into separate services, and scaled independently.
    let pyramid = match ipr.generate_image_pyramid() {
        Ok(p) => p,
        Err(_e) => {
            debug_print!("Error: {}", _e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to generate image pyramid.\n",
            )
                .into_response();
        }
    };

    let app = &mut app_state.write().await;

    if app.db.is_none() {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to acquire handle to image database.\n",
        )
            .into_response();
    }
    let db = app.db.as_ref().unwrap();

    // Write image pyramid levels to gridFS and aggregate the IDs
    let bucket = db.gridfs_bucket(None);
    let mut image_ids = Vec::new();
    for (i, img) in pyramid.iter().enumerate() {
        let mut data = Vec::new();
        let mut cursor = Cursor::new(&mut data);
        match img.write_to(&mut cursor, format) {
            Ok(_) => (),
            Err(_) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Failed to process image pyramid.\n",
                )
                    .into_response();
            }
        }

        let mut upload_stream = bucket.open_upload_stream(format!("pyramid_{}", i), None);
        match upload_stream.write_all(&data).await {
            Ok(_) => (),
            Err(_e) => {
                debug_print!("Error: {}", _e);
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Failed to upload image to database.\n",
                )
                    .into_response();
            }
        }

        let image_id = upload_stream.id();
        image_ids.push(image_id.clone());

        match upload_stream.close().await {
            Ok(_) => (),
            Err(_e) => {
                debug_print!("Error: {}", _e);
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Failed to close upload stream for image.\n",
                )
                    .into_response();
            }
        }
    }

    let pyramid_uuid = uuid::Uuid::new_v4();

    // For each image_id in the pyramid, generate a regular /api/v1/image/{name} resource, where
    // {name} is a UUID
    let mut image_names = Vec::new();
    let mut image_doc_ids = Vec::new();
    for (i, image_id) in image_ids.iter().enumerate() {
        let image_name = format!("{}_L{}", pyramid_uuid, i);
        image_names.push(image_name.clone());
        let doc = doc! {
            "name": image_name.clone(),
            "image": image_id,
            "mime_type": format.to_mime_type(),
        };

        let result = match db.collection("images").insert_one(doc, None).await {
            Ok(r) => r,
            Err(_e) => {
                debug_print!("Error: {}", _e);
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Failed to insert image into database.\n",
                )
                    .into_response();
            }
        };
        image_doc_ids.push(result.inserted_id);
    }

    // Figure out the image URL for each of the image_name values
    let image_urls = image_names
        .iter()
        .map(|name| format!("/api/v1/image/{}", name))
        .collect::<Vec<String>>();

    // Now we generate the actual doc of the pyramid. Set "tiles" to null
    let pyramid_doc = doc! {
        "uuid": format!("{}", pyramid_uuid),
        "url": format!("/api/v1/pyramid/{}", pyramid_uuid),
        "original_filename": image_name,
        "image_files": image_ids,
        "image_names": image_names,
        "image_docs": image_doc_ids,
        "image_urls": image_urls,
        "mime_type": format.to_mime_type(),
        "tiles": "todo",
    };

    let doc_json = match serde_json::to_string(&pyramid_doc) {
        Ok(j) => j,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to serialize pyramid document.\n",
            )
                .into_response();
        }
    };

    match db
        .collection("pyramids")
        .insert_one(pyramid_doc, None)
        .await
    {
        Ok(r) => r,
        Err(_e) => {
            debug_print!("Error: {}", _e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to insert pyramid into database.\n",
            )
                .into_response();
        }
    };

    // We have an image pyramid document, now kick off a new background task that takes the pyramid
    // and:
    //  0. Updates the pyramid doc such that "tiles" field is now "processing" and releases doc lock
    //  1. Breaks each image into tiles of 512x512 pixels
    //  2. Encodes the tile as a PNG and Brotli compresses the PNG data
    //  3. Updates the pyramid doc such that "tiles" field is now "done", when ALL tiles are done
    //  4. Updates the pyramid doc such that "tiles" field is now "failed" if any tile fails
    let pyramid_uuid_to_move = pyramid_uuid;
    let app_state_to_move = app_state.clone();
    let work = move || {
        web_routines::generate_tiles_for_pyramid(State(app_state_to_move), pyramid_uuid_to_move)
            .unwrap();
    };
    let bg_task = tokio::task::spawn_blocking(work);

    // Push bg_task join handle to app state... just in case.
    app.bg_tasks.insert(pyramid_uuid, Arc::new(bg_task));

    // Everything is set, now let's let the user know the pyramid is created!
    Response::builder()
        .status(StatusCode::CREATED)
        .header("Content-Type", "application/json")
        .body(Body::from(doc_json.to_string()))
        .unwrap()
}

#[utoipa::path(
    get,
    path = "/api/v1/pyramid/{uuid}",
    request_body(
        content = Bytes,
    ),
    responses(
        (status = StatusCode::OK, description = "Returned the image pyramid info with of the given uuid", body = Json),
        (status = StatusCode::NOT_FOUND, description = "No such image pyramid available", body = ()),
    )
)]
pub async fn get_pyramid(State(app_state): AppState, Path(uuid): Path<String>) -> Response {
    let app = &mut app_state.read().await;
    if app.db.is_none() {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to acquire handle to image database.\n",
        )
            .into_response();
    }
    let db = app.db.as_ref().unwrap();
    let pyramids: Collection<Document> = db.collection("pyramids");
    let pyramid = match pyramids
        .find_one(
            doc! {
                "uuid": uuid.clone()
            },
            None,
        )
        .await
    {
        Ok(d) => d,
        Err(_e) => {
            debug_print!("Error: {}", _e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to query image database.\n",
            )
                .into_response();
        }
    };

    if pyramid.is_none() {
        return (
            StatusCode::NOT_FOUND,
            format!("Pyramid {} not found.\n", uuid),
        )
            .into_response();
    }

    let pyramid = pyramid.unwrap();
    let json = match serde_json::to_string(&pyramid) {
        Ok(j) => j,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to serialize pyramid document.\n",
            )
                .into_response();
        }
    };

    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .body(Body::from(json.to_string()))
        .unwrap()
}

#[utoipa::path(
    get,
    path = "/api/v1/pyramids",
    responses(
        (status = StatusCode::OK, description = "Returned a JSON list of image documents", body = Json),
    )
)]
pub async fn get_pyramids(State(app_state): AppState) -> Response {
    let app = &mut app_state.read().await;
    if app.db.is_none() {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to acquire handle to image database.\n",
        )
            .into_response();
    }
    let db = app.db.as_ref().unwrap();
    let pyramids: Collection<Document> = db.collection("pyramids");
    let mut found = match pyramids.find(None, None).await {
        Ok(cursor) => cursor,
        Err(_e) => {
            debug_print!("Error: {}", _e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to query pyramid database.\n",
            )
                .into_response();
        }
    };

    let mut image_docs = Vec::new();
    while let Some(doc) = found.next().await {
        match doc {
            Ok(d) => {
                image_docs.push(d);
            }
            Err(_e) => {
                debug_print!("Error: {}", _e);
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Failed to read pyramid document.\n",
                )
                    .into_response();
            }
        }
    }

    let json = match serde_json::to_string(&image_docs) {
        Ok(j) => j,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to serialize pyramid document.\n",
            )
                .into_response();
        }
    };

    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .body(Body::from(json.to_string()))
        .unwrap()
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
pub async fn get_image(state: AppState, path: Path<String>, request: Request) -> Response {
    get_image_from_collection(state, path, request, "images").await
}

#[utoipa::path(
    put,
    path = "/api/v1/image/{name}",
    request_body(
        content = Bytes,
    ),
    responses(
        (status = StatusCode::OK, description = "Image of the given name is updated to the provided image data", body = ()),
        (status = StatusCode::CREATED, description = "No existing image found, so, image of the given name was added", body = ()),
        (status = StatusCode::INTERNAL_SERVER_ERROR, description = "Failed to read image from request", body = ()),
        (status = StatusCode::BAD_REQUEST, description = "Unable to handle request. Please pass an image body and specify content type.", body = ()),
        (status = StatusCode::NOT_ACCEPTABLE, description = "Unsupported image format.", body = ()),
    )
)]
pub async fn put_image(state: AppState, path: Path<String>, request: Request) -> Response {
    put_image_in_collection(state, path, request, "images").await
}

#[utoipa::path(
    delete,
    path = "/api/v1/image/{name}",
    request_body(
        content = Bytes,
    ),
    responses(
        (status = StatusCode::OK, description = "Image of the given name is updated to the provided image data", body = ()),
        (status = StatusCode::NOT_FOUND, description = "No existing image found; could not delete.", body = ()),
        (status = StatusCode::INTERNAL_SERVER_ERROR, description = "Failed to read image from request", body = ()),
    )
)]
pub async fn delete_image(state: AppState, path: Path<String>) -> Response {
    delete_image_from_collection(state, path, "images").await
}

///////////////////////////////////////////////////////////////////////////////////////////////////
// Same thing, but for pyramid levels
///////////////////////////////////////////////////////////////////////////////////////////////////

#[utoipa::path(
    get,
    path = "/api/v1/level/{name}",
    request_body(
        content = Bytes,
    ),
    responses(
        (status = StatusCode::OK, description = "Returned the image of the given name", body = Vec<u8>),
        (status = StatusCode::NOT_FOUND, description = "No such image available", body = ()),
    )
)]
pub async fn get_level(state: AppState, path: Path<String>, request: Request) -> Response {
    get_image_from_collection(state, path, request, "levels").await
}

#[utoipa::path(
    put,
    path = "/api/v1/level/{name}",
    request_body(
        content = Bytes,
    ),
    responses(
        (status = StatusCode::OK, description = "Image of the given name is updated to the provided image data", body = ()),
        (status = StatusCode::CREATED, description = "No existing image found, so, image of the given name was added", body = ()),
        (status = StatusCode::INTERNAL_SERVER_ERROR, description = "Failed to read image from request", body = ()),
        (status = StatusCode::BAD_REQUEST, description = "Unable to handle request. Please pass an image body and specify content type.", body = ()),
        (status = StatusCode::NOT_ACCEPTABLE, description = "Unsupported image format.", body = ()),
    )
)]
pub async fn put_level(state: AppState, path: Path<String>, request: Request) -> Response {
    put_image_in_collection(state, path, request, "levels").await
}

#[utoipa::path(
    delete,
    path = "/api/v1/level/{name}",
    request_body(
        content = Bytes,
    ),
    responses(
        (status = StatusCode::OK, description = "Image of the given name is updated to the provided image data", body = ()),
        (status = StatusCode::NOT_FOUND, description = "No existing image found; could not delete.", body = ()),
        (status = StatusCode::INTERNAL_SERVER_ERROR, description = "Failed to read image from request", body = ()),
    )
)]
pub async fn delete_level(state: AppState, path: Path<String>) -> Response {
    delete_image_from_collection(state, path, "levels").await
}

///////////////////////////////////////////////////////////////////////////////////////////////////
// Same thing, but for pyramid tiles
///////////////////////////////////////////////////////////////////////////////////////////////////

#[utoipa::path(
    get,
    path = "/api/v1/tile/{name}",
    request_body(
        content = Bytes,
    ),
    responses(
        (status = StatusCode::OK, description = "Returned the image of the given name", body = Vec<u8>),
        (status = StatusCode::NOT_FOUND, description = "No such image available", body = ()),
    )
)]
pub async fn get_tile(state: AppState, path: Path<String>, request: Request) -> Response {
    get_image_from_collection(state, path, request, "tiles").await
}

#[utoipa::path(
    put,
    path = "/api/v1/tile/{name}",
    request_body(
        content = Bytes,
    ),
    responses(
        (status = StatusCode::OK, description = "Image of the given name is updated to the provided image data", body = ()),
        (status = StatusCode::CREATED, description = "No existing image found, so, image of the given name was added", body = ()),
        (status = StatusCode::INTERNAL_SERVER_ERROR, description = "Failed to read image from request", body = ()),
        (status = StatusCode::BAD_REQUEST, description = "Unable to handle request. Please pass an image body and specify content type.", body = ()),
        (status = StatusCode::NOT_ACCEPTABLE, description = "Unsupported image format.", body = ()),
    )
)]
pub async fn put_tile(state: AppState, path: Path<String>, request: Request) -> Response {
    put_image_in_collection(state, path, request, "tiles").await
}

#[utoipa::path(
    delete,
    path = "/api/v1/tile/{name}",
    request_body(
        content = Bytes,
    ),
    responses(
        (status = StatusCode::OK, description = "Image of the given name is updated to the provided image data", body = ()),
        (status = StatusCode::NOT_FOUND, description = "No existing image found; could not delete.", body = ()),
        (status = StatusCode::INTERNAL_SERVER_ERROR, description = "Failed to read image from request", body = ()),
    )
)]
pub async fn delete_tile(state: AppState, path: Path<String>) -> Response {
    delete_image_from_collection(state, path, "tiles").await
}
