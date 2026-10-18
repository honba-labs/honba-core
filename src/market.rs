//! Canonical market data models and feeds normalization.

use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::types::{InstrumentId, Price, PriceBand, Quantity};

/// Level 2 market depth entry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct DepthLevel {
    /// Depth price level
    pub price: Price,
    /// Total quantity at this price
    pub quantity: Quantity,
    /// Number of open orders at this price (if provided by broker)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub orders_count: Option<u32>,
}

/// Canonical typed Tick enum for broker feeds.
///
/// Indian broker feeds (Zerodha, Upstox, Fyers) deliver ticks in different shapes:
/// - LTP only (lightweight mode)
/// - Best Bid/Ask quote (with optional price band)
/// - Full 5-level market depth
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", content = "data", rename_all = "snake_case")]
pub enum Tick {
    /// Last Traded Price tick.
    Ltp {
        /// Canonical instrument ID
        instrument: InstrumentId,
        /// Last traded price
        price: Price,
        /// Cumulative volume or last traded quantity
        volume: Quantity,
        /// Exchange timestamp
        exchange_ts: DateTime<Utc>,
        /// Local ingestion receive timestamp
        receive_ts: DateTime<Utc>,
    },
    /// Best Bid/Ask top-of-book quote tick.
    Quote {
        /// Canonical instrument ID
        instrument: InstrumentId,
        /// Best bid price
        bid: Price,
        /// Best ask price
        ask: Price,
        /// Best bid quantity
        bid_qty: Quantity,
        /// Best ask quantity
        ask_qty: Quantity,
        /// Last traded price (if available)
        last_price: Option<Price>,
        /// Exchange timestamp
        exchange_ts: DateTime<Utc>,
        /// Local ingestion receive timestamp
        receive_ts: DateTime<Utc>,
        /// Scrip price band enriched by market data feed
        #[serde(default, skip_serializing_if = "Option::is_none")]
        band: Option<PriceBand>,
    },
    /// Full market depth tick (L2).
    Depth {
        /// Canonical instrument ID
        instrument: InstrumentId,
        /// Bid depth levels (typically top 5 in Indian markets)
        bids: Vec<DepthLevel>,
        /// Ask depth levels (typically top 5 in Indian markets)
        asks: Vec<DepthLevel>,
        /// Last traded price
        last_price: Option<Price>,
        /// Exchange timestamp
        exchange_ts: DateTime<Utc>,
        /// Local ingestion receive timestamp
        receive_ts: DateTime<Utc>,
    },
}

impl Tick {
    /// Get the instrument ID for this tick.
    pub fn instrument(&self) -> &InstrumentId {
        match self {
            Self::Ltp { instrument, .. } => instrument,
            Self::Quote { instrument, .. } => instrument,
            Self::Depth { instrument, .. } => instrument,
        }
    }

    /// Get the receive timestamp for this tick.
    pub fn receive_ts(&self) -> DateTime<Utc> {
        match self {
            Self::Ltp { receive_ts, .. } => *receive_ts,
            Self::Quote { receive_ts, .. } => *receive_ts,
            Self::Depth { receive_ts, .. } => *receive_ts,
        }
    }

    /// Extract the most recent price from this tick if present.
    pub fn last_price(&self) -> Option<Price> {
        match self {
            Self::Ltp { price, .. } => Some(*price),
            Self::Quote {
                last_price, bid, ..
            } => last_price.or(Some(*bid)),
            Self::Depth {
                last_price, bids, ..
            } => last_price.or_else(|| bids.first().map(|b| b.price)),
        }
    }
}

/// Standard OHLCV bar / candle.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct Bar {
    /// Canonical instrument ID
    pub instrument: InstrumentId,
    /// Bar opening price
    pub open: Price,
    /// Bar high price
    pub high: Price,
    /// Bar low price
    pub low: Price,
    /// Bar closing price
    pub close: Price,
    /// Total volume during bar interval
    pub volume: Quantity,
    /// Bar start or end timestamp (UTC)
    pub timestamp: DateTime<Utc>,
}
