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
pub mod market;
pub mod order;
pub mod types;

pub use decision::Decision;
pub use error::CoreError;
pub use market::{Bar, DepthLevel, Tick};
pub use order::{
    FillReport, OrderAck, OrderRequest, OrderSide, OrderStatus, OrderType, TimeInForce,
};
pub use types::{Currency, Exchange, InstrumentId, Money, Price, PriceBand, Quantity};

// Re-export foundational external crates for version consistency
pub use chrono;
pub use rust_decimal;
pub use uuid;
