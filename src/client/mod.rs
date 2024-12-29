use reqwest::{StatusCode, Url};
use tracing::{instrument, Level};

use crate::model::ApiStatus;

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

    #[instrument(level = Level::DEBUG, skip(self))]
    pub async fn get_status(&self) -> Result<ApiStatus, anyhow::Error> {
        let res = self
            .client
            .get(self.base_url.clone())
            .send()
            .await
            .map_err(anyhow::Error::new)?;

        debug_assert_eq!(res.status(), StatusCode::OK);

        res.json().await.map_err(anyhow::Error::new)
    }
}
