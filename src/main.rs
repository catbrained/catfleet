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

    let mut client = client::Client::new().await.unwrap();

    match client.get_status().await {
        Ok(status) => event!(Level::INFO, status.status),
        Err(e) => event!(Level::ERROR, %e),
    }
    // match client.get_public_agent("CATBRAINED".to_string()).await {
    //     Ok(agent) => event!(Level::INFO, ?agent),
    //     Err(e) => event!(Level::ERROR, %e),
    // }
    // match client.get_system("X1-AT80".to_string()).await {
    //     Ok(system) => event!(Level::INFO, ?system),
    //     Err(e) => event!(Level::ERROR, %e),
    // }
    // match client.get_waypoint("X1-AT80-A1".to_string()).await {
    //     Ok(waypoint) => event!(Level::INFO, ?waypoint),
    //     Err(e) => event!(Level::ERROR, %e),
    // }
    // match client.get_market("X1-AT80-A1".to_string()).await {
    //     Ok(market) => event!(Level::INFO, ?market),
    //     Err(e) => event!(Level::ERROR, %e),
    // }
    // match client.get_shipyard("X1-AT80-A2".to_string()).await {
    //     Ok(shipyard) => event!(Level::INFO, ?shipyard),
    //     Err(e) => event!(Level::ERROR, %e),
    // }
    // match client.get_jumpgate("X1-AT80-I55".to_string()).await {
    //     Ok(jumpgate) => event!(Level::INFO, ?jumpgate),
    //     Err(e) => event!(Level::ERROR, %e),
    // }
    // match client
    //     .get_construction_site("X1-AT80-I55".to_string())
    //     .await
    // {
    //     Ok(construction_site) => event!(Level::INFO, ?construction_site),
    //     Err(e) => event!(Level::ERROR, %e),
    // }
    // match client.list_agents(Some(20), Some(2)).await {
    //     Ok((agents, meta)) => event!(Level::INFO, ?meta, ?agents),
    //     Err(e) => event!(Level::ERROR, %e),
    // }
    // match client.list_factions(Some(20), Some(1)).await {
    //     Ok((factions, meta)) => event!(Level::INFO, ?meta, ?factions),
    //     Err(e) => event!(Level::ERROR, %e),
    // }
    // match client.list_systems(Some(20), Some(1)).await {
    //     Ok((systems, meta)) => event!(Level::INFO, ?meta, ?systems),
    //     Err(e) => event!(Level::ERROR, %e),
    // }
    // match client
    //     .list_waypoints(
    //         "X1-AT80".to_string(),
    //         Some(20),
    //         Some(1),
    //         Some(vec![
    //             model::WaypointTraitSymbol::Marketplace,
    //             model::WaypointTraitSymbol::Industrial,
    //         ]),
    //         Some(model::WaypointType::Planet),
    //     )
    //     .await
    // {
    //     Ok((waypoints, meta)) => event!(Level::INFO, ?meta, ?waypoints),
    //     Err(e) => event!(Level::ERROR, %e),
    // }
    // match client
    //     .register_new_agent(
    //         model::FactionSymbol::Cosmic,
    //         "uheffulaiykck".to_string(),
    //         None,
    //     )
    //     .await
    // {
    //     Ok(agent) => event!(Level::INFO, ?agent),
    //     Err(e) => event!(Level::ERROR, %e),
    // }
    match client.get_agent().await {
        Ok(agent) => event!(Level::INFO, ?agent),
        Err(e) => event!(Level::ERROR, %e),
    }
    // match client.list_contracts(Some(20), Some(1)).await {
    //     Ok((contracts, meta)) => event!(Level::INFO, ?meta, ?contracts),
    //     Err(e) => event!(Level::ERROR, %e),
    // }
    // match client
    //     .get_contract("cm54fmx7n9e3ws60c4lmtm00n".to_string())
    //     .await
    // {
    //     Ok(contract) => event!(Level::INFO, ?contract),
    //     Err(e) => event!(Level::ERROR, %e),
    // }
    // match client.get_faction(model::FactionSymbol::Cosmic).await {
    //     Ok(faction) => event!(Level::INFO, ?faction),
    //     Err(e) => event!(Level::ERROR, %e),
    // }
    // match client.list_ships(Some(20), Some(1)).await {
    //     Ok((ships, meta)) => event!(Level::INFO, ?meta, ?ships),
    //     Err(e) => event!(Level::ERROR, %e),
    // }
    // match client.get_ship("CATBRAINED-3".to_string()).await {
    //     Ok(ship) => event!(Level::INFO, ?ship),
    //     Err(e) => event!(Level::ERROR, %e),
    // }
    // match client.get_ship_cargo("CATBRAINED-3".to_string()).await {
    //     Ok(cargo) => event!(Level::INFO, ?cargo),
    //     Err(e) => event!(Level::ERROR, %e),
    // }
    match client.get_ship_nav("CATBRAINED-3".to_string()).await {
        Ok(nav) => event!(Level::INFO, ?nav),
        Err(e) => event!(Level::ERROR, %e),
    }
}
