//! Strategy/indicator signal wire contracts.

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::types::InstrumentId;

/// Standard bar/candle interval.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Timeframe {
    /// 1 minute.
    M1,
    /// 5 minutes.
    M5,
    /// 15 minutes.
    M15,
    /// 30 minutes.
    M30,
    /// 1 hour.
    H1,
    /// 1 day.
    D1,
    /// 1 week.
    W1,
}

/// The directional bias a signal expresses.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SignalDirection {
    /// Expects price to rise.
    Bullish,
    /// Expects price to fall.
    Bearish,
    /// No directional edge.
    Neutral,
}

/// The holding horizon a signal is meant for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SignalHorizon {
    /// Closed out same day.
    Intraday,
    /// Held days to a few weeks.
    Swing,
    /// Held weeks to months.
    Positional,
}

/// A single strategy/indicator signal.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct Signal {
    /// The instrument this signal is about.
    pub instrument: InstrumentId,
    /// Directional bias.
    pub direction: SignalDirection,
    /// Intended holding horizon.
    pub horizon: SignalHorizon,
    /// Bar interval the signal was computed on.
    pub timeframe: Timeframe,
    /// Confidence, in `[0, 1]`.
    pub strength: Decimal,
    /// Strategy/algo id that generated this signal.
    pub generated_by: String,
    /// When the signal was generated.
    pub timestamp: DateTime<Utc>,
}

/// Request to scan for signals matching the given criteria.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct SignalScanRequest {
    /// Instruments to scan. Empty means the full universe.
    pub universe: Vec<InstrumentId>,
    /// Restrict to this horizon, if set.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub horizon: Option<SignalHorizon>,
    /// Restrict to this direction, if set.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub direction: Option<SignalDirection>,
    /// Restrict to this timeframe, if set.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timeframe: Option<Timeframe>,
}

/// The matching signals for a [`SignalScanRequest`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct SignalScanResponse {
    /// Matching signals.
    pub signals: Vec<Signal>,
}
