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

    let client = client::Client::new();

    let status = client.get_status().await;

    println!("{:?}", status);
}
