use std::{
    ops::{Deref, DerefMut},
    time::Duration,
};

use anyhow::anyhow;
use http_body_util::{BodyExt, Full};
use hyper::{
    body::{Buf, Bytes},
    header, Method, Request, StatusCode,
};
use tower::{Service, ServiceBuilder, ServiceExt};
use tower_http::auth::{AddAuthorization, AddAuthorizationLayer};
use tracing::{event, instrument, Level};

use crate::model::{
    Agent, ApiResponse, ApiResponseData, ApiStatus, Chart, Construction, Contract, Cooldown,
    DeliverCargo, Destination, Extraction, Faction, FactionSymbol, JumpGate, Market,
    MarketTransaction, Meta, Produce, RegisterAgent, RegisterAgentSuccess, Ship, ShipCargo,
    ShipConditionEvent, ShipFuel, ShipMount, ShipNav, ShipPurchase, ShipTransaction, ShipType,
    Shipyard, ShipyardTransaction, Siphon, Survey, System, TradeGoodAmount, TradeSymbol, Waypoint,
    WaypointTraitSymbol, WaypointType,
};
use inner::InnerClient;
use limit::{RateLimitWithBurst, RateLimitWithBurstLayer};

pub mod inner;
mod limit;

const RATELIMIT_REQUESTS_DEFAULT: u64 = 2;
const RATELIMIT_DURATION_DEFAULT: Duration = Duration::from_secs(1);
const RATELIMIT_REQUESTS_BURST: u64 = 30;
const RATELIMIT_DURATION_BURST: Duration = Duration::from_secs(60);

#[derive(Debug)]
struct WrappedClient(RateLimitWithBurst<AddAuthorization<InnerClient<Full<Bytes>>>>);

impl WrappedClient {
    async fn new(url: &str) -> Result<Self, anyhow::Error> {
        let client = InnerClient::new(url).await?;
        let rate_limit = RateLimitWithBurstLayer::new(
            RATELIMIT_REQUESTS_DEFAULT,
            RATELIMIT_DURATION_DEFAULT,
            RATELIMIT_REQUESTS_BURST,
            RATELIMIT_DURATION_BURST,
        );
        let token = std::env::var("SPACETRADERS_TOKEN")
            .map_err(|err| event!(Level::ERROR, %err, "SPACETRADERS_TOKEN not found"))
            .unwrap();
        let auth = AddAuthorizationLayer::bearer(&token).as_sensitive(true);

        let service = ServiceBuilder::new()
            .layer(rate_limit)
            .layer(auth)
            .service(client);

        Ok(Self(service))
    }
}

impl Deref for WrappedClient {
    type Target = RateLimitWithBurst<AddAuthorization<InnerClient<Full<Bytes>>>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for WrappedClient {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Debug)]
pub struct Client {
    inner: WrappedClient,
}

impl Client {
    #[instrument(level = Level::TRACE)]
    pub async fn new() -> Result<Self, anyhow::Error> {
        let client = WrappedClient::new("https://api.spacetraders.io/v2/").await?;

        Ok(Self { inner: client })
    }

    #[instrument(level = Level::TRACE)]
    pub async fn new_with_url(url: &str) -> Result<Self, anyhow::Error> {
        let client = WrappedClient::new(url).await?;

        Ok(Self { inner: client })
    }

    #[instrument(level = Level::DEBUG, skip(self), err(Debug))]
    pub async fn get_status(&mut self) -> Result<ApiStatus, anyhow::Error> {
        // Path for GET status is the base URL,
        // so no need to specify it here, since
        // the inner client will take care of it.
        let req = Request::builder()
            .method(Method::GET)
            .body(Full::<Bytes>::new(Bytes::new()))?;

        let res = self.inner.ready().await?.call(req).await?;
        event!(Level::DEBUG, "Response status: {}", res.status());

        let body = res.collect().await?.aggregate();

        serde_json::from_reader(body.reader()).map_err(anyhow::Error::new)
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

        let body = match serde_json::to_vec(&agent) {
            Ok(body) => body,
            Err(e) => return Err(anyhow!(e)),
        };

        let req = Request::builder()
            .uri("/register")
            .method(Method::POST)
            .header(header::CONTENT_TYPE, "application/json")
            .body(Full::<Bytes>::new(body.into()))?;

        let res = self.inner.ready().await?.call(req).await?;
        event!(Level::DEBUG, "Response status: {}", res.status());

        let body = res.collect().await?.aggregate();

        let json = serde_json::from_reader(body.reader()).map(|res: ApiResponse| res.data);
        match json {
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

        let req = Request::builder()
            .uri(format!("/agents/{agent_name}"))
            .method(Method::GET)
            .body(Full::<Bytes>::new(Bytes::new()))?;

        let res = self.inner.ready().await?.call(req).await?;
        event!(Level::DEBUG, "Response status: {}", res.status());

        let body = res.collect().await?.aggregate();

        let json = serde_json::from_reader(body.reader()).map(|res: ApiResponse| res.data);
        match json {
            Err(e) => Err(anyhow!(e)),
            Ok(ApiResponseData::GetAgent(agent)) => Ok(agent),
            Ok(d) => Err(anyhow!("Unexpected response data: {d:?}")),
        }
    }

    #[instrument(level = Level::DEBUG, skip(self))]
    pub async fn get_system(&mut self, system_symbol: String) -> Result<System, anyhow::Error> {
        let req = Request::builder()
            .uri(format!("/systems/{system_symbol}"))
            .method(Method::GET)
            .body(Full::<Bytes>::new(Bytes::new()))?;

        let res = self.inner.ready().await?.call(req).await?;
        event!(Level::DEBUG, "Response status: {}", res.status());

        let body = res.collect().await?.aggregate();

        let json = serde_json::from_reader(body.reader()).map(|res: ApiResponse| res.data);
        match json {
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

        let req = Request::builder()
            .uri(format!(
                "/systems/{system_symbol}/waypoints/{waypoint_symbol}"
            ))
            .method(Method::GET)
            .body(Full::<Bytes>::new(Bytes::new()))?;

        let res = self.inner.ready().await?.call(req).await?;
        event!(Level::DEBUG, "Response status: {}", res.status());

        let body = res.collect().await?.aggregate();

        let json = serde_json::from_reader(body.reader()).map(|res: ApiResponse| res.data);
        match json {
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

        let req = Request::builder()
            .uri(format!(
                "/systems/{system_symbol}/waypoints/{waypoint_symbol}/market"
            ))
            .method(Method::GET)
            .body(Full::<Bytes>::new(Bytes::new()))?;

        let res = self.inner.ready().await?.call(req).await?;
        event!(Level::DEBUG, "Response status: {}", res.status());

        let body = res.collect().await?.aggregate();

        let json = serde_json::from_reader(body.reader()).map(|res: ApiResponse| res.data);
        match json {
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

        let req = Request::builder()
            .uri(format!(
                "/systems/{system_symbol}/waypoints/{waypoint_symbol}/shipyard"
            ))
            .method(Method::GET)
            .body(Full::<Bytes>::new(Bytes::new()))?;

        let res = self.inner.ready().await?.call(req).await?;
        event!(Level::DEBUG, "Response status: {}", res.status());

        let body = res.collect().await?.aggregate();

        let json = serde_json::from_reader(body.reader()).map(|res: ApiResponse| res.data);
        match json {
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

        let req = Request::builder()
            .uri(format!(
                "/systems/{system_symbol}/waypoints/{waypoint_symbol}/jump-gate"
            ))
            .method(Method::GET)
            .body(Full::<Bytes>::new(Bytes::new()))?;

        let res = self.inner.ready().await?.call(req).await?;
        event!(Level::DEBUG, "Response status: {}", res.status());

        let body = res.collect().await?.aggregate();

        let json = serde_json::from_reader(body.reader()).map(|res: ApiResponse| res.data);
        match json {
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

        let req = Request::builder()
            .uri(format!(
                "/systems/{system_symbol}/waypoints/{waypoint_symbol}/construction"
            ))
            .method(Method::GET)
            .body(Full::<Bytes>::new(Bytes::new()))?;

        let res = self.inner.ready().await?.call(req).await?;
        event!(Level::DEBUG, "Response status: {}", res.status());

        let body = res.collect().await?.aggregate();

        let json = serde_json::from_reader(body.reader()).map(|res: ApiResponse| res.data);
        match json {
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

        let req = Request::builder()
            .uri(format!("/agents?limit={limit}&page={page}"))
            .method(Method::GET)
            .body(Full::<Bytes>::new(Bytes::new()))?;

        let res = self.inner.ready().await?.call(req).await?;
        event!(Level::DEBUG, "Response status: {}", res.status());

        let body = res.collect().await?.aggregate();

        let json = serde_json::from_reader(body.reader());
        match json {
            Err(e) => Err(anyhow!(e)),
            Ok(ApiResponse {
                data,
                meta: Some(meta),
            }) => match data {
                ApiResponseData::ListAgents(agents) => Ok((agents, meta)),
                _ => Err(anyhow!("Unexpected response data: {data:?}")),
            },
            Ok(ApiResponse { meta: None, .. }) => Err(anyhow!("Meta field missing in response")),
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

        let req = Request::builder()
            .uri(format!("/factions?limit={limit}&page={page}"))
            .method(Method::GET)
            .body(Full::<Bytes>::new(Bytes::new()))?;

        let res = self.inner.ready().await?.call(req).await?;
        event!(Level::DEBUG, "Response status: {}", res.status());

        let body = res.collect().await?.aggregate();

        let json = serde_json::from_reader(body.reader());
        match json {
            Err(e) => Err(anyhow!(e)),
            Ok(ApiResponse {
                data,
                meta: Some(meta),
            }) => match data {
                ApiResponseData::ListFactions(factions) => Ok((factions, meta)),
                _ => Err(anyhow!("Unexpected response data: {data:?}")),
            },
            Ok(ApiResponse { meta: None, .. }) => Err(anyhow!("Meta field missing in response")),
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

        let req = Request::builder()
            .uri(format!("/systems?limit={limit}&page={page}"))
            .method(Method::GET)
            .body(Full::<Bytes>::new(Bytes::new()))?;

        let res = self.inner.ready().await?.call(req).await?;
        event!(Level::DEBUG, "Response status: {}", res.status());

        let body = res.collect().await?.aggregate();

        let json = serde_json::from_reader(body.reader());
        match json {
            Err(e) => Err(anyhow!(e)),
            Ok(ApiResponse {
                data,
                meta: Some(meta),
            }) => match data {
                ApiResponseData::ListSystems(systems) => Ok((systems, meta)),
                _ => Err(anyhow!("Unexpected response data: {data:?}")),
            },
            Ok(ApiResponse { meta: None, .. }) => Err(anyhow!("Meta field missing in response")),
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

        let req = Request::builder()
            .uri(format!("/systems/{system_symbol}/waypoints?limit={limit}&page={page}&type={waypoint_type}{traits}"))
            .method(Method::GET)
            .body(Full::<Bytes>::new(Bytes::new()))?;

        let res = self.inner.ready().await?.call(req).await?;
        event!(Level::DEBUG, "Response status: {}", res.status());

        let body = res.collect().await?.aggregate();

        let json = serde_json::from_reader(body.reader());
        match json {
            Err(e) => Err(anyhow!(e)),
            Ok(ApiResponse {
                data,
                meta: Some(meta),
            }) => match data {
                ApiResponseData::ListWaypoints(waypoints) => Ok((waypoints, meta)),
                _ => Err(anyhow!("Unexpected response data: {data:?}")),
            },
            Ok(ApiResponse { meta: None, .. }) => Err(anyhow!("Meta field missing in response")),
        }
    }

    #[instrument(level = Level::DEBUG, skip(self))]
    pub async fn get_agent(&mut self) -> Result<Agent, anyhow::Error> {
        let req = Request::builder()
            .uri("/my/agent")
            .method(Method::GET)
            .body(Full::<Bytes>::new(Bytes::new()))?;

        let res = self.inner.ready().await?.call(req).await?;
        event!(Level::DEBUG, "Response status: {}", res.status());

        let body = res.collect().await?.aggregate();

        let json = serde_json::from_reader(body.reader()).map(|res: ApiResponse| res.data);
        match json {
            Err(e) => Err(anyhow!(e)),
            Ok(ApiResponseData::GetAgent(agent)) => Ok(agent),
            Ok(d) => Err(anyhow!("Unexpected response data: {d:?}")),
        }
    }

    #[instrument(level = Level::DEBUG, skip(self))]
    pub async fn list_contracts(
        &mut self,
        limit: Option<u64>,
        page: Option<u64>,
    ) -> Result<(Vec<Contract>, Meta), anyhow::Error> {
        let limit = limit.unwrap_or(10);
        let page = page.unwrap_or(1);

        let req = Request::builder()
            .uri(format!("/my/contracts?limit={limit}&page={page}"))
            .method(Method::GET)
            .body(Full::<Bytes>::new(Bytes::new()))?;

        let res = self.inner.ready().await?.call(req).await?;
        event!(Level::DEBUG, "Response status: {}", res.status());

        let body = res.collect().await?.aggregate();

        let json = serde_json::from_reader(body.reader());
        match json {
            Err(e) => Err(anyhow!(e)),
            Ok(ApiResponse {
                data,
                meta: Some(meta),
            }) => match data {
                ApiResponseData::ListContracts(contracts) => Ok((contracts, meta)),
                _ => Err(anyhow!("Unexpected response data: {data:?}")),
            },
            Ok(ApiResponse { meta: None, .. }) => Err(anyhow!("Meta field missing in response")),
        }
    }

    #[instrument(level = Level::DEBUG, skip(self))]
    pub async fn get_contract(&mut self, contract_id: String) -> Result<Contract, anyhow::Error> {
        let req = Request::builder()
            .uri(format!("/my/contracts/{contract_id}"))
            .method(Method::GET)
            .body(Full::<Bytes>::new(Bytes::new()))?;

        let res = self.inner.ready().await?.call(req).await?;
        event!(Level::DEBUG, "Response status: {}", res.status());

        let body = res.collect().await?.aggregate();

        let json = serde_json::from_reader(body.reader()).map(|res: ApiResponse| res.data);
        match json {
            Err(e) => Err(anyhow!(e)),
            Ok(ApiResponseData::GetContract(contract)) => Ok(contract),
            Ok(d) => Err(anyhow!("Unexpected response data: {d:?}")),
        }
    }

    #[instrument(level = Level::DEBUG, skip(self))]
    pub async fn accept_contract(
        &mut self,
        contract_id: String,
    ) -> Result<(Agent, Contract), anyhow::Error> {
        let req = Request::builder()
            .uri(format!("/my/contracts/{contract_id}/accept"))
            .method(Method::POST)
            .body(Full::<Bytes>::new(Bytes::new()))?;

        let res = self.inner.ready().await?.call(req).await?;
        event!(Level::DEBUG, "Response status: {}", res.status());

        let body = res.collect().await?.aggregate();

        let json = serde_json::from_reader(body.reader()).map(|res: ApiResponse| res.data);
        match json {
            Err(e) => Err(anyhow!(e)),
            Ok(ApiResponseData::UpdateContract {
                agent: Some(agent),
                contract,
                ..
            }) => Ok((agent, contract)),
            Ok(d) => Err(anyhow!("Unexpected response data: {d:?}")),
        }
    }

    #[instrument(level = Level::DEBUG, skip(self))]
    pub async fn fulfill_contract(
        &mut self,
        contract_id: String,
    ) -> Result<(Agent, Contract), anyhow::Error> {
        let req = Request::builder()
            .uri(format!("/my/contracts/{contract_id}/fulfill"))
            .method(Method::POST)
            .body(Full::<Bytes>::new(Bytes::new()))?;

        let res = self.inner.ready().await?.call(req).await?;
        event!(Level::DEBUG, "Response status: {}", res.status());

        let body = res.collect().await?.aggregate();

        let json = serde_json::from_reader(body.reader()).map(|res: ApiResponse| res.data);
        match json {
            Err(e) => Err(anyhow!(e)),
            Ok(ApiResponseData::UpdateContract {
                agent: Some(agent),
                contract,
                ..
            }) => Ok((agent, contract)),
            Ok(d) => Err(anyhow!("Unexpected response data: {d:?}")),
        }
    }

    #[instrument(level = Level::DEBUG, skip(self))]
    pub async fn deliver_contract(
        &mut self,
        contract_id: String,
        ship: String,
        cargo: TradeSymbol,
        amount: u64,
    ) -> Result<(ShipCargo, Contract), anyhow::Error> {
        let delivery = DeliverCargo {
            ship_symbol: ship,
            trade_symbol: cargo,
            units: amount,
        };

        let body = match serde_json::to_vec(&delivery) {
            Ok(body) => body,
            Err(e) => return Err(anyhow!(e)),
        };

        let req = Request::builder()
            .uri(format!("/my/contracts/{contract_id}/deliver"))
            .method(Method::POST)
            .header(header::CONTENT_TYPE, "application/json")
            .body(Full::<Bytes>::new(body.into()))?;

        let res = self.inner.ready().await?.call(req).await?;
        event!(Level::DEBUG, "Response status: {}", res.status());

        let body = res.collect().await?.aggregate();

        let json = serde_json::from_reader(body.reader()).map(|res: ApiResponse| res.data);
        match json {
            Err(e) => Err(anyhow!(e)),
            Ok(ApiResponseData::UpdateContract {
                contract,
                cargo: Some(cargo),
                ..
            }) => Ok((cargo, contract)),
            Ok(d) => Err(anyhow!("Unexpected response data: {d:?}")),
        }
    }

    #[instrument(level = Level::DEBUG, skip(self))]
    pub async fn get_faction(&mut self, faction: FactionSymbol) -> Result<Faction, anyhow::Error> {
        let req = Request::builder()
            .uri(format!("/factions/{faction}"))
            .method(Method::GET)
            .body(Full::<Bytes>::new(Bytes::new()))?;

        let res = self.inner.ready().await?.call(req).await?;
        event!(Level::DEBUG, "Response status: {}", res.status());

        let body = res.collect().await?.aggregate();

        let json = serde_json::from_reader(body.reader()).map(|res: ApiResponse| res.data);
        match json {
            Err(e) => Err(anyhow!(e)),
            Ok(ApiResponseData::GetFaction(faction)) => Ok(faction),
            Ok(d) => Err(anyhow!("Unexpected response data: {d:?}")),
        }
    }

    #[instrument(level = Level::DEBUG, skip(self))]
    pub async fn supply_construction(
        &mut self,
        waypoint: String,
        ship: String,
        cargo: TradeSymbol,
        amount: u64,
    ) -> Result<(ShipCargo, Construction), anyhow::Error> {
        let (system, _) = waypoint.split_at(
            waypoint
                .rfind('-')
                .ok_or_else(|| anyhow!("Invalid waypoint symbol"))?,
        );
        let delivery = DeliverCargo {
            ship_symbol: ship,
            trade_symbol: cargo,
            units: amount,
        };

        let body = match serde_json::to_vec(&delivery) {
            Ok(body) => body,
            Err(e) => return Err(anyhow!(e)),
        };

        let req = Request::builder()
            .uri(format!(
                "/systems/{system}/waypoints/{waypoint}/construction/supply"
            ))
            .method(Method::POST)
            .header(header::CONTENT_TYPE, "application/json")
            .body(Full::<Bytes>::new(body.into()))?;

        let res = self.inner.ready().await?.call(req).await?;
        event!(Level::DEBUG, "Response status: {}", res.status());

        let body = res.collect().await?.aggregate();

        let json = serde_json::from_reader(body.reader()).map(|res: ApiResponse| res.data);
        match json {
            Err(e) => Err(anyhow!(e)),
            Ok(ApiResponseData::UpdateConstruction {
                construction,
                cargo,
            }) => Ok((cargo, construction)),
            Ok(d) => Err(anyhow!("Unexpected response data: {d:?}")),
        }
    }

    #[instrument(level = Level::DEBUG, skip(self))]
    pub async fn list_ships(
        &mut self,
        limit: Option<u64>,
        page: Option<u64>,
    ) -> Result<(Vec<Ship>, Meta), anyhow::Error> {
        let limit = limit.unwrap_or(10);
        let page = page.unwrap_or(1);

        let req = Request::builder()
            .uri(format!("/my/ships?limit={limit}&page={page}"))
            .method(Method::GET)
            .body(Full::<Bytes>::new(Bytes::new()))?;

        let res = self.inner.ready().await?.call(req).await?;
        event!(Level::DEBUG, "Response status: {}", res.status());

        let body = res.collect().await?.aggregate();

        let json = serde_json::from_reader(body.reader());
        match json {
            Err(e) => Err(anyhow!(e)),
            Ok(ApiResponse {
                data,
                meta: Some(meta),
            }) => match data {
                ApiResponseData::ListShips(ships) => Ok((ships, meta)),
                _ => Err(anyhow!("Unexpected response data: {data:?}")),
            },
            Ok(ApiResponse { meta: None, .. }) => Err(anyhow!("Meta field missing in response")),
        }
    }

    #[instrument(level = Level::DEBUG, skip(self))]
    pub async fn get_ship(&mut self, ship: String) -> Result<Box<Ship>, anyhow::Error> {
        let req = Request::builder()
            .uri(format!("/my/ships/{ship}"))
            .method(Method::GET)
            .body(Full::<Bytes>::new(Bytes::new()))?;

        let res = self.inner.ready().await?.call(req).await?;
        event!(Level::DEBUG, "Response status: {}", res.status());

        let body = res.collect().await?.aggregate();

        let json = serde_json::from_reader(body.reader()).map(|res: ApiResponse| res.data);
        match json {
            Err(e) => Err(anyhow!(e)),
            Ok(ApiResponseData::GetShip(ship)) => Ok(ship),
            Ok(d) => Err(anyhow!("Unexpected response data: {d:?}")),
        }
    }

    #[instrument(level = Level::DEBUG, skip(self))]
    pub async fn get_ship_cargo(&mut self, ship: String) -> Result<ShipCargo, anyhow::Error> {
        let req = Request::builder()
            .uri(format!("/my/ships/{ship}/cargo"))
            .method(Method::GET)
            .body(Full::<Bytes>::new(Bytes::new()))?;

        let res = self.inner.ready().await?.call(req).await?;
        event!(Level::DEBUG, "Response status: {}", res.status());

        let body = res.collect().await?.aggregate();

        let json = serde_json::from_reader(body.reader()).map(|res: ApiResponse| res.data);
        match json {
            Err(e) => Err(anyhow!(e)),
            Ok(ApiResponseData::GetCargo(cargo)) => Ok(cargo),
            Ok(d) => Err(anyhow!("Unexpected response data: {d:?}")),
        }
    }

    #[instrument(level = Level::DEBUG, skip(self))]
    pub async fn get_ship_nav(&mut self, ship: String) -> Result<ShipNav, anyhow::Error> {
        let req = Request::builder()
            .uri(format!("/my/ships/{ship}/nav"))
            .method(Method::GET)
            .body(Full::<Bytes>::new(Bytes::new()))?;

        let res = self.inner.ready().await?.call(req).await?;
        event!(Level::DEBUG, "Response status: {}", res.status());

        let body = res.collect().await?.aggregate();

        let json = serde_json::from_reader(body.reader()).map(|res: ApiResponse| res.data);
        match json {
            Err(e) => Err(anyhow!(e)),
            Ok(ApiResponseData::GetNav(nav)) => Ok(nav),
            Ok(d) => Err(anyhow!("Unexpected response data: {d:?}")),
        }
    }

    #[instrument(level = Level::DEBUG, skip(self))]
    pub async fn get_ship_mounts(&mut self, ship: String) -> Result<Vec<ShipMount>, anyhow::Error> {
        let req = Request::builder()
            .uri(format!("/my/ships/{ship}/mounts"))
            .method(Method::GET)
            .body(Full::<Bytes>::new(Bytes::new()))?;

        let res = self.inner.ready().await?.call(req).await?;
        event!(Level::DEBUG, "Response status: {}", res.status());

        let body = res.collect().await?.aggregate();

        let json = serde_json::from_reader(body.reader()).map(|res: ApiResponse| res.data);
        match json {
            Err(e) => Err(anyhow!(e)),
            Ok(ApiResponseData::GetMounts(mounts)) => Ok(mounts),
            Ok(d) => Err(anyhow!("Unexpected response data: {d:?}")),
        }
    }

    #[instrument(level = Level::DEBUG, skip(self))]
    pub async fn get_scrap_ship(&mut self, ship: String) -> Result<ShipTransaction, anyhow::Error> {
        let req = Request::builder()
            .uri(format!("/my/ships/{ship}/scrap"))
            .method(Method::GET)
            .body(Full::<Bytes>::new(Bytes::new()))?;

        let res = self.inner.ready().await?.call(req).await?;
        event!(Level::DEBUG, "Response status: {}", res.status());

        let body = res.collect().await?.aggregate();

        let json = serde_json::from_reader(body.reader()).map(|res: ApiResponse| res.data);
        match json {
            Err(e) => Err(anyhow!(e)),
            Ok(ApiResponseData::GetShipTransaction { transaction }) => Ok(transaction),
            Ok(d) => Err(anyhow!("Unexpected response data: {d:?}")),
        }
    }

    #[instrument(level = Level::DEBUG, skip(self))]
    pub async fn get_repair_ship(
        &mut self,
        ship: String,
    ) -> Result<ShipTransaction, anyhow::Error> {
        let req = Request::builder()
            .uri(format!("/my/ships/{ship}/repair"))
            .method(Method::GET)
            .body(Full::<Bytes>::new(Bytes::new()))?;

        let res = self.inner.ready().await?.call(req).await?;
        event!(Level::DEBUG, "Response status: {}", res.status());

        let body = res.collect().await?.aggregate();

        let json = serde_json::from_reader(body.reader()).map(|res: ApiResponse| res.data);
        match json {
            Err(e) => Err(anyhow!(e)),
            Ok(ApiResponseData::GetShipTransaction { transaction }) => Ok(transaction),
            Ok(d) => Err(anyhow!("Unexpected response data: {d:?}")),
        }
    }

    #[instrument(level = Level::DEBUG, skip(self))]
    pub async fn get_ship_cooldown(
        &mut self,
        ship: String,
    ) -> Result<Option<Cooldown>, anyhow::Error> {
        let req = Request::builder()
            .uri(format!("/my/ships/{ship}/cooldown"))
            .method(Method::GET)
            .body(Full::<Bytes>::new(Bytes::new()))?;

        let res = self.inner.ready().await?.call(req).await?;
        event!(Level::DEBUG, "Response status: {}", res.status());

        // This endpoint is a bit of an outlier in that it
        // either returns data with a 200 OK, or it returns
        // 204 NO CONTENT without data if there is no cooldown.
        if res.status() == StatusCode::NO_CONTENT {
            return Ok(None);
        }

        let body = res.collect().await?.aggregate();

        let json = serde_json::from_reader(body.reader()).map(|res: ApiResponse| res.data);
        match json {
            Err(e) => Err(anyhow!(e)),
            Ok(ApiResponseData::GetCooldown(cooldown)) => Ok(Some(cooldown)),
            Ok(d) => Err(anyhow!("Unexpected response data: {d:?}")),
        }
    }

    #[instrument(level = Level::DEBUG, skip(self))]
    pub async fn purchase_ship(
        &mut self,
        ship_type: ShipType,
        waypoint: String,
    ) -> Result<(Agent, Box<Ship>, ShipyardTransaction), anyhow::Error> {
        let purchase = ShipPurchase {
            ship_type,
            waypoint_symbol: waypoint,
        };

        let body = match serde_json::to_vec(&purchase) {
            Ok(body) => body,
            Err(e) => return Err(anyhow!(e)),
        };

        let req = Request::builder()
            .uri("/my/ships")
            .method(Method::POST)
            .header(header::CONTENT_TYPE, "application/json")
            .body(Full::<Bytes>::new(body.into()))?;

        let res = self.inner.ready().await?.call(req).await?;
        event!(Level::DEBUG, "Response status: {}", res.status());

        let body = res.collect().await?.aggregate();

        let json = serde_json::from_reader(body.reader()).map(|res: ApiResponse| res.data);
        match json {
            Err(e) => Err(anyhow!(e)),
            Ok(ApiResponseData::ShipPurchase {
                agent,
                ship,
                transaction,
            }) => Ok((agent, ship, transaction)),
            Ok(d) => Err(anyhow!("Unexpected response data: {d:?}")),
        }
    }

    #[instrument(level = Level::DEBUG, skip(self))]
    pub async fn orbit_ship(&mut self, ship: String) -> Result<ShipNav, anyhow::Error> {
        let req = Request::builder()
            .uri(format!("/my/ships/{ship}/orbit"))
            .method(Method::POST)
            .body(Full::<Bytes>::new(Bytes::new()))?;

        let res = self.inner.ready().await?.call(req).await?;
        event!(Level::DEBUG, "Response status: {}", res.status());

        let body = res.collect().await?.aggregate();

        let json = serde_json::from_reader(body.reader()).map(|res: ApiResponse| res.data);
        match json {
            Err(e) => Err(anyhow!(e)),
            Ok(ApiResponseData::GetNav(nav)) => Ok(nav),
            Ok(d) => Err(anyhow!("Unexpected response data: {d:?}")),
        }
    }

    #[instrument(level = Level::DEBUG, skip(self))]
    pub async fn ship_refine(
        &mut self,
        ship: String,
        produce: TradeSymbol,
    ) -> Result<
        (
            ShipCargo,
            Cooldown,
            Vec<TradeGoodAmount>,
            Vec<TradeGoodAmount>,
        ),
        anyhow::Error,
    > {
        let produce = Produce { produce };

        let body = match serde_json::to_vec(&produce) {
            Ok(body) => body,
            Err(e) => return Err(anyhow!(e)),
        };

        let req = Request::builder()
            .uri(format!("/my/ships/{ship}/refine"))
            .method(Method::POST)
            .header(header::CONTENT_TYPE, "application/json")
            .body(Full::<Bytes>::new(body.into()))?;

        let res = self.inner.ready().await?.call(req).await?;
        event!(Level::DEBUG, "Response status: {}", res.status());

        let body = res.collect().await?.aggregate();

        let json = serde_json::from_reader(body.reader()).map(|res: ApiResponse| res.data);
        match json {
            Err(e) => Err(anyhow!(e)),
            Ok(ApiResponseData::Refine {
                cargo,
                cooldown,
                produced,
                consumed,
            }) => Ok((cargo, cooldown, produced, consumed)),
            Ok(d) => Err(anyhow!("Unexpected response data: {d:?}")),
        }
    }

    #[instrument(level = Level::DEBUG, skip(self))]
    pub async fn create_chart(&mut self, ship: String) -> Result<(Chart, Waypoint), anyhow::Error> {
        let req = Request::builder()
            .uri(format!("/my/ships/{ship}/chart"))
            .method(Method::POST)
            .body(Full::<Bytes>::new(Bytes::new()))?;

        let res = self.inner.ready().await?.call(req).await?;
        event!(Level::DEBUG, "Response status: {}", res.status());

        let body = res.collect().await?.aggregate();

        let json = serde_json::from_reader(body.reader()).map(|res: ApiResponse| res.data);
        match json {
            Err(e) => Err(anyhow!(e)),
            Ok(ApiResponseData::CreateChart { chart, waypoint }) => Ok((chart, waypoint)),
            Ok(d) => Err(anyhow!("Unexpected response data: {d:?}")),
        }
    }

    #[instrument(level = Level::DEBUG, skip(self))]
    pub async fn dock_ship(&mut self, ship: String) -> Result<ShipNav, anyhow::Error> {
        let req = Request::builder()
            .uri(format!("/my/ships/{ship}/dock"))
            .method(Method::POST)
            .body(Full::<Bytes>::new(Bytes::new()))?;

        let res = self.inner.ready().await?.call(req).await?;
        event!(Level::DEBUG, "Response status: {}", res.status());

        let body = res.collect().await?.aggregate();

        let json = serde_json::from_reader(body.reader()).map(|res: ApiResponse| res.data);
        match json {
            Err(e) => Err(anyhow!(e)),
            Ok(ApiResponseData::GetNav(nav)) => Ok(nav),
            Ok(d) => Err(anyhow!("Unexpected response data: {d:?}")),
        }
    }

    #[instrument(level = Level::DEBUG, skip(self))]
    pub async fn create_survey(
        &mut self,
        ship: String,
    ) -> Result<(Cooldown, Vec<Survey>), anyhow::Error> {
        let req = Request::builder()
            .uri(format!("/my/ships/{ship}/survey"))
            .method(Method::POST)
            .body(Full::<Bytes>::new(Bytes::new()))?;

        let res = self.inner.ready().await?.call(req).await?;
        event!(Level::DEBUG, "Response status: {}", res.status());

        let body = res.collect().await?.aggregate();

        let json = serde_json::from_reader(body.reader()).map(|res: ApiResponse| res.data);
        match json {
            Err(e) => Err(anyhow!(e)),
            Ok(ApiResponseData::CreateSurvey { cooldown, surveys }) => Ok((cooldown, surveys)),
            Ok(d) => Err(anyhow!("Unexpected response data: {d:?}")),
        }
    }

    #[instrument(level = Level::DEBUG, skip(self))]
    pub async fn extract_resources(
        &mut self,
        ship: String,
    ) -> Result<(Cooldown, Extraction, ShipCargo, Vec<ShipConditionEvent>), anyhow::Error> {
        let req = Request::builder()
            .uri(format!("/my/ships/{ship}/extract"))
            .method(Method::POST)
            .body(Full::<Bytes>::new(Bytes::new()))?;

        let res = self.inner.ready().await?.call(req).await?;
        event!(Level::DEBUG, "Response status: {}", res.status());

        let body = res.collect().await?.aggregate();

        let json = serde_json::from_reader(body.reader()).map(|res: ApiResponse| res.data);
        match json {
            Err(e) => Err(anyhow!(e)),
            Ok(ApiResponseData::ExtractResources {
                cooldown,
                extraction,
                cargo,
                events,
            }) => Ok((cooldown, extraction, cargo, events)),
            Ok(d) => Err(anyhow!("Unexpected response data: {d:?}")),
        }
    }

    #[instrument(level = Level::DEBUG, skip(self))]
    pub async fn siphon_resources(
        &mut self,
        ship: String,
    ) -> Result<(Cooldown, Siphon, ShipCargo, Vec<ShipConditionEvent>), anyhow::Error> {
        let req = Request::builder()
            .uri(format!("/my/ships/{ship}/siphon"))
            .method(Method::POST)
            .body(Full::<Bytes>::new(Bytes::new()))?;

        let res = self.inner.ready().await?.call(req).await?;
        event!(Level::DEBUG, "Response status: {}", res.status());

        let body = res.collect().await?.aggregate();

        let json = serde_json::from_reader(body.reader()).map(|res: ApiResponse| res.data);
        match json {
            Err(e) => Err(anyhow!(e)),
            Ok(ApiResponseData::SiphonResources {
                cooldown,
                siphon,
                cargo,
                events,
            }) => Ok((cooldown, siphon, cargo, events)),
            Ok(d) => Err(anyhow!("Unexpected response data: {d:?}")),
        }
    }

    #[instrument(level = Level::DEBUG, skip(self))]
    pub async fn extract_resources_with_survey(
        &mut self,
        ship: String,
        survey: Survey,
    ) -> Result<(Cooldown, Extraction, ShipCargo, Vec<ShipConditionEvent>), anyhow::Error> {
        let body = match serde_json::to_vec(&survey) {
            Ok(body) => body,
            Err(e) => return Err(anyhow!(e)),
        };

        let req = Request::builder()
            .uri(format!("/my/ships/{ship}/extract/survey"))
            .method(Method::POST)
            .header(header::CONTENT_TYPE, "application/json")
            .body(Full::<Bytes>::new(body.into()))?;

        let res = self.inner.ready().await?.call(req).await?;
        event!(Level::DEBUG, "Response status: {}", res.status());

        let body = res.collect().await?.aggregate();

        let json = serde_json::from_reader(body.reader()).map(|res: ApiResponse| res.data);
        match json {
            Err(e) => Err(anyhow!(e)),
            Ok(ApiResponseData::ExtractResources {
                cooldown,
                extraction,
                cargo,
                events,
            }) => Ok((cooldown, extraction, cargo, events)),
            Ok(d) => Err(anyhow!("Unexpected response data: {d:?}")),
        }
    }

    #[instrument(level = Level::DEBUG, skip(self))]
    pub async fn jettison_cargo(
        &mut self,
        ship: String,
        cargo: TradeGoodAmount,
    ) -> Result<ShipCargo, anyhow::Error> {
        let body = match serde_json::to_vec(&cargo) {
            Ok(body) => body,
            Err(e) => return Err(anyhow!(e)),
        };

        let req = Request::builder()
            .uri(format!("/my/ships/{ship}/jettison"))
            .method(Method::POST)
            .header(header::CONTENT_TYPE, "application/json")
            .body(Full::<Bytes>::new(body.into()))?;

        let res = self.inner.ready().await?.call(req).await?;
        event!(Level::DEBUG, "Response status: {}", res.status());

        let body = res.collect().await?.aggregate();

        let json = serde_json::from_reader(body.reader()).map(|res: ApiResponse| res.data);
        match json {
            Err(e) => Err(anyhow!(e)),
            Ok(ApiResponseData::GetCargo(cargo)) => Ok(cargo),
            Ok(d) => Err(anyhow!("Unexpected response data: {d:?}")),
        }
    }

    #[instrument(level = Level::DEBUG, skip(self))]
    pub async fn jump_ship(
        &mut self,
        ship: String,
        destination: String,
    ) -> Result<(Box<ShipNav>, Cooldown, MarketTransaction, Agent), anyhow::Error> {
        let destination = Destination {
            waypoint_symbol: destination,
        };

        let body = match serde_json::to_vec(&destination) {
            Ok(body) => body,
            Err(e) => return Err(anyhow!(e)),
        };

        let req = Request::builder()
            .uri(format!("/my/ships/{ship}/jump"))
            .method(Method::POST)
            .header(header::CONTENT_TYPE, "application/json")
            .body(Full::<Bytes>::new(body.into()))?;

        let res = self.inner.ready().await?.call(req).await?;
        event!(Level::DEBUG, "Response status: {}", res.status());

        let body = res.collect().await?.aggregate();

        let json = serde_json::from_reader(body.reader()).map(|res: ApiResponse| res.data);
        match json {
            Err(e) => Err(anyhow!(e)),
            Ok(ApiResponseData::JumpShip {
                nav,
                cooldown,
                transaction,
                agent,
            }) => Ok((nav, cooldown, transaction, agent)),
            Ok(d) => Err(anyhow!("Unexpected response data: {d:?}")),
        }
    }

    #[instrument(level = Level::DEBUG, skip(self))]
    pub async fn navigate_ship(
        &mut self,
        ship: String,
        destination: String,
    ) -> Result<(ShipFuel, ShipNav, Vec<ShipConditionEvent>), anyhow::Error> {
        let destination = Destination {
            waypoint_symbol: destination,
        };

        let body = match serde_json::to_vec(&destination) {
            Ok(body) => body,
            Err(e) => return Err(anyhow!(e)),
        };

        let req = Request::builder()
            .uri(format!("/my/ships/{ship}/navigate"))
            .method(Method::POST)
            .header(header::CONTENT_TYPE, "application/json")
            .body(Full::<Bytes>::new(body.into()))?;

        let res = self.inner.ready().await?.call(req).await?;
        event!(Level::DEBUG, "Response status: {}", res.status());

        let body = res.collect().await?.aggregate();

        let json = serde_json::from_reader(body.reader()).map(|res: ApiResponse| res.data);
        match json {
            Err(e) => Err(anyhow!(e)),
            Ok(ApiResponseData::NavigateShip { fuel, nav, events }) => Ok((fuel, nav, events)),
            Ok(d) => Err(anyhow!("Unexpected response data: {d:?}")),
        }
    }
}
