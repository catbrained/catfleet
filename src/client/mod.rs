use std::{
    ops::{Deref, DerefMut},
    time::Duration,
};

use anyhow::anyhow;
use limit::{RateLimitWithBurst, RateLimitWithBurstLayer};
use reqwest::{
    header::{self, HeaderValue},
    Method, Request, StatusCode, Url,
};
use tower::{Service, ServiceBuilder, ServiceExt};
use tracing::{instrument, Level};

use crate::model::{
    Agent, ApiResponse, ApiResponseData, ApiStatus, Construction, Faction, FactionSymbol, JumpGate,
    Market, Meta, RegisterAgent, RegisterAgentSuccess, Shipyard, System, Waypoint,
    WaypointTraitSymbol, WaypointType,
};

mod limit;

const RATELIMIT_REQUESTS_DEFAULT: u64 = 2;
const RATELIMIT_DURATION_DEFAULT: Duration = Duration::from_secs(1);
const RATELIMIT_REQUESTS_BURST: u64 = 30;
const RATELIMIT_DURATION_BURST: Duration = Duration::from_secs(60);

#[derive(Debug)]
struct InnerClient(RateLimitWithBurst<reqwest::Client>);

impl InnerClient {
    fn new() -> Self {
        let client = reqwest::Client::new();
        let rate_limit = RateLimitWithBurstLayer::new(
            RATELIMIT_REQUESTS_DEFAULT,
            RATELIMIT_DURATION_DEFAULT,
            RATELIMIT_REQUESTS_BURST,
            RATELIMIT_DURATION_BURST,
        );

        let service = ServiceBuilder::new().layer(rate_limit).service(client);

        Self(service)
    }
}

impl Deref for InnerClient {
    type Target = RateLimitWithBurst<reqwest::Client>;

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
        let client = InnerClient::new();

        Self {
            client,
            base_url: "https://api.spacetraders.io/v2/"
                .try_into()
                .expect("Base URL should be valid"),
        }
    }

    #[instrument(level = Level::TRACE)]
    pub fn new_with_url(url: &str) -> Result<Self, anyhow::Error> {
        let client = InnerClient::new();

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
    ) -> Result<Box<RegisterAgentSuccess>, anyhow::Error> {
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

        match res.json::<ApiResponse>().await.map(|res| res.data) {
            Err(e) => Err(anyhow!(e)),
            Ok(ApiResponseData::RegisterAgent(s)) => Ok(s),
            Ok(d) => Err(anyhow!("Unexpected response data: {d:?}")),
        }
    }

    #[instrument(level = Level::DEBUG, skip(self))]
    pub async fn get_public_agent(&mut self, agent_name: String) -> Result<Agent, anyhow::Error> {
        if !(3..=14).contains(&agent_name.len()) {
            return Err(anyhow!(
                "Agent name must be between 3 and 14 characters long"
            ));
        }

        let url = self
            .base_url
            .join(&format!("agents/{agent_name}"))
            .map_err(anyhow::Error::new)?;

        let req = Request::new(Method::GET, url);

        let res = self.client.ready().await?.call(req).await?;

        match res.json::<ApiResponse>().await.map(|res| res.data) {
            Err(e) => Err(anyhow!(e)),
            Ok(ApiResponseData::PublicAgent(agent)) => Ok(agent),
            Ok(d) => Err(anyhow!("Unexpected response data: {d:?}")),
        }
    }

    #[instrument(level = Level::DEBUG, skip(self))]
    pub async fn get_system(&mut self, system_symbol: String) -> Result<System, anyhow::Error> {
        let url = self
            .base_url
            .join(&format!("systems/{system_symbol}"))
            .map_err(anyhow::Error::new)?;

        let req = Request::new(Method::GET, url);

        let res = self.client.ready().await?.call(req).await?;

        match res.json::<ApiResponse>().await.map(|res| res.data) {
            Err(e) => Err(anyhow!(e)),
            Ok(ApiResponseData::GetSystem(system)) => Ok(system),
            Ok(d) => Err(anyhow!("Unexpected response data: {d:?}")),
        }
    }

    #[instrument(level = Level::DEBUG, skip(self))]
    pub async fn get_waypoint(
        &mut self,
        waypoint_symbol: String,
    ) -> Result<Waypoint, anyhow::Error> {
        let (system_symbol, _) = waypoint_symbol.split_at(
            waypoint_symbol
                .rfind('-')
                .ok_or_else(|| anyhow!("Invalid waypoint symbol"))?,
        );
        let url = self
            .base_url
            .join(&format!(
                "systems/{system_symbol}/waypoints/{waypoint_symbol}"
            ))
            .map_err(anyhow::Error::new)?;

        let req = Request::new(Method::GET, url);

        let res = self.client.ready().await?.call(req).await?;

        match res.json::<ApiResponse>().await.map(|res| res.data) {
            Err(e) => Err(anyhow!(e)),
            Ok(ApiResponseData::GetWaypoint(waypoint)) => Ok(waypoint),
            Ok(d) => Err(anyhow!("Unexpected response data: {d:?}")),
        }
    }

    #[instrument(level = Level::DEBUG, skip(self))]
    pub async fn get_market(&mut self, waypoint_symbol: String) -> Result<Market, anyhow::Error> {
        let (system_symbol, _) = waypoint_symbol.split_at(
            waypoint_symbol
                .rfind('-')
                .ok_or_else(|| anyhow!("Invalid waypoint symbol"))?,
        );
        let url = self
            .base_url
            .join(&format!(
                "systems/{system_symbol}/waypoints/{waypoint_symbol}/market"
            ))
            .map_err(anyhow::Error::new)?;

        let req = Request::new(Method::GET, url);

        let res = self.client.ready().await?.call(req).await?;

        match res.json::<ApiResponse>().await.map(|res| res.data) {
            Err(e) => Err(anyhow!(e)),
            Ok(ApiResponseData::GetMarket(market)) => Ok(market),
            Ok(d) => Err(anyhow!("Unexpected response data: {d:?}")),
        }
    }

    #[instrument(level = Level::DEBUG, skip(self))]
    pub async fn get_shipyard(
        &mut self,
        waypoint_symbol: String,
    ) -> Result<Shipyard, anyhow::Error> {
        let (system_symbol, _) = waypoint_symbol.split_at(
            waypoint_symbol
                .rfind('-')
                .ok_or_else(|| anyhow!("Invalid waypoint symbol"))?,
        );
        let url = self
            .base_url
            .join(&format!(
                "systems/{system_symbol}/waypoints/{waypoint_symbol}/shipyard"
            ))
            .map_err(anyhow::Error::new)?;

        let req = Request::new(Method::GET, url);

        let res = self.client.ready().await?.call(req).await?;

        match res.json::<ApiResponse>().await.map(|res| res.data) {
            Err(e) => Err(anyhow!(e)),
            Ok(ApiResponseData::GetShipyard(shipyard)) => Ok(shipyard),
            Ok(d) => Err(anyhow!("Unexpected response data: {d:?}")),
        }
    }

    #[instrument(level = Level::DEBUG, skip(self))]
    pub async fn get_jumpgate(
        &mut self,
        waypoint_symbol: String,
    ) -> Result<JumpGate, anyhow::Error> {
        let (system_symbol, _) = waypoint_symbol.split_at(
            waypoint_symbol
                .rfind('-')
                .ok_or_else(|| anyhow!("Invalid waypoint symbol"))?,
        );
        let url = self
            .base_url
            .join(&format!(
                "systems/{system_symbol}/waypoints/{waypoint_symbol}/jump-gate"
            ))
            .map_err(anyhow::Error::new)?;

        let req = Request::new(Method::GET, url);

        let res = self.client.ready().await?.call(req).await?;

        match res.json::<ApiResponse>().await.map(|res| res.data) {
            Err(e) => Err(anyhow!(e)),
            Ok(ApiResponseData::GetJumpGate(gate)) => Ok(gate),
            Ok(d) => Err(anyhow!("Unexpected response data: {d:?}")),
        }
    }

    #[instrument(level = Level::DEBUG, skip(self))]
    pub async fn get_construction_site(
        &mut self,
        waypoint_symbol: String,
    ) -> Result<Construction, anyhow::Error> {
        let (system_symbol, _) = waypoint_symbol.split_at(
            waypoint_symbol
                .rfind('-')
                .ok_or_else(|| anyhow!("Invalid waypoint symbol"))?,
        );
        let url = self
            .base_url
            .join(&format!(
                "systems/{system_symbol}/waypoints/{waypoint_symbol}/construction"
            ))
            .map_err(anyhow::Error::new)?;

        let req = Request::new(Method::GET, url);

        let res = self.client.ready().await?.call(req).await?;

        match res.json::<ApiResponse>().await.map(|res| res.data) {
            Err(e) => Err(anyhow!(e)),
            Ok(ApiResponseData::GetConstructionSite(construction)) => Ok(construction),
            Ok(d) => Err(anyhow!("Unexpected response data: {d:?}")),
        }
    }

    #[instrument(level = Level::DEBUG, skip(self))]
    pub async fn list_agents(
        &mut self,
        limit: Option<u64>,
        page: Option<u64>,
    ) -> Result<(Vec<Agent>, Meta), anyhow::Error> {
        let limit = limit.unwrap_or(10);
        let page = page.unwrap_or(1);

        let url = self
            .base_url
            .join(&format!("agents?limit={limit}&page={page}"))
            .map_err(anyhow::Error::new)?;

        let req = Request::new(Method::GET, url);

        let res = self.client.ready().await?.call(req).await?;

        match res.json::<ApiResponse>().await {
            Err(e) => Err(anyhow!(e)),
            Ok(ApiResponse { data, meta }) => match data {
                ApiResponseData::ListAgents(agents) => Ok((agents, meta)),
                _ => Err(anyhow!("Unexpected response data: {data:?}")),
            },
        }
    }

    #[instrument(level = Level::DEBUG, skip(self))]
    pub async fn list_factions(
        &mut self,
        limit: Option<u64>,
        page: Option<u64>,
    ) -> Result<(Vec<Faction>, Meta), anyhow::Error> {
        let limit = limit.unwrap_or(10);
        let page = page.unwrap_or(1);

        let url = self
            .base_url
            .join(&format!("factions?limit={limit}&page={page}"))
            .map_err(anyhow::Error::new)?;

        let req = Request::new(Method::GET, url);

        let res = self.client.ready().await?.call(req).await?;

        match res.json::<ApiResponse>().await {
            Err(e) => Err(anyhow!(e)),
            Ok(ApiResponse { data, meta }) => match data {
                ApiResponseData::ListFactions(factions) => Ok((factions, meta)),
                _ => Err(anyhow!("Unexpected response data: {data:?}")),
            },
        }
    }

    #[instrument(level = Level::DEBUG, skip(self))]
    pub async fn list_systems(
        &mut self,
        limit: Option<u64>,
        page: Option<u64>,
    ) -> Result<(Vec<System>, Meta), anyhow::Error> {
        let limit = limit.unwrap_or(10);
        let page = page.unwrap_or(1);

        let url = self
            .base_url
            .join(&format!("systems?limit={limit}&page={page}"))
            .map_err(anyhow::Error::new)?;

        let req = Request::new(Method::GET, url);

        let res = self.client.ready().await?.call(req).await?;

        match res.json::<ApiResponse>().await {
            Err(e) => Err(anyhow!(e)),
            Ok(ApiResponse { data, meta }) => match data {
                ApiResponseData::ListSystems(systems) => Ok((systems, meta)),
                _ => Err(anyhow!("Unexpected response data: {data:?}")),
            },
        }
    }

    #[instrument(level = Level::DEBUG, skip(self))]
    pub async fn list_waypoints(
        &mut self,
        system_symbol: String,
        limit: Option<u64>,
        page: Option<u64>,
        traits: Option<Vec<WaypointTraitSymbol>>,
        waypoint_type: Option<WaypointType>,
    ) -> Result<(Vec<Waypoint>, Meta), anyhow::Error> {
        let limit = limit.unwrap_or(10);
        let page = page.unwrap_or(1);
        let waypoint_type = waypoint_type.map(|t| t.to_string()).unwrap_or_default();
        let traits = traits
            .map(|t| {
                t.iter().fold(String::new(), |mut acc, el| {
                    acc.push_str(&format!("&traits={el}"));
                    acc
                })
            })
            .unwrap_or_default();
        let traits = traits.trim_end_matches('+');

        let url = self
            .base_url
            .join(&format!("systems/{system_symbol}/waypoints?limit={limit}&page={page}&type={waypoint_type}{traits}"))
            .map_err(anyhow::Error::new)?;

        let req = Request::new(Method::GET, url);

        let res = self.client.ready().await?.call(req).await?;

        match res.json::<ApiResponse>().await {
            Err(e) => Err(anyhow!(e)),
            Ok(ApiResponse { data, meta }) => match data {
                ApiResponseData::ListWaypoints(waypoints) => Ok((waypoints, meta)),
                _ => Err(anyhow!("Unexpected response data: {data:?}")),
            },
        }
    }
}
