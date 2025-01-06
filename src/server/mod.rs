use std::sync::Arc;

use axum::{extract::State, response::IntoResponse, routing::get, Json, Router};
use tokio::sync::Mutex;
use tracing::{event, instrument, Level};

use crate::client::Client;

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

    let app = Router::new()
        .route("/status", get(status))
        .with_state(state);

    let address = "127.0.0.1:3000";
    let listener = tokio::net::TcpListener::bind(address).await.unwrap();

    event!(Level::INFO, "Starting server on `http://{address}`");
    axum::serve(listener, app).await.unwrap();
}

#[instrument(level = Level::DEBUG, skip(state))]
async fn status(State(state): State<AppState>) -> impl IntoResponse {
    let status = state.http_client.lock().await.get_status().await.unwrap();

    Json(status)
}
