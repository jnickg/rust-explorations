//
// Modules
//

mod web_api;
mod web_appstate;

//
// Uses
//

use std::sync::Arc;

use tokio::sync::RwLock;

use axum::{
    body::Bytes,
    extract::{FromRequest, Path, Request, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Router,
};

use tower_http::trace;
extern crate tracing;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use utoipa::OpenApi;
use utoipa_rapidoc::RapiDoc;
use utoipa_redoc::{Redoc, Servable};
use utoipa_swagger_ui::SwaggerUi;

use rand::Rng;

use web_api as api;
use web_appstate::{AppState, RuntimeData};

//
// Implementation
//

async fn handler_404() -> Response {
    (StatusCode::NOT_FOUND, "404 Not Found").into_response()
}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "knock_knock=debug,info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
    // https://carlosmv.hashnode.dev/adding-logging-and-tracing-to-an-axum-app-rust
    let trace_layer = trace::TraceLayer::new_for_http()
        .make_span_with(trace::DefaultMakeSpan::new().level(tracing::Level::INFO))
        .on_response(trace::DefaultOnResponse::new().level(tracing::Level::INFO));

    let state: RuntimeData = RuntimeData::new();

    let api_routes = Router::new()
        .route("/hello", get(api::get_hello))
        .route("/something", post(api::post_something))
        .route(
            "/something/:id",
            get(api::get_something)
                .put(api::put_something)
                .delete(api::delete_something)
                .post(api::post_something_with_id),
        )
        .route("/image", post(api::post_image));

    let swagger_ui =
        SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", api::Documentation::openapi());
    let redoc_ui = Redoc::with_url("/redoc", api::Documentation::openapi());
    let rapidoc_ui = RapiDoc::new("/api-docs/openapi.json").path("/rapidoc");

    let app = Router::new()
        .route("/", get(api::get_index))
        .route("/index.html", get(api::get_index))
        .merge(swagger_ui)
        .merge(redoc_ui)
        .merge(rapidoc_ui)
        .nest("/api/v1", api_routes)
        .fallback(handler_404)
        .layer(trace_layer)
        .with_state(Arc::new(RwLock::new(state)));

    let port = 3000;
    println!("Listening on port {}", port);
    let listener = tokio::net::TcpListener::bind(format!("127.0.0.1:{}", port))
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();
}
