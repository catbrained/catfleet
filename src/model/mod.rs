use std::fmt::Display;

use serde::{Deserialize, Serialize};

/// The activity level of a trade good.
/// If the good is an import, this represents how strong consumption is.
/// If the good is an export, this represents how strong the production is for the good.
/// When activity is strong, consumption or production is near maximum capacity.
/// When activity is weak, consumption or production is near minimum capacity.
#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ActivityLevel {
    Weak,
    Growing,
    Strong,
    Restricted,
}

/// Agent details.
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename = "agent", rename_all = "camelCase")]
pub struct Agent {
    /// Account ID that is tied to this agent. Only included on your own agent.
    /// >= 1 characters
    pub account_id: Option<String>,
    /// Symbol of the agent.
    /// >= 3 characters && <= 14 characters
    pub symbol: String,
    /// The headquarters of the agent.
    /// >= 1 characters
    pub headquarters: String,
    /// The number of credits the agent has available.
    /// Credits can be negative if funds have been overdrawn.
    pub credits: i64,
    /// The faction the agent started with.
    /// >= 1 characters
    pub starting_faction: String,
    /// How many ships are owned by the agent.
    pub ship_count: u64,
}

/// The chart of a system or waypoint, which makes the
/// location visible to other agents.
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename = "chart", rename_all = "camelCase")]
pub struct Chart {
    /// The symbol of the waypoint.
    /// >= 1 characters
    pub waypoint_symbol: Option<String>,
    /// The agent that submitted the chart for this waypoint.
    pub submitted_by: Option<String>,
    /// The time the chart for this waypoint was submitted.
    pub submitted_on: Option<String>, // TODO: This is supposed to be a "date-time". Figure out what Rust type this maps to.
}

// TODO: Figure out where this is used.
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename = "connectedSystem", rename_all = "camelCase")]
struct ConnectedSystem {
    /// The symbol of the system.
    /// >= 1 characters
    symbol: String,
    /// The sector of this system.
    /// >= 1 characters
    sector_symbol: String,
    /// The type of system.
    #[serde(rename = "type")]
    system_type: SystemType,
    /// The symbol of the faction that owns the connected jump gate in the system.
    faction_symbol: Option<String>,
    /// Position in the universe in the x axis.
    x: i64,
    /// Position in the universe in the y axis.
    y: i64,
    /// The distance of this system to the connected jump gate.
    distance: u64,
}

/// The type of system.
#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SystemType {
    NeutronStar,
    RedStar,
    OrangeStar,
    BlueStar,
    YoungStar,
    WhiteDwarf,
    BlackHole,
    Hypergiant,
    Nebula,
    Unstable,
}

/// The construction details of a waypoint.
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename = "construction", rename_all = "camelCase")]
pub struct Construction {
    /// The symbol of the waypoint.
    pub symbol: String,
    /// The materials required to construct the waypoint.
    pub materials: Vec<ConstructionMaterial>,
    /// Wether the waypoint has been constructed.
    pub is_complete: bool,
}

/// The details of the required construction materials
/// for a given waypoint under construction.
#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ConstructionMaterial {
    /// The good's symbol.
    pub trade_symbol: TradeSymbol,
    /// The number of units required.
    pub required: u64,
    /// The number of units fullfilled toward the required amount.
    pub fulfilled: u64,
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TradeSymbol {
    PreciousStones,
    QuartzSand,
    SiliconCrystals,
    AmmoniaIce,
    LiquidHydrogen,
    LiquidNitrogen,
    IceWater,
    ExoticMatter,
    AdvancedCircuitry,
    GravitonEmitters,
    Iron,
    IronOre,
    Copper,
    CopperOre,
    Aluminum,
    AluminumOre,
    Silver,
    SilverOre,
    Gold,
    GoldOre,
    Platinum,
    PlatinumOre,
    Diamonds,
    Uranite,
    UraniteOre,
    Meritium,
    MeritiumOre,
    Hydrocarbon,
    Antimatter,
    FabMats,
    Fertilizers,
    Fabrics,
    Food,
    Jewelry,
    Machinery,
    Firearms,
    AssaultRifles,
    MilitaryEquipment,
    Explosives,
    LabInstruments,
    Ammunition,
    Electronics,
    ShipPlating,
    ShipParts,
    Equipment,
    Fuel,
    Medicine,
    Drugs,
    Clothing,
    Microprocessors,
    Plastics,
    Polynucleotides,
    Biocomposites,
    QuantumStabilizers,
    Nanobots,
    AiMainframes,
    QuantumDrives,
    RoboticDrones,
    CyberImplants,
    GeneTherapeutics,
    NeuralChips,
    MoodRegulators,
    ViralAgents,
    MicroFusionGenerators,
    Supergrains,
    LaserRifles,
    Holographics,
    ShipSalvage,
    RelicTech,
    NovelLifeforms,
    BotanicalSpecimens,
    CulturalArtifacts,
    FrameProbe,
    FrameDrone,
    FrameInterceptor,
    FrameRacer,
    FrameFighter,
    FrameFrigate,
    FrameShuttle,
    FrameExplorer,
    FrameMiner,
    FrameLightFreighter,
    FrameHeavyFreighter,
    FrameTransport,
    FrameDestroyer,
    FrameCruiser,
    FrameCarrier,
    ReactorSolarI,
    ReactorFusionI,
    ReactorFissionI,
    ReactorChemicalI,
    ReactorAntimatterI,
    EngineImpulseDriveI,
    EngineIonDriveI,
    EngineIonDriveIi,
    EngineHyperDriveI,
    ModuleMineralProcessorI,
    ModuleGasProcessorI,
    ModuleCargoHoldI,
    ModuleCargoHoldIi,
    ModuleCargoHoldIii,
    ModuleCrewQuartersI,
    ModuleEnvoyQuartersI,
    ModulePassengerCabinI,
    ModuleMicroRefineryI,
    ModuleScienceLabI,
    ModuleJumpDriveI,
    ModuleJumpDriveIi,
    ModuleJumpDriveIii,
    ModuleWarpDriveI,
    ModuleWarpDriveIi,
    ModuleWarpDriveIii,
    ModuleShieldGeneratorI,
    ModuleShieldGeneratorIi,
    ModuleOreRefineryI,
    ModuleFuelRefineryI,
    MountGasSiphonI,
    MountGasSiphonIi,
    MountGasSiphonIii,
    MountSurveyorI,
    MountSurveyorIi,
    MountSurveyorIii,
    MountSensorArrayI,
    MountSensorArrayIi,
    MountSensorArrayIii,
    MountMiningLaserI,
    MountMiningLaserIi,
    MountMiningLaserIii,
    MountLaserCannonI,
    MountMissileLauncherI,
    MountTurretI,
    ShipProbe,
    ShipMiningDrone,
    ShipSiphonDrone,
    ShipInterceptor,
    ShipLightHauler,
    ShipCommandFrigate,
    ShipExplorer,
    ShipHeavyFreighter,
    ShipLightShuttle,
    ShipOreHound,
    ShipRefiningFreighter,
    ShipSurveyor,
}

/// Contract details.
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename = "contract", rename_all = "camelCase")]
pub struct Contract {
    /// ID of the contract.
    /// >= 1 characters
    pub id: String,
    /// The symbol of the faction that this contract is for.
    /// >= 1 characters
    pub faction_symbol: String,
    /// Type of contract.
    #[serde(rename = "type")]
    pub contract_type: ContractType,
    /// The terms to fulfill the contract.
    pub terms: ContractTerms,
    /// Whether the contract has been accepted by the agent.
    pub accepted: bool,
    /// Whether the contract has been fulfilled.
    pub fulfilled: bool,
    /// Deprecated in favor of deadline_to_accept.
    pub expiration: String, // TODO: This is supposed to be a "date-time". Figure out the correct Rust type for that.
    /// The time at which the contract is no longer available to be accepted.
    pub deadline_to_accept: String, // TODO: This is supposed to be a "date-time". Figure out the correct Rust type for that.
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ContractType {
    Procurement,
    Transport,
    Shuttle,
}

/// The terms to fulfill the contract.
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ContractTerms {
    /// The deadline for the contract.
    pub deadline: String, // TODO: This is supposed to be a "date-time". Figure out the correct Rust type for that.
    /// Payments for the contract.
    pub payment: ContractPayment,
    /// The cargo that needs to be delivered to fulfill the contract.
    pub deliver: Option<Vec<ContractDeliverGood>>,
}

/// Payments for the contract.
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ContractPayment {
    /// The amount of credits received up front for accepting the contract.
    pub on_accepted: u64,
    /// The amount of credits received when the contract is fulfilled.
    pub on_fulfilled: u64,
}

/// The details of a delivery contract.
/// Includes the type of good, units needed, and the destination.
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ContractDeliverGood {
    /// The symbol of the trade good to deliver.
    /// >= 1 characters
    pub trade_symbol: TradeSymbol,
    /// The destination where goods need to be delivered.
    /// >= 1 characters
    pub destination_symbol: String,
    /// The number of units that need to be delivered on this contract.
    pub units_required: u64,
    /// The number of units fulfilled on this contract.
    pub units_fulfilled: u64,
}

/// A cooldown is a period of time in which a ship cannot perform certain actions.
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Cooldown {
    /// The symbol of the ship that is on cooldown.
    /// >= 1 characters
    pub ship_symbol: String,
    /// The total duration of the cooldown in seconds.
    /// >= 0
    pub total_seconds: u64,
    /// The remaining duration of the cooldown in seconds.
    /// >= 0
    pub remaining_seconds: u64,
    /// The date and time when the cooldown expires in ISO 8601 format.
    pub expiration: Option<String>, // TODO: This is supposed to be a "date-time". Figure out the correct Rust type for that.
}

/// Extraction details.
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Extraction {
    /// Symbol of the ship that executed the extraction.
    /// >= 1 characters
    pub ship_symbol: String,
    /// A yield from the extraction operation.
    #[serde(rename = "yield")]
    pub extraction_yield: ExtractionYield,
}

/// A yield from the extraction operation.
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ExtractionYield {
    /// The good's symbol.
    pub symbol: TradeSymbol,
    /// The number of units extracted that were placed into the ship's cargo hold.
    pub units: u64,
}

/// Faction details.
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Faction {
    /// The symbol of the faction.
    pub symbol: FactionSymbol,
    /// Name of the faction.
    /// >= 1 characters
    pub name: String, // XXX: How does this relate to the faction symbol?
    /// Description of the faction.
    /// >= 1 characters
    pub description: String,
    /// The waypoint in which the faction's HQ is located in.
    /// >= 1 characters
    pub headquarters: String,
    /// List of traits that define this faction.
    pub traits: Vec<FactionTrait>,
    /// Whether or not the faction is currently recruiting new agents.
    pub is_recruiting: bool,
}

/// The symbol of the faction.
#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum FactionSymbol {
    Cosmic,
    Void,
    Galactic,
    Quantum,
    Dominion,
    Astro,
    Corsairs,
    Obsidian,
    Aegis,
    United,
    Solitary,
    Cobalt,
    Omega,
    Echo,
    Lords,
    Cult,
    Ancients,
    Shadow,
    Ethereal,
}

impl Display for FactionSymbol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let res = match self {
            FactionSymbol::Cosmic => "COSMIC",
            FactionSymbol::Void => "VOID",
            FactionSymbol::Galactic => "GALACTIC",
            FactionSymbol::Quantum => "QUANTUM",
            FactionSymbol::Dominion => "DOMINION",
            FactionSymbol::Astro => "ASTRO",
            FactionSymbol::Corsairs => "CORSAIRS",
            FactionSymbol::Obsidian => "OBSIDIAN",
            FactionSymbol::Aegis => "AEGIS",
            FactionSymbol::United => "UNITED",
            FactionSymbol::Solitary => "SOLITARY",
            FactionSymbol::Cobalt => "COBALT",
            FactionSymbol::Omega => "OMEGA",
            FactionSymbol::Echo => "ECHO",
            FactionSymbol::Lords => "LORDS",
            FactionSymbol::Cult => "CULT",
            FactionSymbol::Ancients => "ANCIENTS",
            FactionSymbol::Shadow => "SHADOW",
            FactionSymbol::Ethereal => "ETHEREAL",
        };

        write!(f, "{res}")
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct FactionTrait {
    /// The unique identifier of the trait.
    pub symbol: FactionTraitSymbol,
    /// The name of the trait.
    pub name: String,
    /// A description of the trait.
    pub description: String,
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum FactionTraitSymbol {
    Bureaucratic,
    Secretive,
    Capitalistic,
    Industrious,
    Peaceful,
    Distrustful,
    Welcoming,
    Smugglers,
    Scavengers,
    Rebellious,
    Exiles,
    Pirates,
    Raiders,
    Clan,
    Guild,
    Dominion,
    Fringe,
    Forsaken,
    Isolated,
    Localized,
    Established,
    Notable,
    Dominant,
    Inescapable,
    Innovative,
    Bold,
    Visionary,
    Curious,
    Daring,
    Exploratory,
    Resourceful,
    Flexible,
    Cooperative,
    United,
    Strategic,
    Intelligent,
    ResearchFocused,
    Collaborative,
    Progressive,
    Militaristic,
    TechnologicallyAdvanced,
    Aggressive,
    Imperialistic,
    TreasureHunters,
    Dexterous,
    Unpredictable,
    Brutal,
    Fleeting,
    Adaptable,
    SelfSufficient,
    Defensive,
    Proud,
    Diverse,
    Independent,
    SelfInterested,
    Fragmented,
    Commercial,
    FreeMarkets,
    Entrepreneurial,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct JumpGate {
    /// The symbol of the waypoint.
    /// >= 1 characters
    pub symbol: String,
    /// All the gates that are connected to this waypoint.
    pub connections: Vec<String>, // XXX: Are these the ConnectedSystem things?
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Market {
    /// The symbol of the market. The symbol is the same
    /// as the waypoint where the market is located.
    pub symbol: String,
    /// The list of goods that are exported from this market.
    pub exports: Vec<TradeGood>,
    /// The list of good that are sought as imports in this market.
    pub imports: Vec<TradeGood>,
    /// The list of goods that are bought and sold between agents at this market.
    pub exchange: Vec<TradeGood>,
    /// The list of recent transactions at this market.
    /// Visible only when a ship is present at the market.
    pub transactions: Option<Vec<MarketTransaction>>,
    /// The list of goods that are traded at this market.
    /// Visible only when a ship is present at the market.
    pub trade_goods: Option<Vec<MarketTradeGood>>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TradeGood {
    /// The good's symbol.
    pub symbol: TradeSymbol,
    /// The name of the good.
    pub name: String,
    /// The description of the good.
    pub description: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MarketTradeGood {
    /// The good's symbol.
    pub symbol: TradeSymbol,
    /// The type of trade good (export, import, or exchange).
    #[serde(rename = "type")]
    pub good_type: TradeGoodType,
    /// This is the maximum number of units that can be purchased or sold
    /// at this market in a single trade for this good. Trade volume also
    /// gives an indication of price volatility. A market with a low trade
    /// volume will have large price swings, while high trade volume will
    /// be more resilient to price changes.
    /// >= 1
    pub trade_volume: u64,
    /// The supply level of a trade good.
    pub supply: SupplyLevel,
    /// The activity level of a trade good. If the good is an import,
    /// this represents how strong consumption is. If the good is an
    /// export, this represents how strong the production is for the
    /// good. When activity is strong, consumption or production is near
    /// maximum capacity. When activity is weak, consumption or production
    /// is near minimum capacity.
    pub activity: Option<ActivityLevel>,
    /// The prive at which this good can be purchased from the market.
    /// >= 0
    pub purchase_price: u64,
    /// The price at which this good can be sold to the market.
    /// >= 0
    pub sell_price: u64,
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TradeGoodType {
    Export,
    Import,
    Exchange,
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SupplyLevel {
    Scarce,
    Limited,
    Moderate,
    High,
    Abundant,
}

/// Result of a transaction with a market.
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MarketTransaction {
    /// The symbol of the waypoint.
    /// >= 1 characters
    pub waypoint_symbol: String,
    /// The symbol of the ship that made the transaction.
    pub ship_symbol: String,
    /// The symbol of the trade good.
    pub trade_symbol: TradeSymbol,
    /// The type of transaction.
    #[serde(rename = "type")]
    pub transaction_type: TransactionType,
    /// The number of units of the transaction.
    /// >= 0
    pub units: u64,
    /// The price per unit of the transaction.
    /// >= 0
    pub price_per_unit: u64,
    /// The total price of the transaction.
    /// >= 0
    pub total_price: u64,
    /// The timestamp of the transaction.
    pub timestamp: String, // TODO: This is supposed to be a "date-time". Figure out the correct Rust type for that.
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TransactionType {
    Purchase,
    Sell,
}

/// Meta details for pagination.
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Meta {
    /// Show the total amount of items of this kind that exist.
    /// >= 0
    pub total: u64,
    /// A page denotes an amount of items, offset from the first
    /// item. Each page hold an amount of items equal to the `limit`.
    /// >= 1
    pub page: u64,
    /// The amount of items in each page. Limits how many items can
    /// be fetched at once.
    /// >= 1 && <= 20
    pub limit: u8,
}

/// Result of a repair or scrap transaction (or preview thereof).
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ShipTransaction {
    /// The symbol of the waypoint.
    /// >= 1 characters
    pub waypoint_symbol: String,
    /// The symbol of the ship.
    pub ship_symbol: String,
    /// The total price of the transaction.
    /// >= 0
    pub total_price: u64,
    /// The timestamp of the transaction.
    pub timestamp: String, // TODO: This is supposed to be a "date-time". Figure out the correct Rust type for that.
}

/// The ship that was scanned.
/// Details include information about the ship that could be
/// detected by the scanner.
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct ScannedShip {
    /// The globally unique identifier of the ship.
    symbol: String,
    /// The public registration information of the ship.
    registration: ShipRegistration,
    /// The navigation information of the ship.
    nav: ShipNav,
    /// The frame of the ship.
    frame: Option<ShipFrame>,
    /// The reactor of the ship.
    reactor: Option<ShipReactor>,
    /// The engine of the ship.
    engine: ShipEngine,
    /// List of mounts installed in the ship.
    mounts: Option<Vec<ShipMount>>,
}

/// Details of a system that was scanned.
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct ScannedSystem {
    /// Symbol of the system.
    /// >= 1 characters
    symbol: String,
    /// Symbol of the system's sector.
    /// >= 1 characters
    sector_symbol: String,
    /// The type of system.
    #[serde(rename = "type")]
    system_type: SystemType,
    /// Position in the universe in the x axis.
    x: i64,
    /// Position in the universe in the y axis.
    y: i64,
    /// The system's distance from the scanning ship.
    distance: u64,
}

/// A waypoint that was scanned by a ship.
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct ScannedWaypoint {
    /// The symbol of the waypoint.
    /// >= 1 characters
    symbol: String,
    /// The type of the waypoint.
    #[serde(rename = "type")]
    waypoint_type: WaypointType,
    /// The symbol of the system.
    /// >= 1 characters
    system_symbol: String,
    /// Position in the universe in the x axis.
    x: i64,
    /// Position in the universe in the y axis.
    y: i64,
    /// List of waypoints that orbit this waypoint.
    orbitals: Vec<Waypoint>,
    /// The faction that controls the waypoint.
    faction: Option<Faction>,
    /// The traits of the waypoint.
    traits: Vec<WaypointTrait>,
    /// The chart of a system or waypoint, which makes the location
    /// visible to other agents.
    chart: Option<Chart>,
}

/// Ship details.
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Ship {
    /// The globally unique identifier of the ship
    /// in the following format:
    /// [AGENT_SYMBOL]-[HEX_ID]
    pub symbol: String,
    /// The public registration information of the ship.
    pub registration: ShipRegistration,
    /// The navigation information of the ship.
    pub nav: ShipNav,
    /// The ship's crew service and maintain the ship's systems
    /// and equipment.
    pub crew: ShipCrew,
    /// The frame of the ship. The frame determines the number
    /// of modules and mounting points of the ship, as well
    /// as base fuel capacity. As the condition of the frame
    /// takes more wear, the ship will become more sluggish
    /// and less maneuverable.
    pub frame: ShipFrame,
    /// The reactor of the ship. The reactor is responsible
    /// for powering the ship's systems and weapons.
    pub reactor: ShipReactor,
    /// The engine determines how quickly a ship travels
    /// between waypoints.
    pub engine: ShipEngine,
    /// A cooldown is a period of time in which a ship cannot
    /// perform certain actions.
    pub cooldown: Cooldown,
    /// Modules installed on this ship.
    pub modules: Vec<ShipModule>,
    /// Mounts installed in this ship.
    pub mounts: Vec<ShipMount>,
    /// Ship cargo details.
    pub cargo: ShipCargo,
    /// Details of the ship's fuel tanks including how much
    /// fuel was consumed during the last transit or action.
    pub fuel: ShipFuel,
}

/// Ship cargo details.
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ShipCargo {
    /// The max number of items that can be stored in the cargo hold.
    /// >= 0
    pub capacity: u64,
    /// The number of items currently stored in the cargo hold.
    /// >= 0
    pub units: u64,
    /// The items currently in the cargo hold.
    pub inventory: Vec<ShipCargoItem>,
}

/// The type of cargo item and the number of units.
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ShipCargoItem {
    /// The good's symbol.
    pub symbol: TradeSymbol,
    /// The name of the cargo item type.
    pub name: String,
    /// The description of the cargo item type.
    pub description: String,
    /// The number of units of the cargo item.
    /// >= 1
    pub units: u64,
}

/// The repairable condition of a component.
/// A value of 0 indicates the component needs significant
/// repairs, while a value of 1 indicates the component is
/// in near perfect condition. As the condition of a component
/// is repaired, the overall integrity of the component decreases.
/// >= 0 && <= 1
#[derive(Serialize, Deserialize, Debug)]
pub struct ShipComponentCondition(f64);

/// The overall integrity of the component, which determines
/// the performance of the component. A value of 0 indicates
/// that the component is almost completely degraded, while
/// a value of 1 indicates that the component is in near perfect
/// condition. The integrity of the component is non-repairable,
/// and represents permanent wear over time.
/// >= 0 && <= 1
#[derive(Serialize, Deserialize, Debug)]
pub struct ShipComponentIntegrity(f64);

/// An event that represents damage or wear to
/// a ship's reactor, frame, or engine, reducing
/// the condition of the ship.
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ShipConditionEvent {
    pub symbol: ShipConditionEventType,
    pub component: ShipComponentType,
    /// The name of the event.
    pub name: String,
    /// A description of the event.
    pub description: String,
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ShipConditionEventType {
    ReactorOverload,
    EnergySpikeFromMineral,
    SolarFlareInterference,
    CoolantLeak,
    PowerDistributionFluctuation,
    MagneticFieldDisruption,
    HullMicrometeoriteStrikes,
    StructuralStressFractures,
    CorrosiveMineralContamination,
    ThermalExpansionMismatch,
    VibrationDamageFromDrilling,
    ElectromagneticFieldInterference,
    ImpactWithExtractedDebris,
    FuelEfficiencyDegradation,
    CoolantSystemAgeing,
    DustMicroabrasions,
    ThrusterNozzleWear,
    ExhaustPortClogging,
    BearingLubricationFade,
    SensorCalibrationDrift,
    HullMicrometeoriteDamage,
    SpaceDebrisCollision,
    ThermalStress,
    VibrationOverload,
    PressureDifferentialStress,
    ElectromagneticSurgeEffects,
    AtmosphericEntryHeat,
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ShipComponentType {
    Frame,
    Reactor,
    Engine,
}

/// The ship's crew service and maintain the
/// ship's systems and equipment.
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ShipCrew {
    /// The current number of crew members on the ship.
    pub current: u64,
    /// The minimum number of crew members required to maintain the ship.
    pub required: u64,
    /// The maximum number of crew members the ship can support.
    pub capacity: u64,
    /// The rotation of crew shifts. A stricter shift improves the
    /// ship's performance. A more relaxed shift improves the crew's morale.
    pub rotation: ShiftType,
    /// A rough measure of the crew's morale. A higher morale
    /// means the crew is happier and more productive. A lower
    /// morale means the ship is more prone to accidents.
    /// >= 0 && <= 100
    pub morale: u8,
    /// The amount of credits per crew member paid per hour.
    /// Wages are paid when a ship docks at a civilized waypoint.
    /// >= 0
    pub wages: u64,
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ShiftType {
    Strict,
    Relaxed,
}

/// The engine determines how quickly a ship travels
/// between waypoints.
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ShipEngine {
    /// The symbol of the engine.
    pub symbol: EngineType,
    /// The name of the engine.
    pub name: String,
    /// The description of the engine.
    pub description: String,
    pub condition: ShipComponentCondition,
    pub integrity: ShipComponentIntegrity,
    /// The speed stat of this engine. The higher the speed,
    /// the faster a ship can travel from one point to another.
    /// Reduces the time of arrival when navigating the ship.
    /// >= 1
    pub speed: u64,
    /// The requirements for installation on a ship.
    pub requirements: ShipRequirements,
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EngineType {
    EngineImpulseDriveI,
    EngineIonDriveI,
    EngineIonDriveIi,
    EngineHyperDriveI,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ShipFrame {
    pub symbol: FrameType,
    pub name: String,
    pub description: String,
    pub condition: ShipComponentCondition,
    pub integrity: ShipComponentIntegrity,
    /// The amount of slots that can be dedicated to modules
    /// installed in the ship. Each installed module takes up
    /// a number of slots, and once there are no more slots, no
    /// more modules can be installed.
    /// >= 0
    pub module_slots: u64,
    /// The amount of points that can be dedicated to mounts
    /// installed in this ship. Each installed mount takes up
    /// a number of points, and once there are no more points
    /// remaining, no new mounts can be installed.
    /// >= 0
    pub mounting_points: u64,
    /// The maximum amount of fuel that can be stored in this ship.
    /// When refueling, the ship will be refueled to this amount.
    /// >= 0
    pub fuel_capacity: u64,
    /// The requirements for installation on a ship.
    pub requirements: ShipRequirements,
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[expect(clippy::enum_variant_names)]
pub enum FrameType {
    FrameProbe,
    FrameDrone,
    FrameInterceptor,
    FrameRacer,
    FrameFighter,
    FrameFrigate,
    FrameShuttle,
    FrameExplorer,
    FrameMiner,
    FrameLightFreighter,
    FrameHeavyFrighter,
    FrameTransport,
    FrameDestroyer,
    FrameCruiser,
    FrameCarrier,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ShipFuel {
    pub current: u64,
    pub capacity: u64,
    /// An object that only shows up when an action has consumed
    /// fuel in the process. Shows the fuel consumption data.
    pub consumed: Option<FuelConsumption>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct FuelConsumption {
    pub amount: u64,
    pub timestamp: String, // TODO: This is supposed to be a "date-time". Figure out the correct Rust type for that.
}

/// Result of a transaction for a ship modification,
/// such as installing a mount or a module.
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct ShipModificationTransaction {
    waypoint_symbol: String,
    ship_symbol: String,
    trade_symbol: TradeSymbol,
    total_price: u64,
    timestamp: String, // TODO: This is supposed to be a "date-time". Figure out the correct Rust type for that.
}

/// A module can be installed in a ship and provides
/// a set of capabilities such as storage space or
/// quarters for crew. Module installations are permanent.
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ShipModule {
    pub symbol: ModuleType,
    /// Modules that provide capacity, such as cargo hold or crew
    /// quarters, will show this value to denote how much of a
    /// bonus the module grants.
    pub capacity: Option<u64>,
    /// Modules that have a range, such as sensor arrays,
    /// will show this value to denote how far the module can reach
    /// with its capabilities.
    pub range: Option<u64>,
    pub name: String,
    pub description: String,
    pub requirements: ShipRequirements,
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ModuleType {
    ModuleMineralProcessorI,
    ModuleGasProcessorI,
    ModuleCargoHoldI,
    ModuleCargoHoldIi,
    ModuleCargoHoldIii,
    ModuleCrewQuartersI,
    ModuleEnvoyQuartersI,
    ModulePassengerCabinI,
    ModuleMicroRefineryI,
    ModuleOreRefineryI,
    ModuleFuelRefineryI,
    ModuleScienceLabI,
    ModuleJumpDriveI,
    ModuleJumpDriveIi,
    ModuleJumpDriveIii,
    ModuleWarpDriveI,
    ModuleWarpDriveIi,
    ModuleWarpDriveIii,
    ModuleShieldGeneratorI,
    ModuleShieldGeneratorIi,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ShipMount {
    pub symbol: MountType,
    pub name: String,
    pub description: Option<String>,
    /// Mounts that have this value, such as mining lasers,
    /// denote how powerful this mount's capabilities are.
    pub strength: Option<u64>,
    /// Mounts that have this value denote what goods can
    /// be produced from using this mount.
    pub deposits: Option<Vec<DepositType>>,
    pub requirements: ShipRequirements,
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MountType {
    MountGasSiphonI,
    MountGasSiphonIi,
    MountGasSiphonIii,
    MountSurveyorI,
    MountSurveyorIi,
    MountSurveyorIii,
    MountSensorArrayI,
    MountSensorArrayIi,
    MountSensorArrayIii,
    MountMiningLaserI,
    MountMiningLaserIi,
    MountMiningLaserIii,
    MountLaserCannonI,
    MountMissileLauncherI,
    MountTurretI,
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DepositType {
    QuartzSand,
    SiliconCrystals,
    PreciousStones,
    IceWater,
    AmmoniaIce,
    IronOre,
    CopperOre,
    SilverOre,
    AluminumOre,
    GoldOre,
    PlatinumOre,
    Diamonds,
    UraniteOre,
    MeritiumOre,
}

/// The navigation information of the ship.
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ShipNav {
    pub system_symbol: String,
    pub waypoint_symbol: String,
    /// The routing information for the ship's most
    /// recent transit or current location.
    pub route: ShipNavRoute,
    pub status: ShipNavStatus,
    pub flight_mode: ShipNavFlightMode,
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ShipNavFlightMode {
    Drift,
    Stealth,
    Cruise,
    Burn,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ShipNavRoute {
    pub destination: ShipNavRouteWaypoint,
    pub origin: ShipNavRouteWaypoint,
    pub departure_time: String, // TODO: This is supposed to be a "date-time". Figure out the correct Rust type for that.
    pub arrival: String, // TODO: This is supposed to be a "date-time". Figure out the correct Rust type for that.
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ShipNavRouteWaypoint {
    pub symbol: String,
    #[serde(rename = "type")]
    pub waypoint_type: WaypointType,
    pub system_symbol: String,
    pub x: i64,
    pub y: i64,
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ShipNavStatus {
    InTransit,
    InOrbit,
    Docked,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ShipReactor {
    pub symbol: ReactorType,
    pub name: String,
    pub description: String,
    pub condition: ShipComponentCondition,
    pub integrity: ShipComponentIntegrity,
    /// The amount of power provided by this reactor.
    /// The more power a reactor provides to the ship,
    /// the lower the cooldown it gets when using a module
    /// or mount that taxes the ship's power.
    pub power_output: u64,
    pub requirements: ShipRequirements,
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ReactorType {
    ReactorSolarI,
    ReactorFusionI,
    ReactorFissionI,
    ReactorChemicalI,
    ReactorAntimatterI,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ShipRegistration {
    pub name: String,
    pub faction_symbol: FactionSymbol,
    /// The registered role of the ship.
    pub role: ShipRole,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ShipRequirements {
    pub power: Option<u64>,
    pub crew: Option<i64>,
    pub slots: Option<u64>,
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ShipRole {
    Fabricator,
    Harvester,
    Hauler,
    Interceptor,
    Excavator,
    Transport,
    Repair,
    Surveyor,
    Command,
    Carrier,
    Patrol,
    Satellite,
    Explorer,
    Refinery,
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[expect(clippy::enum_variant_names)]
pub enum ShipType {
    ShipProbe,
    ShipMiningDrone,
    ShipSiphonDrone,
    ShipInterceptor,
    ShipLightHauler,
    ShipCommandFrigate,
    ShipExplorer,
    ShipHeavyFreighter,
    ShipLightShuttle,
    ShipOreHound,
    ShipRefiningFreighter,
    ShipSurveyor,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ShipTypeListItem {
    #[serde(rename = "type")]
    pub ship_type: ShipType,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Shipyard {
    pub symbol: String,
    pub ship_types: Vec<ShipTypeListItem>,
    pub transactions: Option<Vec<ShipyardTransaction>>,
    pub ships: Option<Vec<ShipyardShip>>,
    /// The fee to modify a ship at this shipyard.
    /// This includes installing or removing modules
    /// and mounts on a ship. In the case of mounts, the
    /// fee is a flat rate per mount. In the case of modules,
    /// the fee is per slot the module occupies.
    pub modifications_fee: u64,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ShipyardShip {
    #[serde(rename = "type")]
    pub ship_type: ShipType,
    pub name: String,
    pub description: String,
    pub supply: SupplyLevel,
    pub activity: Option<ActivityLevel>,
    pub purchase_price: u64,
    pub frame: ShipFrame,
    pub reactor: ShipReactor,
    pub engine: ShipEngine,
    pub modules: Vec<ShipModule>,
    pub mounts: Vec<ShipMount>,
    pub crew: ShipCrew,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ShipyardTransaction {
    pub waypoint_symbol: String,
    pub ship_symbol: String,
    pub ship_type: ShipType,
    pub price: u64,
    pub agent_symbol: String,
    pub timestamp: String, // TODO: This is supposed to be a "date-time". Figure out the correct Rust type for that.
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Siphon {
    ship_symbol: String,
    #[serde(rename = "yield")]
    siphon_yield: SiphonYield,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct SiphonYield {
    symbol: TradeSymbol,
    units: u64,
}

/// A resource survey of a waypoint, detailing
/// a specific extraction location and the types of
/// resources that can be found there.
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Survey {
    /// A unique signature for the location of this survey.
    /// This signature is verified when attempting an extraction using this survey.
    pub signature: String,
    pub symbol: String,
    pub deposits: Vec<SurveyDeposit>,
    pub expiration: String, // TODO: This is supposed to be a "date-time". Figure out the correct Rust type for that.
    pub size: DepositSize,
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DepositSize {
    Small,
    Moderate,
    Large,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SurveyDeposit(String);

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct System {
    pub symbol: SystemSymbol,
    pub sector_symbol: String,
    #[serde(rename = "type")]
    pub system_type: SystemType,
    pub x: i64,
    pub y: i64,
    pub waypoints: Vec<SystemWaypoint>,
    pub factions: Vec<SystemFaction>,
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SystemFaction {
    Cosmic,
    Void,
    Galactic,
    Quantum,
    Dominion,
    Astro,
    Corsairs,
    Obsidian,
    Aegis,
    United,
    Solitary,
    Cobalt,
    Omega,
    Echo,
    Lords,
    Cult,
    Ancients,
    Shadow,
    Ethereal,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SystemSymbol(String);

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SystemWaypoint {
    symbol: String,
    #[serde(rename = "type")]
    waypoint_type: WaypointType,
    x: i64,
    y: i64,
    orbitals: Vec<WaypointOrbital>,
    orbits: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Waypoint {
    pub symbol: String,
    #[serde(rename = "type")]
    pub waypoint_type: WaypointType,
    pub system_symbol: String,
    pub x: i64,
    pub y: i64,
    pub orbitals: Vec<WaypointOrbital>,
    pub orbits: Option<String>,
    pub faction: Option<WaypointFaction>,
    pub traits: Vec<WaypointTrait>,
    pub modifiers: Option<Vec<WaypointModifier>>,
    pub chart: Option<Chart>,
    pub is_under_construction: bool,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct WaypointFaction {
    pub symbol: FactionSymbol,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct WaypointModifier {
    pub symbol: WaypointModifierSymbol,
    pub name: String,
    pub description: String,
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum WaypointModifierSymbol {
    Stripped,
    Unstable,
    RadiationLeak,
    CriticalLimit,
    CivilUnrest,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct WaypointOrbital {
    pub symbol: WaypointSymbol,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct WaypointSymbol(String);

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct WaypointTrait {
    pub symbol: WaypointTraitSymbol,
    pub name: String,
    pub description: String,
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum WaypointTraitSymbol {
    Uncharted,
    UnderConstruction,
    Marketplace,
    Shipyard,
    Outpost,
    ScatteredSettlements,
    SprawlingCities,
    MegaStructures,
    PirateBase,
    Overcrowded,
    HighTech,
    Corrupt,
    Bureaucratic,
    TradingHub,
    Industrial,
    BlackMarket,
    ResearchFacility,
    MilitaryBase,
    SurveillanceOutpost,
    ExplorationOutpost,
    MineralDeposits,
    CommonMetalDeposits,
    PreciousMetalDeposits,
    RareMetalDeposits,
    MethanePools,
    IceCrystals,
    ExplosiveGases,
    StrongMagnetosphere,
    VibrantAuroras,
    SaltFlats,
    Canyons,
    PerpetualDaylight,
    PerpetualOvercast,
    DrySeabeds,
    MagmaSeas,
    Supervolcanoes,
    AshClouds,
    VastRuins,
    MutatedFlora,
    Terraformed,
    ExtremeTemperatures,
    ExtremePressure,
    DiverseLife,
    ScarceLife,
    Fossils,
    WeakGravity,
    StrongGravity,
    CrushingGravity,
    ToxicAtmosphere,
    CorrosiveAtmosphere,
    BreathableAtmosphere,
    ThinAtmosphere,
    Jovian,
    Rocky,
    Volcanic,
    Frozen,
    Swamp,
    Barren,
    Temperate,
    Jungle,
    Ocean,
    Radioactive,
    MicroGravityAnomalies,
    DebrisCluster,
    DeepCraters,
    ShallowCraters,
    UnstableComposition,
    HollowedInterior,
    Stripped,
}

impl Display for WaypointTraitSymbol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let res = match self {
            WaypointTraitSymbol::Uncharted => "UNCHARTED",
            WaypointTraitSymbol::UnderConstruction => "UNDER_CONSTRUCTION",
            WaypointTraitSymbol::Marketplace => "MARKETPLACE",
            WaypointTraitSymbol::Shipyard => "SHIPYARD",
            WaypointTraitSymbol::Outpost => "OUTPOST",
            WaypointTraitSymbol::ScatteredSettlements => "SCATTERED_SETTLEMENTS",
            WaypointTraitSymbol::SprawlingCities => "SPRAWLING_CITIES",
            WaypointTraitSymbol::MegaStructures => "MEGA_STRUCTURES",
            WaypointTraitSymbol::PirateBase => "PIRATE_BASE",
            WaypointTraitSymbol::Overcrowded => "OVERCROWDED",
            WaypointTraitSymbol::HighTech => "HIGH_TECH",
            WaypointTraitSymbol::Corrupt => "CORRUPT",
            WaypointTraitSymbol::Bureaucratic => "BUREAUCRATIC",
            WaypointTraitSymbol::TradingHub => "TRADING_HUB",
            WaypointTraitSymbol::Industrial => "INDUSTRIAL",
            WaypointTraitSymbol::BlackMarket => "BLACK_MARKET",
            WaypointTraitSymbol::ResearchFacility => "RESEARCH_FACILITY",
            WaypointTraitSymbol::MilitaryBase => "MILITARY_BASE",
            WaypointTraitSymbol::SurveillanceOutpost => "SURVEILLANCE_OUTPOST",
            WaypointTraitSymbol::ExplorationOutpost => "EXPLORATION_OUTPOST",
            WaypointTraitSymbol::MineralDeposits => "MINERAL_DEPOSITS",
            WaypointTraitSymbol::CommonMetalDeposits => "COMMON_METAL_DEPOSITS",
            WaypointTraitSymbol::PreciousMetalDeposits => "PRECIOUS_METAL_DEPOSITS",
            WaypointTraitSymbol::RareMetalDeposits => "RARE_METAL_DEPOSITS",
            WaypointTraitSymbol::MethanePools => "METHANE_POOLS",
            WaypointTraitSymbol::IceCrystals => "ICE_CRYSTALS",
            WaypointTraitSymbol::ExplosiveGases => "EXPLOSIVE_GASES",
            WaypointTraitSymbol::StrongMagnetosphere => "STRONG_MAGNETOSPHERE",
            WaypointTraitSymbol::VibrantAuroras => "VIBRANT_AURORAS",
            WaypointTraitSymbol::SaltFlats => "SALT_FLATS",
            WaypointTraitSymbol::Canyons => "CANYONS",
            WaypointTraitSymbol::PerpetualDaylight => "PERPETUAL_DAYLIGHT",
            WaypointTraitSymbol::PerpetualOvercast => "PERPETUAL_OVERCAST",
            WaypointTraitSymbol::DrySeabeds => "DRY_SEABEDS",
            WaypointTraitSymbol::MagmaSeas => "MAGMA_SEAS",
            WaypointTraitSymbol::Supervolcanoes => "SUPERVOLCANOES",
            WaypointTraitSymbol::AshClouds => "ASH_CLOUDS",
            WaypointTraitSymbol::VastRuins => "VAST_RUINS",
            WaypointTraitSymbol::MutatedFlora => "MUTATED_FLORA",
            WaypointTraitSymbol::Terraformed => "TERRAFORMED",
            WaypointTraitSymbol::ExtremeTemperatures => "EXTREME_TEMPERATURES",
            WaypointTraitSymbol::ExtremePressure => "EXTREME_PRESSURE",
            WaypointTraitSymbol::DiverseLife => "DIVERSE_LIFE",
            WaypointTraitSymbol::ScarceLife => "SCARCE_LIFE",
            WaypointTraitSymbol::Fossils => "FOSSILS",
            WaypointTraitSymbol::WeakGravity => "WEAK_GRAVITY",
            WaypointTraitSymbol::StrongGravity => "STRONG_GRAVITY",
            WaypointTraitSymbol::CrushingGravity => "CRUSHING_GRAVITY",
            WaypointTraitSymbol::ToxicAtmosphere => "TOXIC_ATMOSPHERE",
            WaypointTraitSymbol::CorrosiveAtmosphere => "CORROSIVE_ATMOSPHERE",
            WaypointTraitSymbol::BreathableAtmosphere => "BREATHABLE_ATMOSPHERE",
            WaypointTraitSymbol::ThinAtmosphere => "THIN_ATMOSPHERE",
            WaypointTraitSymbol::Jovian => "JOVIAN",
            WaypointTraitSymbol::Rocky => "ROCKY",
            WaypointTraitSymbol::Volcanic => "VOLCANIC",
            WaypointTraitSymbol::Frozen => "FROZEN",
            WaypointTraitSymbol::Swamp => "SWAMP",
            WaypointTraitSymbol::Barren => "BARREN",
            WaypointTraitSymbol::Temperate => "TEMPERATE",
            WaypointTraitSymbol::Jungle => "JUNGLE",
            WaypointTraitSymbol::Ocean => "OCEAN",
            WaypointTraitSymbol::Radioactive => "RADIOACTIVE",
            WaypointTraitSymbol::MicroGravityAnomalies => "MICRO_GRAVITY_ANOMALIES",
            WaypointTraitSymbol::DebrisCluster => "DEBRIS_CLUSTER",
            WaypointTraitSymbol::DeepCraters => "DEEP_CRATERS",
            WaypointTraitSymbol::ShallowCraters => "SHALLOW_CRATERS",
            WaypointTraitSymbol::UnstableComposition => "UNSTABLE_COMPOSITION",
            WaypointTraitSymbol::HollowedInterior => "HOLLOWED_INTERIOR",
            WaypointTraitSymbol::Stripped => "STRIPPED",
        };

        write!(f, "{res}")
    }
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum WaypointType {
    Planet,
    GasGiant,
    Moon,
    OrbitalStation,
    JumpGate,
    AsteroidField,
    Asteroid,
    EngineeredAsteroid,
    AsteroidBase,
    Nebula,
    DebrisField,
    GravityWell,
    ArtificialGravityWell,
    FuelStation,
}

impl Display for WaypointType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let res = match self {
            WaypointType::Planet => "PLANET",
            WaypointType::GasGiant => "GAS_GIANT",
            WaypointType::Moon => "MOON",
            WaypointType::OrbitalStation => "ORBITAL_STATION",
            WaypointType::JumpGate => "JUMP_GATE",
            WaypointType::AsteroidField => "ASTEROID_FIELD",
            WaypointType::Asteroid => "ASTEROID",
            WaypointType::EngineeredAsteroid => "ENGINEERED_ASTEROID",
            WaypointType::AsteroidBase => "ASTEROID_BASE",
            WaypointType::Nebula => "NEBULA",
            WaypointType::DebrisField => "DEBRIS_FIELD",
            WaypointType::GravityWell => "GRAVITY_WELL",
            WaypointType::ArtificialGravityWell => "ARTIFICIAL_GRAVITY_WELL",
            WaypointType::FuelStation => "FUEL_STATION",
        };

        write!(f, "{res}")
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ApiStatus {
    pub status: String,
    pub version: String,
    pub reset_date: String,
    pub description: String,
    pub stats: GameStats,
    pub leaderboards: Leaderboards,
    pub server_resets: ServerResets,
    pub announcements: Vec<Announcement>,
    pub links: Vec<Link>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GameStats {
    pub agents: u64,
    pub ships: u64,
    pub systems: u64,
    pub waypoints: u64,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Leaderboards {
    pub most_credits: Vec<LeaderboardAgentCredits>,
    pub most_submitted_charts: Vec<LeaderboardAgentCharts>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct LeaderboardAgentCredits {
    pub agent_symbol: String,
    pub credits: i64,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct LeaderboardAgentCharts {
    pub agent_symbol: String,
    pub chart_count: u64,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ServerResets {
    pub next: String,
    pub frequency: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Announcement {
    pub title: String,
    pub body: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Link {
    pub name: String,
    pub url: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RegisterAgent {
    pub faction: FactionSymbol,
    pub symbol: String,
    pub email: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DeliverCargo {
    pub ship_symbol: String,
    pub trade_symbol: TradeSymbol,
    pub units: u64,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ShipPurchase {
    pub ship_type: ShipType,
    pub waypoint_symbol: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Produce {
    pub produce: TradeSymbol,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ApiResponse {
    pub data: ApiResponseData,
    pub meta: Option<Meta>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum ApiResponseData {
    RegisterAgent(Box<RegisterAgentSuccess>),
    GetAgent(Agent),
    GetSystem(System),
    GetWaypoint(Waypoint),
    GetMarket(Market),
    GetShipyard(Shipyard),
    GetJumpGate(JumpGate),
    GetConstructionSite(Construction),
    ListAgents(Vec<Agent>),
    ListFactions(Vec<Faction>),
    ListSystems(Vec<System>),
    ListWaypoints(Vec<Waypoint>),
    ListContracts(Vec<Contract>),
    GetContract(Contract),
    UpdateContract {
        agent: Option<Agent>,
        contract: Contract,
        cargo: Option<ShipCargo>,
    },
    GetFaction(Faction),
    UpdateConstruction {
        construction: Construction,
        cargo: ShipCargo,
    },
    ListShips(Vec<Ship>),
    GetShip(Box<Ship>),
    GetCargo(ShipCargo),
    GetNav(ShipNav),
    GetMounts(Vec<ShipMount>),
    GetShipTransaction {
        transaction: ShipTransaction,
    },
    GetCooldown(Cooldown),
    ShipPurchase {
        agent: Agent,
        ship: Box<Ship>,
        transaction: ShipyardTransaction,
    },
    Refine {
        cargo: ShipCargo,
        cooldown: Cooldown,
        produced: Vec<TradeGoodAmount>,
        consumed: Vec<TradeGoodAmount>,
    },
    CreateChart {
        chart: Chart,
        waypoint: Waypoint,
    },
    CreateSurvey {
        cooldown: Cooldown,
        surveys: Vec<Survey>,
    },
    ExtractResources {
        cooldown: Cooldown,
        extraction: Extraction,
        cargo: ShipCargo,
        events: Vec<ShipConditionEvent>,
    },
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RegisterAgentSuccess {
    pub agent: Agent,
    pub contract: Contract,
    pub faction: Faction,
    pub ship: Ship,
    pub token: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TradeGoodAmount {
    pub trade_symbol: TradeSymbol,
    pub units: u64,
}
