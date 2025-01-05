use std::{
    ops::{Deref, DerefMut},
    time::Duration,
};

use anyhow::anyhow;
use http_body_util::{BodyExt, Full};
use hyper::{
    body::{Buf, Bytes},
    header, Method, Request,
};
use tower::{Service, ServiceBuilder, ServiceExt};
use tower_http::auth::{AddAuthorization, AddAuthorizationLayer};
use tracing::{event, instrument, Level};

use crate::model::{
    Agent, ApiResponse, ApiResponseData, ApiStatus, Construction, Contract, DeliverCargo, Faction,
    FactionSymbol, JumpGate, Market, Meta, RegisterAgent, RegisterAgentSuccess, Ship, ShipCargo,
    ShipNav, Shipyard, System, TradeSymbol, Waypoint, WaypointTraitSymbol, WaypointType,
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
}
