use reqwest::Url;
use tracing::{instrument, Level};

pub struct Client {
    client: reqwest::Client,
    base_url: Url,
}

impl Client {
    #[instrument(level = Level::TRACE)]
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url: "https://api.spacetraders.io/v2"
                .try_into()
                .expect("Base URL should be valid"),
        }
    }
}
