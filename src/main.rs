use axum::{
    extract::{Path, Request, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{
        get, post
    },
    Router};

use std::sync::Arc;
use std::collections::HashSet;

#[derive(Clone)]
struct MyAppState {
    somethings: HashSet<usize>
}

impl MyAppState {
    pub fn new() -> Self {
        MyAppState {
            somethings: HashSet::<usize>::new()
        }
    }
}

async fn index() -> Response {
    (StatusCode::OK, "Welcome to my website.").into_response()
}

async fn hello() -> Response {
    (StatusCode::OK, "Hello, World!").into_response()
}

async fn get_something(State(app_state): State<MyAppState>, Path(id): Path<u32>) -> Response {
    (StatusCode::OK, format!("Getting something with id {}\n", id)).into_response()
}

async fn put_something(State(app_state): State<MyAppState>, Path(id): Path<u32>) -> Response {
    (StatusCode::OK, format!("Putting something with id {}\n", id)).into_response()
}

async fn delete_something(State(app_state): State<MyAppState>, Path(id): Path<u32>) -> Response {
    (StatusCode::OK, format!("Deleting something with id {}\n", id)).into_response()
}

async fn post_something(State(app_state): State<MyAppState>) -> Response {
    let new_id = 42;
    (StatusCode::CREATED, format!("Posting something with id {}\n", new_id)).into_response()
}

async fn post_something_with_id(State(app_state): State<MyAppState>, Path(id): Path<u32>) -> Response {
    (StatusCode::CREATED, format!("Posting something with id {}\n", id)).into_response()
}

async fn post_image(State(app_state): State<MyAppState>, request: Request) -> Response {
    if let Some(content_type) = request.headers().get("Content-Type") {
        let mime_type = content_type.to_str().unwrap();
        match mime_type {
            "image/png" => {
                // let image_data = to_bytes(body_copy, usize::MAX).await.unwrap();
                return (StatusCode::CREATED, "Image uploaded successfully.\n").into_response()
            }
            _ => {
                return (StatusCode::INTERNAL_SERVER_ERROR, "Only PNG images are supported.\n").into_response()
            }
        }
    }
    (StatusCode::BAD_REQUEST, "Unable to handle request. Please pass an image body and specify content type.\n").into_response()
}

#[tokio::main]
async fn main() {
    let state: MyAppState = MyAppState::new();

    let app = Router::new()
        .route("/", get(index))
        .route("/hello", get(hello))
        .route("/api/something", post(post_something))
        .route("/api/something/:id",  get(get_something)
                                    .put(put_something)
                                    .delete(delete_something)
                                    .post(post_something_with_id))
        .route("/api/image", post(post_image))
        .with_state(Arc::new(state));
    println!("Router created.");

    let port = 3000;
    println!("Listening on port {}", port);
    let listener = tokio::net::TcpListener::bind(format!("127.0.0.1:{}", port)).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}