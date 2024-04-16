use axum::{
    extract::{Path, Request, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{
        get, post
    },
    Router};

use std::sync::{Arc, Mutex};
use std::collections::HashSet;
use rand::Rng;

#[derive(Clone)]
struct RuntimeData {
    somethings: HashSet<u32>
}

impl RuntimeData {
    pub fn new() -> Self {
        RuntimeData {
            somethings: HashSet::<u32>::new()
        }
    }
}

type AppState = State<Arc<Mutex<RuntimeData>>>;

async fn index() -> Response {
    (StatusCode::OK, "Welcome to my website.").into_response()
}

async fn hello() -> Response {
    (StatusCode::OK, "Hello, World!").into_response()
}

async fn get_something(State(app_state): AppState, Path(id): Path<u32>) -> Response {
    let app = app_state.lock();
    match app {
        Ok(app) => {
            match app.somethings.get(&id) {
                Some(_) => {
                    (StatusCode::OK, format!("Getting something with id {}\n", id)).into_response()
                },
                None => {
                    (StatusCode::NOT_FOUND, format!("Something with id {} not found.\n", id)).into_response()
                }
            }
        },
        Err(_) => {
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to acquire lock on app state.\n").into_response()
        }
    }
}

async fn put_something(State(app_state): AppState, Path(id): Path<u32>) -> Response {
    let app = &mut app_state.lock();
    match app {
        Ok(app) => {
            if app.somethings.contains(&id) {
                return (StatusCode::OK, format!("Updating something with id {}\n", id)).into_response()
            }
            let somethings = &mut app.somethings;
            somethings.insert(id);
            (StatusCode::CREATED, format!("Creating something with id {}\n", id)).into_response()
        },
        Err(_) => {
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to acquire lock on app state.\n").into_response()
        }
    }
}

async fn delete_something(State(app_state): AppState, Path(id): Path<u32>) -> Response {
    let app = &mut app_state.lock();
    match app {
        Ok(app) => {
            if app.somethings.contains(&id) {
                app.somethings.remove(&id);
                return (StatusCode::OK, format!("Deleting something with id {}\n", id)).into_response()
            }
            (StatusCode::NOT_FOUND, format!("Something with id {} not found.\n", id)).into_response()
        },
        Err(_) => {
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to acquire lock on app state.\n").into_response()
        }
    }

}

async fn post_something(State(app_state): AppState) -> Response {
    let mut rng = rand::thread_rng();
    let app = &mut app_state.lock();
    match app {
        Ok(app) => {
            // Probably wiser to use an incrementing counter that skips past any IDs manually added
            // but this is a good excuse to exercise using a loop to return a value
            let id = loop {
                let random_id = rng.gen::<u32>();
                if app.somethings.contains(&random_id) {
                    continue;
                }
                break random_id;
            };
            app.somethings.insert(id);
            (StatusCode::CREATED, format!("Posting something with id {}\n", id)).into_response()
        },
        Err(_) => {
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to acquire lock on app state.\n").into_response()
        }
    }
}

async fn post_something_with_id(State(app_state): AppState, Path(id): Path<u32>) -> Response {
    let app = &mut app_state.lock();
    match app {
        Ok(app) => {
            let inserted = app.somethings.insert(id);
            match inserted {
                true => {
                    (StatusCode::CREATED, format!("Posting something with id {}\n", id)).into_response()
                },
                false => {
                    (StatusCode::CONFLICT, format!("Something with id {} already exists.\n", id)).into_response()
                }
            }
        },
        Err(_) => {
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to acquire lock on app state.\n").into_response()
        }
    }
}

async fn post_image(State(app_state): AppState, request: Request) -> Response {
    let app = &mut app_state.lock();
    match app {
        Ok(_) => {
            let content_type_hdr = request.headers().get("Content-Type");
            if content_type_hdr.is_none() {
                return (StatusCode::BAD_REQUEST, "Unable to handle request. Please pass an image body and specify content type.\n").into_response()
            }
            let mime_type = content_type_hdr.unwrap().to_str().unwrap();
            match mime_type {
                "image/png" => {
                    todo!("Add image to app state and return StatusCode::CREATED");
                },
                _ => {
                    (StatusCode::INTERNAL_SERVER_ERROR, "Only PNG images are supported.\n").into_response()
                }
            }
        },
        Err(_) => {
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to acquire lock on app state.\n").into_response()
        }
    }
}

#[tokio::main]
async fn main() {
    let state: RuntimeData = RuntimeData::new();

    let app = Router::new()
        .route("/", get(index))
        .route("/hello", get(hello))
        .route("/api/something", post(post_something))
        .route("/api/something/:id",  get(get_something)
                                    .put(put_something)
                                    .delete(delete_something)
                                    .post(post_something_with_id))
        .route("/api/image", post(post_image))
        .with_state(Arc::new(Mutex::new(state)));
    println!("Router created.");

    let port = 3000;
    println!("Listening on port {}", port);
    let listener = tokio::net::TcpListener::bind(format!("127.0.0.1:{}", port)).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}