//! Strategy/indicator signal wire contracts.

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::types::{InstrumentId, Price};

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
    /// Target entry price for trade execution, if recommended.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub entry_price: Option<Price>,
    /// Recommended stop loss price level.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stop_loss: Option<Price>,
    /// Recommended take profit price level.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub take_profit: Option<Price>,
    /// Current or computed volatility (e.g. ATR).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub volatility_atr: Option<Decimal>,
}

impl Signal {
    /// Creates a new signal with core attributes and execution levels unset.
    pub fn new(
        instrument: InstrumentId,
        direction: SignalDirection,
        horizon: SignalHorizon,
        timeframe: Timeframe,
        strength: Decimal,
        generated_by: impl Into<String>,
        timestamp: DateTime<Utc>,
    ) -> Self {
        Self {
            instrument,
            direction,
            horizon,
            timeframe,
            strength,
            generated_by: generated_by.into(),
            timestamp,
            entry_price: None,
            stop_loss: None,
            take_profit: None,
            volatility_atr: None,
        }
    }

    /// Attach a recommended entry price.
    pub fn with_entry_price(mut self, price: Price) -> Self {
        self.entry_price = Some(price);
        self
    }

    /// Attach a recommended stop loss level.
    pub fn with_stop_loss(mut self, stop_loss: Price) -> Self {
        self.stop_loss = Some(stop_loss);
        self
    }

    /// Attach a recommended take profit level.
    pub fn with_take_profit(mut self, take_profit: Price) -> Self {
        self.take_profit = Some(take_profit);
        self
    }

    /// Attach a volatility estimate (e.g. ATR).
    pub fn with_volatility_atr(mut self, volatility_atr: Decimal) -> Self {
        self.volatility_atr = Some(volatility_atr);
        self
    }
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
