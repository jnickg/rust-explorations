use axum::{
    extract::Path,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{
        get, post
    },
    Json,
    Router};

async fn index() -> Response {
    (StatusCode::OK, "Welcome to my website.").into_response()
}

async fn hello() -> Response {
    (StatusCode::OK, "Hello, World!").into_response()
}

async fn get_something(Path(id): Path<u32>) -> Response {
    (StatusCode::OK, format!("Getting something with id {}", id)).into_response()
}

async fn put_something(Path(id): Path<u32>) -> Response {
    (StatusCode::OK, format!("Putting something with id {}", id)).into_response()
}

async fn delete_something(Path(id): Path<u32>) -> Response {
    (StatusCode::OK, format!("Deleting something with id {}", id)).into_response()
}

async fn post_something() -> Response {
    let new_id = 42;
    (StatusCode::OK, format!("Posting something with id {}", new_id)).into_response()
}

async fn post_something_with_id(Path(id): Path<u32>, Json(payload): Json<String>) -> Response {
    (StatusCode::OK, format!("Posting something with id {} and payload {}", id, payload)).into_response()
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(index))
        .route("/hello", get(hello))
        .route("/api/something", post(post_something))
        .route("/api/something/:id",  get(get_something)
                                    .put(put_something)
                                    .delete(delete_something)
                                    .post(post_something_with_id));
    println!("Router created.");

    let port = 3000;
    println!("Listening on port {}", port);
    let listener = tokio::net::TcpListener::bind(format!("127.0.0.1:{}", port)).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}