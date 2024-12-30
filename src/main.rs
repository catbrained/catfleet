use tracing::{event, Level};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

mod client;
mod model;

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| {
            if cfg!(debug_assertions) {
                format!("{}=debug", env!("CARGO_CRATE_NAME")).into()
            } else {
                format!("{}=info", env!("CARGO_CRATE_NAME")).into()
            }
        }))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let mut client = client::Client::new();

    match client.get_status().await {
        Ok(status) => event!(Level::INFO, status.status),
        Err(e) => event!(Level::ERROR, %e),
    }
}
