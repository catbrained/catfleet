use std::sync::Arc;

use axum::{extract::State, response::IntoResponse, routing::get, Json};
use tokio::sync::Mutex;
use tracing::{event, instrument, Level};
use utoipa::OpenApi;
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::client::Client;

#[derive(OpenApi)]
#[openapi()]
struct ApiDoc;

#[derive(Clone)]
struct AppState {
    http_client: Arc<Mutex<Client>>,
}

#[instrument(name = "catfleet_server", level = Level::INFO)]
pub async fn start() {
    let client = Arc::new(Mutex::new(Client::new().await.unwrap()));
    let state = AppState {
        http_client: client,
    };

    let (app, openapi) = OpenApiRouter::with_openapi(ApiDoc::openapi())
        .routes(routes!(status))
        .with_state(state)
        .split_for_parts();

    let app = app.route(
        "/api-docs/openapi.json",
        get(move || async { Json(openapi) }),
    );

    let address = "127.0.0.1:3000";
    let listener = tokio::net::TcpListener::bind(address).await.unwrap();

    event!(Level::INFO, "Starting server on `http://{address}`");
    axum::serve(listener, app).await.unwrap();
}

/// Returns the SpaceTraders API status.
#[utoipa::path(
    get,
    path = "/status",
    responses(
        (status = 200, body = crate::model::ApiStatus)
    )
)]
#[instrument(level = Level::DEBUG, skip(state))]
async fn status(State(state): State<AppState>) -> impl IntoResponse {
    let status = state.http_client.lock().await.get_status().await.unwrap();

    Json(status)
}
