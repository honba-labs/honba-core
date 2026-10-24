//! Canonical order models, state machine statuses, and execution reports.

use crate::types::{InstrumentId, Money, Price, Quantity};
use crate::wire::TraceContext;
use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

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

/// Cancel request for an existing order.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct CancelRequest {
    /// Instrument to cancel
    pub instrument: InstrumentId,
    /// Client order ID of the order to cancel
    pub client_order_id: Uuid,
    /// Algo ID to cancel (if None, cancel all algos for this instrument)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub algo_id: Option<String>,
    /// Optional client ID
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub client_id: Option<String>,
}

/// Modify request for an existing order.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ModifyRequest {
    /// Instrument to modify
    pub instrument: InstrumentId,
    /// New quantity (None leaves quantity unchanged)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub new_qty: Option<Quantity>,
    /// New limit price if applicable (None leaves price unchanged)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub new_price: Option<Price>,
    /// Algo ID associated with the modification
    pub algo_id: String,
    /// Optional client ID
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub client_id: Option<String>,
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
    /// Optional client ID / account ID
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub client_id: Option<String>,
    /// Optional correlation ID for tracing across async actor hops
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub correlation_id: Option<Uuid>,
    /// Optional W3C distributed trace context
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trace_context: Option<TraceContext>,
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
        let client_order_id = Uuid::new_v4();
        Self {
            client_order_id,
            instrument,
            side,
            order_type,
            quantity,
            tif: TimeInForce::Day,
            algo_id: algo_id.into(),
            tag: None,
            client_id: None,
            correlation_id: Some(client_order_id),
            trace_context: None,
        }
    }

    /// Attach a client or account ID.
    pub fn with_client_id(mut self, client_id: impl Into<String>) -> Self {
        self.client_id = Some(client_id.into());
        self
    }

    /// Attach an explicit correlation ID.
    pub fn with_correlation_id(mut self, correlation_id: Uuid) -> Self {
        self.correlation_id = Some(correlation_id);
        self
    }

    /// Attach an explicit W3C trace context.
    pub fn with_trace_context(mut self, trace_context: TraceContext) -> Self {
        self.trace_context = Some(trace_context);
        self
    }

    /// Get correlation ID, falling back to client_order_id.
    pub fn correlation_id(&self) -> Uuid {
        self.correlation_id.unwrap_or(self.client_order_id)
    }

    /// Get client ID or fallback to "default".
    pub fn client_id_str(&self) -> &str {
        self.client_id.as_deref().unwrap_or("default")
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
    /// Distributed trace correlation ID
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub correlation_id: Option<Uuid>,
    /// Optional client ID
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub client_id: Option<String>,
    /// Optional W3C distributed trace context
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trace_context: Option<TraceContext>,
}

impl OrderAck {
    /// Create a new order acknowledgement.
    pub fn new(
        order_id: impl Into<String>,
        client_order_id: Uuid,
        status: OrderStatus,
        timestamp: DateTime<Utc>,
    ) -> Self {
        Self {
            order_id: order_id.into(),
            client_order_id,
            exchange_order_id: None,
            status,
            timestamp,
            correlation_id: Some(client_order_id),
            client_id: None,
            trace_context: None,
        }
    }

    /// Attach an explicit W3C trace context.
    pub fn with_trace_context(mut self, trace_context: TraceContext) -> Self {
        self.trace_context = Some(trace_context);
        self
    }

    /// Returns correlation ID or falls back to client_order_id.
    pub fn correlation_id(&self) -> Uuid {
        self.correlation_id.unwrap_or(self.client_order_id)
    }

    /// Returns client ID or "default".
    pub fn client_id_str(&self) -> &str {
        self.client_id.as_deref().unwrap_or("default")
    }
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
    /// Distributed trace correlation ID
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub correlation_id: Option<Uuid>,
    /// Optional client ID
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub client_id: Option<String>,
    /// Total transaction cost charged on this fill (brokerage, statutory
    /// levies, and exchange charges combined), in the fill's currency.
    ///
    /// Defaults to zero for callers that don't yet have cost data (e.g. a
    /// simulated/paper fill). The statutory calculation itself lives in
    /// `honba-me` (`honba-me-market::india::costs`); this crate only owns
    /// the contract.
    #[serde(default = "Money::zero_inr")]
    pub fee: Money,
    /// Itemized breakdown of [`FillReport::fee`], when the caller has it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fee_breakdown: Option<FeeBreakdown>,
    /// Optional W3C distributed trace context
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trace_context: Option<TraceContext>,
}

impl FillReport {
    /// Create a new fill report.
    ///
    /// `fee` defaults to zero — use [`FillReport::with_fee`] or
    /// [`FillReport::with_fee_breakdown`] once cost data is available.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        fill_id: impl Into<String>,
        order_id: impl Into<String>,
        client_order_id: Uuid,
        instrument: InstrumentId,
        side: OrderSide,
        price: Price,
        quantity: Quantity,
        algo_id: impl Into<String>,
        timestamp: DateTime<Utc>,
    ) -> Self {
        Self {
            fill_id: fill_id.into(),
            order_id: order_id.into(),
            client_order_id,
            instrument,
            side,
            price,
            quantity,
            algo_id: algo_id.into(),
            timestamp,
            correlation_id: Some(client_order_id),
            client_id: None,
            fee: Money::zero_inr(),
            fee_breakdown: None,
            trace_context: None,
        }
    }

    /// Returns correlation ID or falls back to client_order_id.
    pub fn correlation_id(&self) -> Uuid {
        self.correlation_id.unwrap_or(self.client_order_id)
    }

    /// Returns client ID or "default".
    pub fn client_id_str(&self) -> &str {
        self.client_id.as_deref().unwrap_or("default")
    }

    /// Attaches a total fee with no itemized breakdown.
    pub fn with_fee(mut self, fee: Money) -> Self {
        self.fee = fee;
        self
    }

    /// Attaches an itemized fee breakdown, setting [`FillReport::fee`] to its total.
    pub fn with_fee_breakdown(mut self, breakdown: FeeBreakdown) -> Self {
        self.fee = breakdown.total();
        self.fee_breakdown = Some(breakdown);
        self
    }

    /// Attach an explicit W3C trace context.
    pub fn with_trace_context(mut self, trace_context: TraceContext) -> Self {
        self.trace_context = Some(trace_context);
        self
    }
}

/// Itemized Indian equity/F&O transaction cost breakdown for a single fill.
///
/// Each component is computed by `honba-me-market::india::costs`'s statutory
/// calculators; this type is just the shared wire shape they fill in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct FeeBreakdown {
    /// Broker commission.
    pub brokerage: Money,
    /// Securities Transaction Tax.
    pub stt: Money,
    /// Exchange transaction charges.
    pub exchange_charges: Money,
    /// Goods and Services Tax (on brokerage + exchange charges).
    pub gst: Money,
    /// Stamp duty.
    pub stamp_duty: Money,
    /// SEBI turnover fees.
    pub sebi_charges: Money,
}

impl FeeBreakdown {
    /// Sums every component into the total fee. All components are expected
    /// to share one currency (SEBI-regulated Indian venues settle in INR);
    /// this just adds the exact `Decimal` amounts and keeps the first
    /// component's currency tag.
    pub fn total(&self) -> Money {
        let amount = self.brokerage.amount
            + self.stt.amount
            + self.exchange_charges.amount
            + self.gst.amount
            + self.stamp_duty.amount
            + self.sebi_charges.amount;
        Money::new(amount, self.brokerage.currency)
    }
}

/// Reason an order or modification was refused after submission (by the
/// exchange/broker, or by expiring unfilled).
///
/// Distinct from a pre-trade risk-gate refusal, which never produces an
/// [`OrderEvent`] at all — see [`crate::events::ExecutionEvent::RiskRefused`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "reason", content = "detail", rename_all = "snake_case")]
pub enum RejectReason {
    /// Rejected by the exchange or broker, with their message.
    ExchangeRejected(String),
    /// Expired unfilled per its time-in-force.
    Expired,
    /// Refused for insufficient margin.
    InsufficientMargin,
}

/// Tag-only mirror of [`OrderEvent`]'s variants, for filtering/logging
/// without carrying the full payload.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum OrderEventKind {
    /// See [`OrderEvent::Acked`].
    Acked,
    /// See [`OrderEvent::Filled`].
    Filled,
    /// See [`OrderEvent::Rejected`].
    Rejected,
    /// See [`OrderEvent::Cancelled`].
    Cancelled,
}

/// A single lifecycle event for an order, wrapping the existing
/// acknowledgement/fill/rejection/cancellation payloads into one stream type.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum OrderEvent {
    /// The order was acknowledged by the broker/exchange.
    Acked(OrderAck),
    /// The order (or part of it) was filled.
    Filled(Box<FillReport>),
    /// The order was rejected after submission.
    Rejected {
        /// This platform's order id.
        order_id: String,
        /// Why it was rejected.
        reason: RejectReason,
    },
    /// The order was cancelled.
    Cancelled {
        /// This platform's order id.
        order_id: String,
    },
}

impl OrderEvent {
    /// Returns this event's tag-only [`OrderEventKind`].
    pub fn kind(&self) -> OrderEventKind {
        match self {
            Self::Acked(_) => OrderEventKind::Acked,
            Self::Filled(_) => OrderEventKind::Filled,
            Self::Rejected { .. } => OrderEventKind::Rejected,
            Self::Cancelled { .. } => OrderEventKind::Cancelled,
        }
    }
}
