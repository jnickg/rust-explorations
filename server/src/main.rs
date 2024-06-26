#![feature(test)]
//
// Modules
//

mod axum_helpers;
mod utoipa_helpers;
mod web_api;
mod web_appstate;
mod web_routines;
mod window_iterator;
mod wrappers;

//
// Uses
//

use std::sync::Arc;

use axum::extract::DefaultBodyLimit;
use clap::{arg, Parser};

use tokio::sync::RwLock;

use ::axum::{
    body::Bytes,
    extract::{FromRequest, Path, Request, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Router,
};

use mongodb::Client;

use tower::ServiceExt;
use tower_http::{services::ServeDir, trace};
extern crate tracing;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use ::utoipa::OpenApi;
use ::utoipa_rapidoc::RapiDoc;
use ::utoipa_redoc::{Redoc, Servable};
use ::utoipa_swagger_ui::SwaggerUi;

use rand::Rng;

use web_api as api;
use web_appstate::{AppState, RuntimeData};

//
// Implementation
//

async fn handler_404() -> Response {
    (StatusCode::NOT_FOUND, "404 Not Found").into_response()
}

#[derive(Parser, Debug)]
#[command(
    version,
    about,
    long_about = "A tiling service written in rust. Breaks down images into bite-size tiles and serves them to application clients"
)]
struct Args {
    /// Hostname of the MongoDB server
    #[arg(long, value_name = "STR")]
    host: String,

    /// Username with which to log into the MongoDB
    #[arg(long, value_name = "STR")]
    user: String,

    /// The path to a file containing the password for MongoDB
    #[arg(long, value_name = "PATH")]
    pass: String,

    /// The port through which to access MongoDB
    #[arg(long = "db-port", value_name = "NUM")]
    db_port: u16,

    /// The port on which the server runs
    #[arg(long, value_name = "NUM")]
    port: u16,

    /// Set the directory where static files are to be found for serving
    #[clap(long = "static-dir", default_value = "./dist")]
    static_dir: String,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let mut state: RuntimeData = RuntimeData::new();

    let password_str = std::fs::read_to_string(&args.pass).unwrap();
    let uri = format!(
        "mongodb://{}:{}@{}:{}/",
        args.user, password_str, args.host, args.db_port
    );
    let client = Client::with_uri_str(uri).await;
    let database = match client {
        Ok(c) => c.database("tiler"),
        Err(_e) => {
            eprintln!("Error: {:?}", _e);
            return;
        }
    };
    state.db = Some(database);

    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .init();
    // https://carlosmv.hashnode.dev/adding-logging-and-tracing-to-an-axum-app-rust
    let trace_layer = trace::TraceLayer::new_for_http()
        .make_span_with(trace::DefaultMakeSpan::new().level(tracing::Level::INFO))
        .on_response(trace::DefaultOnResponse::new().level(tracing::Level::INFO));

    let api_routes = Router::new()
        .route("/", get(api::get_api_index))
        .route("/index.html", get(api::get_api_index))
        .route("/hello", get(api::get_hello))
        .route("/something", post(api::post_something))
        .route(
            "/something/:id",
            get(api::get_something)
                .put(api::put_something)
                .delete(api::delete_something)
                .post(api::post_something_with_id),
        )
        .route("/image", post(api::post_image))
        .route("/images", get(api::get_images))
        .route(
            "/image/:name",
            get(api::get_image)
                .put(api::put_image)
                .delete(api::delete_image),
        )
        .route(
            "/level/:name",
            get(api::get_level)
                .put(api::put_level)
                .delete(api::delete_level),
        )
        .route(
            "/tile/:name",
            get(api::get_tile)
                .put(api::put_tile)
                .delete(api::delete_tile),
        )
        .route(
            "/matrix/:name",
            post(api::post_matrix_with_name)
                .get(api::get_matrix)
                .put(api::put_matrix)
                .delete(api::delete_matrix),
        )
        .route("/matrix/:name/dims", get(api::get_matrix_dims))
        .route(
            "/matrix/multiply/:name1/:name2",
            post(api::post_matrix_multiply),
        )
        .route("/matrix/add/:name1/:name2", post(api::post_matrix_add))
        .route(
            "/matrix/subtract/:name1/:name2",
            post(api::post_matrix_subtract),
        )
        .route("/pyramid", post(api::post_pyramid))
        .route("/pyramid/:uuid", get(api::get_pyramid))
        .route("/pyramids", get(api::get_pyramids));

    let swagger_ui =
        SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", api::Documentation::openapi());
    let redoc_ui = Redoc::with_url("/redoc", api::Documentation::openapi());
    let rapidoc_ui = RapiDoc::new("/api-docs/openapi.json").path("/rapidoc");

    let app = Router::new()
        .fallback_service(get(|req| async move {
            ServeDir::new(args.static_dir).oneshot(req).await.unwrap()
        }))
        .merge(swagger_ui)
        .merge(redoc_ui)
        .merge(rapidoc_ui)
        .nest("/api/v1", api_routes)
        .fallback(handler_404)
        .layer(trace_layer)
        .layer(DefaultBodyLimit::disable())
        .with_state(Arc::new(RwLock::new(state)));

    println!("Listening on port {}", args.port);
    let listener = tokio::net::TcpListener::bind(format!("127.0.0.1:{}", args.port))
        .await
        .unwrap();
    ::axum::serve(listener, app).await.unwrap();
}
