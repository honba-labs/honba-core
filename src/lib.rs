//! # 本場・核 honba-core
//!
//! Shared core canonical types for the Honba Indian equities trading platform.
//!
//! Provides:
//! - Canonical [`Exchange`] and [`InstrumentId`]
//! - Exact decimal financial math ([`Price`], [`Quantity`], [`Money`])
//! - Canonical market data [`Tick`], [`DepthLevel`], [`Bar`]
//! - Canonical [`OrderRequest`], [`OrderAck`], [`FillReport`]
//! - Safe, closed, strictly-validated AI [`Decision`]
//! - Scrip-wise [`PriceBand`] limits for Indian market microstructure

pub mod decision;
pub mod error;
pub mod events;
pub mod instrument;
pub mod market;
pub mod order;
pub mod portfolio;
pub mod ports;
pub mod screener;
pub mod signal;
pub mod types;

pub use decision::Decision;
pub use error::CoreError;
pub use events::{ExecutionEvent, Reason, ReasonGroup, ReasonPolarity, TradingState};
pub use instrument::{
    DatasetId, Instrument, InstrumentKind, OptionType, PositionSide, TradeId, VenueOrderId,
};
pub use market::{Bar, DepthLevel, Tick};
pub use order::{
    CancelRequest, FillReport, ModifyRequest, OrderAck, OrderEvent, OrderEventKind, OrderRequest,
    OrderSide, OrderStatus, OrderType, RejectReason, TimeInForce,
};
pub use portfolio::{
    PortfolioAccount, PortfolioGreeks, PortfolioHolding, PortfolioMode, PortfolioPosition,
    PortfolioSnapshot, PortfolioSource, PortfolioTotals,
};
pub use ports::{
    BarReader, BarRequest, ConnectorState, DatedInstrument, DepthReader, ExecutionGateway,
    InstrumentMaster, MarketDataConnector, MarketEvent, PortError, PortResult, QuoteReader,
    SubscriptionParams, TickReader, TickRequest,
};
pub use screener::{
    FilterOp, MetricDefinition, MetricKeySpec, MetricPeriod, MetricRef, ScreenerFilterGroup,
    ScreenerFilterPredicate, ScreenerRow, ScreenerScanRequest, ScreenerScanResponse, SortSpec,
};
pub use signal::{
    Signal, SignalDirection, SignalHorizon, SignalScanRequest, SignalScanResponse, Timeframe,
};
pub use types::{Currency, Exchange, InstrumentId, MinorUnit, Money, Price, PriceBand, Quantity};

/// Version of the JSON wire contract, owned here because this crate owns the
/// wire types.
///
/// The generated JSON Schema, the OpenAPI document, the MCP tool schemas and
/// the Python stubs all carry it; a change to any wire type bumps it.
///
/// v6: added instrument master data, portfolio snapshots, signals, the
/// screener, cross-subsystem domain events, and the `ports` module —
/// the real home for symbols that used to live in the now-empty
/// `honba-kokoro-{entities,messages,ports}` marker crates.
pub const SCHEMA_VERSION: u32 = 6;

/// Semantic version of the REST endpoint surface.
pub const API_VERSION: &str = "v1";

// Re-export foundational external crates for version consistency
pub use chrono;
pub use rust_decimal;
pub use uuid;
