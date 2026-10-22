//! Hexagonal port traits (interfaces only, no implementations).
//!
//! `honba-core` stays zero-I/O (`AGENTS.md` §2.5): these traits are abstract
//! contracts, not implementations. Real market-data connectors and
//! execution gateways live in `honba-me`/`honba-te` and implement these
//! traits, injected as `Arc<dyn Trait>` per `AGENTS.md` §2.1. Every trait
//! here is object-safe for exactly that reason.

use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::instrument::Instrument;
use crate::market::{Bar, DepthLevel, Tick};
use crate::order::{ModifyRequest, OrderAck, OrderRequest};
use crate::signal::Timeframe;
use crate::types::InstrumentId;

/// Errors a port implementation can return.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum PortError {
    /// The underlying connection could not be established or was lost.
    #[error("connection failed: {0}")]
    ConnectionFailed(String),
    /// The operation did not complete in time.
    #[error("operation timed out")]
    Timeout,
    /// The requested resource does not exist.
    #[error("not found: {0}")]
    NotFound(String),
    /// The operation is not supported by this implementation.
    #[error("unsupported: {0}")]
    Unsupported(String),
}

/// Result type for port operations.
pub type PortResult<T> = Result<T, PortError>;

/// Connection status of a market-data connector.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum ConnectorState {
    /// Not connected.
    Disconnected,
    /// Connection in progress.
    Connecting,
    /// Connected and healthy.
    Connected,
    /// Connected but degraded.
    Degraded {
        /// Why it's degraded.
        reason: String,
    },
}

/// A request for historical bars.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct BarRequest {
    /// The instrument to fetch bars for.
    pub instrument: InstrumentId,
    /// Bar interval.
    pub timeframe: Timeframe,
    /// Start of the window, inclusive.
    pub from: DateTime<Utc>,
    /// End of the window, inclusive.
    pub to: DateTime<Utc>,
}

/// A request for historical ticks.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct TickRequest {
    /// The instrument to fetch ticks for.
    pub instrument: InstrumentId,
    /// Start of the window, inclusive.
    pub from: DateTime<Utc>,
    /// End of the window, inclusive.
    pub to: DateTime<Utc>,
}

/// Parameters for a live market-data subscription.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct SubscriptionParams {
    /// Instruments to subscribe to.
    pub instruments: Vec<InstrumentId>,
    /// Whether full market depth (L2) is wanted, not just top-of-book.
    pub depth: bool,
}

/// An instrument's reference data as of a specific date — corporate
/// actions and expiry rollovers change an instrument's metadata over time,
/// so lookups are always dated.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct DatedInstrument {
    /// The instrument record.
    pub instrument: Instrument,
    /// The date this record is valid as of.
    pub as_of: DateTime<Utc>,
}

/// An event pushed by a [`MarketDataConnector`] to its subscribers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum MarketEvent {
    /// A tick (LTP, quote, or depth).
    Tick(Tick),
    /// A completed bar.
    Bar(Bar),
    /// The connector's own connection state changed.
    ConnectorStateChanged(ConnectorState),
}

/// Reads historical bars.
pub trait BarReader {
    /// Fetch bars matching the request.
    fn read_bars(&self, req: &BarRequest) -> PortResult<Vec<Bar>>;
}

/// Reads historical ticks.
pub trait TickReader {
    /// Fetch ticks matching the request.
    fn read_ticks(&self, req: &TickRequest) -> PortResult<Vec<Tick>>;
}

/// Reads the latest quote for an instrument.
pub trait QuoteReader {
    /// Fetch the most recent quote tick.
    fn latest_quote(&self, instrument: &InstrumentId) -> PortResult<Tick>;
}

/// Reads current market depth for an instrument.
pub trait DepthReader {
    /// Fetch the current depth book.
    fn depth(&self, instrument: &InstrumentId) -> PortResult<Vec<DepthLevel>>;
}

/// A live market-data connection (broker WebSocket, replay feed, etc).
pub trait MarketDataConnector {
    /// Current connection state.
    fn state(&self) -> ConnectorState;
    /// Subscribe to the given instruments.
    fn subscribe(&mut self, params: &SubscriptionParams) -> PortResult<()>;
    /// Drain whatever market events have arrived since the last poll.
    fn poll(&mut self) -> PortResult<Vec<MarketEvent>>;
}

/// Instrument master-data lookup.
pub trait InstrumentMaster {
    /// Look up an instrument's dated reference data.
    fn lookup(&self, id: &InstrumentId) -> PortResult<DatedInstrument>;
    /// Search instruments by symbol prefix.
    fn search(&self, symbol_prefix: &str) -> PortResult<Vec<InstrumentId>>;
}

/// Order execution against a broker or simulator.
pub trait ExecutionGateway {
    /// Submit a new order.
    fn submit(&mut self, req: &OrderRequest) -> PortResult<OrderAck>;
    /// Cancel an existing order.
    fn cancel(&mut self, order_id: &str) -> PortResult<()>;
    /// Modify an existing order.
    fn modify(&mut self, req: &ModifyRequest) -> PortResult<OrderAck>;
}
