//! Portfolio snapshot wire contracts.
//!
//! These cross the Heart engine (`honba-kokoro`) → House API (`honba-ya`) →
//! Face UI (`honba-kao`) boundary, so they're Kernel wire types, not
//! kokoro-internal engine state.

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::instrument::PositionSide;
use crate::types::{InstrumentId, Money, Price, Quantity};

/// Where a portfolio's numbers come from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum PortfolioMode {
    /// Real money, real broker.
    Live,
    /// Simulated fills against a real broker account/paper account.
    Paper,
    /// Historical backtest run.
    Backtest,
}

/// Where a portfolio's positions were sourced from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum PortfolioSource {
    /// Reconciled from the broker's own books.
    Broker,
    /// Tracked entirely in-process (paper/backtest).
    Simulated,
}

/// An open intraday or carried position.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct PortfolioPosition {
    /// The instrument held.
    pub instrument: InstrumentId,
    /// Net direction.
    pub side: PositionSide,
    /// Net quantity held.
    pub quantity: Quantity,
    /// Volume-weighted average entry price.
    pub avg_price: Price,
    /// Mark-to-market profit/loss, not yet realized.
    pub unrealized_pnl: Money,
    /// Profit/loss already realized on this instrument today.
    pub realized_pnl: Money,
}

/// A T+1 delivery holding (as opposed to an intraday position).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct PortfolioHolding {
    /// The instrument held.
    pub instrument: InstrumentId,
    /// Quantity held.
    pub quantity: Quantity,
    /// Volume-weighted average acquisition price.
    pub avg_price: Price,
    /// Current mark-to-market value of the holding.
    pub current_value: Money,
}

/// Aggregate options Greeks for a portfolio (or a single position).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct PortfolioGreeks {
    /// Price sensitivity.
    pub delta: Decimal,
    /// Delta's sensitivity to price.
    pub gamma: Decimal,
    /// Time decay.
    pub theta: Decimal,
    /// Sensitivity to implied volatility.
    pub vega: Decimal,
}

/// Account-level cash and margin totals.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct PortfolioTotals {
    /// Free cash balance.
    pub cash: Money,
    /// Margin currently blocked by open positions/orders.
    pub margin_used: Money,
    /// Margin still available to deploy.
    pub margin_available: Money,
    /// Total profit/loss, realized plus unrealized.
    pub total_pnl: Money,
    /// Net worth: cash plus mark-to-market value of all holdings/positions.
    pub net_worth: Money,
}

/// Identifies the account a snapshot belongs to.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct PortfolioAccount {
    /// Broker or internal account identifier.
    pub account_id: String,
    /// Live, paper, or backtest.
    pub mode: PortfolioMode,
    /// Broker-reconciled or simulated.
    pub source: PortfolioSource,
}

/// A full point-in-time view of an account's positions, holdings, and
/// totals — the wire contract served by the portfolio API and consumed by
/// the dashboard.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct PortfolioSnapshot {
    /// Which account this snapshot describes.
    pub account: PortfolioAccount,
    /// Open intraday/carried positions.
    pub positions: Vec<PortfolioPosition>,
    /// T+1 delivery holdings.
    pub holdings: Vec<PortfolioHolding>,
    /// Cash/margin/PnL totals.
    pub totals: PortfolioTotals,
    /// Aggregate options Greeks, if the account holds any options.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub greeks: Option<PortfolioGreeks>,
    /// When this snapshot was taken.
    pub timestamp: DateTime<Utc>,
}
