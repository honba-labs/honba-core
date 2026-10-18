//! Canonical order models, state machine statuses, and execution reports.

use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::CoreError;
use crate::types::{InstrumentId, Price, Quantity};

/// Order side.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "UPPERCASE")]
pub enum OrderSide {
    /// Buy order
    Buy,
    /// Sell order
    Sell,
}

impl OrderSide {
    /// Returns the opposite order side.
    pub fn opposite(&self) -> Self {
        match self {
            Self::Buy => Self::Sell,
            Self::Sell => Self::Buy,
        }
    }
}

/// Order type specification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", content = "price", rename_all = "snake_case")]
pub enum OrderType {
    /// Execute immediately at prevailing market price.
    Market,
    /// Execute at specified limit price or better.
    Limit(Price),
    /// Stop-loss order triggered at trigger price.
    StopLoss(Price),
    /// Stop-limit order with separate trigger and limit prices.
    StopLimit {
        /// Trigger price
        trigger: Price,
        /// Limit price
        limit: Price,
    },
}

impl OrderType {
    /// Returns the limit price if this order has a limit component.
    pub fn limit_price(&self) -> Option<Price> {
        match self {
            Self::Limit(p) => Some(*p),
            Self::StopLimit { limit, .. } => Some(*limit),
            _ => None,
        }
    }

    /// Returns the trigger price if this is a stop order.
    pub fn trigger_price(&self) -> Option<Price> {
        match self {
            Self::StopLoss(p) => Some(*p),
            Self::StopLimit { trigger, .. } => Some(*trigger),
            _ => None,
        }
    }
}

/// Time in force.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema, Default)]
#[serde(rename_all = "UPPERCASE")]
pub enum TimeInForce {
    /// Order valid for the trading day.
    #[default]
    Day,
    /// Immediate or Cancel (unfilled portion is cancelled).
    Ioc,
    /// Good 'Til Cancelled.
    Gtc,
}

/// Lifecycle status of an order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum OrderStatus {
    /// Order created locally, awaiting transmission.
    PendingNew,
    /// Order acknowledged by broker/exchange.
    New,
    /// Order partially filled.
    PartiallyFilled,
    /// Order completely filled.
    Filled,
    /// Order cancelled by user or algorithm.
    Cancelled,
    /// Order rejected by broker or exchange.
    Rejected,
    /// Order expired at market close or TIF expiration.
    Expired,
}

impl OrderStatus {
    /// Returns true if the order is in a final, terminal state.
    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            Self::Filled | Self::Cancelled | Self::Rejected | Self::Expired
        )
    }

    /// Returns true if the order is active and can receive fills or cancellations.
    pub fn is_active(&self) -> bool {
        matches!(self, Self::PendingNew | Self::New | Self::PartiallyFilled)
    }
}

/// Request to place a new order.
///
/// NOTE: In compliance with SEBI's Retail Algo Circular, every order request
/// MUST carry a non-empty `algo_id` uniquely identifying the algorithmic strategy.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct OrderRequest {
    /// Unique client correlation ID
    pub client_order_id: Uuid,
    /// Instrument to trade
    pub instrument: InstrumentId,
    /// Order direction
    pub side: OrderSide,
    /// Pricing instructions
    pub order_type: OrderType,
    /// Quantity to execute
    pub quantity: Quantity,
    /// Validity of the order
    pub tif: TimeInForce,
    /// SEBI-mandated unique algorithm identifier
    pub algo_id: String,
    /// Optional strategy user tag
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tag: Option<String>,
}

impl OrderRequest {
    /// Create a new order request with mandatory algo ID.
    pub fn new(
        instrument: InstrumentId,
        side: OrderSide,
        order_type: OrderType,
        quantity: Quantity,
        algo_id: impl Into<String>,
    ) -> Self {
        Self {
            client_order_id: Uuid::new_v4(),
            instrument,
            side,
            order_type,
            quantity,
            tif: TimeInForce::Day,
            algo_id: algo_id.into(),
            tag: None,
        }
    }

    /// Validate the order request according to SEBI and Indian market rules.
    pub fn validate(&self) -> Result<(), CoreError> {
        if self.algo_id.trim().is_empty() {
            return Err(CoreError::Validation(
                "algo_id is mandatory and cannot be empty per SEBI Retail Algo circular".into(),
            ));
        }
        if !self.quantity.is_positive() {
            return Err(CoreError::Validation(
                "order quantity must be strictly positive".into(),
            ));
        }
        if self.instrument.symbol.trim().is_empty() {
            return Err(CoreError::Validation(
                "instrument symbol cannot be empty".into(),
            ));
        }
        match self.order_type {
            OrderType::Limit(p) => {
                if !p.is_positive() {
                    return Err(CoreError::Validation(
                        "limit price must be strictly positive".into(),
                    ));
                }
            }
            OrderType::StopLoss(p) => {
                if !p.is_positive() {
                    return Err(CoreError::Validation(
                        "stop-loss trigger price must be strictly positive".into(),
                    ));
                }
            }
            OrderType::StopLimit { trigger, limit } => {
                if !trigger.is_positive() {
                    return Err(CoreError::Validation(
                        "stop-limit trigger price must be strictly positive".into(),
                    ));
                }
                if !limit.is_positive() {
                    return Err(CoreError::Validation(
                        "stop-limit limit price must be strictly positive".into(),
                    ));
                }
            }
            OrderType::Market => {}
        }
        Ok(())
    }
}

/// Acknowledgment of order placement from broker or execution engine.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct OrderAck {
    /// Broker-assigned order ID
    pub order_id: String,
    /// Client-specified correlation ID
    pub client_order_id: Uuid,
    /// Exchange-assigned order ID (if already available)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exchange_order_id: Option<String>,
    /// Current status
    pub status: OrderStatus,
    /// Acknowledgment timestamp
    pub timestamp: DateTime<Utc>,
}

/// Execution fill report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct FillReport {
    /// Unique fill identifier
    pub fill_id: String,
    /// Associated broker order ID
    pub order_id: String,
    /// Associated client correlation ID
    pub client_order_id: Uuid,
    /// Traded instrument
    pub instrument: InstrumentId,
    /// Order side
    pub side: OrderSide,
    /// Exact execution price
    pub price: Price,
    /// Executed quantity
    pub quantity: Quantity,
    /// Associated algorithm ID
    pub algo_id: String,
    /// Execution timestamp
    pub timestamp: DateTime<Utc>,
}
