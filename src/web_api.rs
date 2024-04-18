use utoipa::OpenApi;

use crate::*;

#[derive(OpenApi)]
#[openapi(
    paths(
        get_something,
        put_something,
        delete_something,
        post_something,
        post_something_with_id,
        post_image
    ),
    tags(
        (name = "knock-knock", description = "Knock-Knock Joke API")
    )
)]
pub struct Documentation;

pub async fn get_index() -> Response {
    (StatusCode::OK, "Welcome to my website.").into_response()
}

pub async fn get_hello() -> Response {
    (StatusCode::OK, "Hello, World!").into_response()
}

#[utoipa::path(
    get,
    path = "/something/{id}",
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
    path = "/something/{id}",
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
    path = "/something/{id}",
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
    path = "/something",
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
    path = "/something/{id}",
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
    path = "/image",
    responses(
        (status = StatusCode::CREATED, description = "Added the image with the returned ID", body = str),
        (status = StatusCode::INTERNAL_SERVER_ERROR, description = "Failed to read image from request", body = ()),
        (status = StatusCode::BAD_REQUEST, description = "Unable to handle request. Please pass an image body and specify content type.", body = ()),
        (status = StatusCode::NOT_ACCEPTABLE, description = "Only PNG images are supported.", body = ())
    )
)]
pub async fn post_image(State(app_state): AppState, request: Request) -> Response {
    let content_type_hdr = request.headers().get("Content-Type");
    if content_type_hdr.is_none() {
        return (StatusCode::BAD_REQUEST, "Unable to handle request. Please pass an image body and specify content type.\n").into_response();
    }
    let mime_type = content_type_hdr.unwrap().to_str().unwrap();
    match mime_type {
        "image/png" => {
            println!("Image is a PNG.");
            let image_from_req = Bytes::from_request(request, &app_state).await;
            match image_from_req {
                Ok(image) => {
                    println!("Received image with {} bytes.", image.len());
                    let _app = &mut app_state.write().await;
                    todo!("Use `app` state to add new image!");
                    // (StatusCode::CREATED, "Image received.\n").into_response()
                }
                Err(_) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Failed to read image from request.\n",
                )
                    .into_response(),
            }
        }
        _ => (
            StatusCode::NOT_ACCEPTABLE,
            "Only PNG images are supported.\n",
        )
            .into_response(),
    }
}