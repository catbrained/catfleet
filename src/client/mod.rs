use anyhow::anyhow;
use reqwest::{StatusCode, Url};
use tracing::{instrument, Level};

use crate::model::{
    ApiResponse, ApiResponseData, ApiStatus, FactionSymbol, RegisterAgent, RegisterAgentSuccess,
};

#[derive(Debug)]
pub struct Client {
    client: reqwest::Client,
    base_url: Url,
}

impl Client {
    #[instrument(level = Level::TRACE)]
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url: "https://api.spacetraders.io/v2/"
                .try_into()
                .expect("Base URL should be valid"),
        }
    }

    #[instrument(level = Level::TRACE)]
    pub fn new_with_url(url: &str) -> Result<Self, anyhow::Error> {
        Ok(Self {
            client: reqwest::Client::new(),
            base_url: url.try_into()?,
        })
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

    #[instrument(level = Level::DEBUG, skip(self))]
    pub async fn register_new_agent(
        &self,
        faction: FactionSymbol,
        agent_name: String,
        email: Option<String>,
    ) -> Result<RegisterAgentSuccess, anyhow::Error> {
        if !(3..=14).contains(&agent_name.len()) {
            return Err(anyhow!(
                "Agent name must be between 3 and 14 characters long"
            ));
        }

        let agent = RegisterAgent {
            faction,
            symbol: agent_name,
            email,
        };

        let url = self
            .base_url
            .join("register")
            .expect("Register URL should be valid");
        let res = self
            .client
            .post(url)
            .json(&agent)
            .send()
            .await
            .map_err(anyhow::Error::new)?;

        debug_assert_eq!(res.status(), StatusCode::CREATED);

        res.json::<ApiResponse>()
            .await
            .map(|res| {
                let ApiResponseData::RegisterAgentSuccess(r) = res.data;
                r
            })
            .map_err(anyhow::Error::new)
    }
}
