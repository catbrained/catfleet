use std::{
    ops::{Deref, DerefMut},
    time::Duration,
};

use anyhow::anyhow;
use reqwest::{
    header::{self, HeaderValue},
    Method, Request, StatusCode, Url,
};
use tower::{
    limit::{RateLimit, RateLimitLayer},
    Layer, Service, ServiceExt,
};
use tracing::{instrument, Level};

use crate::model::{
    ApiResponse, ApiResponseData, ApiStatus, FactionSymbol, RegisterAgent, RegisterAgentSuccess,
};

const RATELIMIT_REQUESTS: u64 = 2;
const RATELIMIT_DURATION: Duration = Duration::from_secs(1);

#[derive(Debug)]
struct InnerClient(RateLimit<reqwest::Client>);

impl Deref for InnerClient {
    type Target = RateLimit<reqwest::Client>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for InnerClient {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Debug)]
pub struct Client {
    client: InnerClient,
    base_url: Url,
}

impl Client {
    #[instrument(level = Level::TRACE)]
    pub fn new() -> Self {
        let client = reqwest::Client::new();
        let rate_limit = RateLimitLayer::new(RATELIMIT_REQUESTS, RATELIMIT_DURATION);

        let client = InnerClient(rate_limit.layer(client));

        Self {
            client,
            base_url: "https://api.spacetraders.io/v2/"
                .try_into()
                .expect("Base URL should be valid"),
        }
    }

    #[instrument(level = Level::TRACE)]
    pub fn new_with_url(url: &str) -> Result<Self, anyhow::Error> {
        let client = reqwest::Client::new();
        let rate_limit = RateLimitLayer::new(RATELIMIT_REQUESTS, RATELIMIT_DURATION);

        let client = InnerClient(rate_limit.layer(client));

        Ok(Self {
            client,
            base_url: url.try_into()?,
        })
    }

    #[instrument(level = Level::DEBUG, skip(self))]
    pub async fn get_status(&mut self) -> Result<ApiStatus, anyhow::Error> {
        let req = Request::new(Method::GET, self.base_url.clone());
        let res = self.client.ready().await?.call(req).await?;

        debug_assert_eq!(res.status(), StatusCode::OK);

        res.json().await.map_err(anyhow::Error::new)
    }

    #[instrument(level = Level::DEBUG, skip(self))]
    pub async fn register_new_agent(
        &mut self,
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
        let mut req = Request::new(Method::POST, url);

        match serde_json::to_vec(&agent) {
            Ok(body) => {
                req.headers_mut().insert(
                    header::CONTENT_TYPE,
                    HeaderValue::from_static("application/json"),
                );
                *req.body_mut() = Some(body.into());
            }
            Err(e) => return Err(anyhow!(e)),
        }

        let res = self.client.ready().await?.call(req).await?;

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
