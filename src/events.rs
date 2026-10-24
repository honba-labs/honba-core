//! Cross-subsystem domain events (immutable, propagate state changes —
//! see `AGENTS.md` §2.2).

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::instrument::PositionSide;
use crate::order::{FillReport, OrderEvent, OrderSide};
use crate::sizing::{PositionSizingRequest, PositionSizingResult};
use crate::types::{InstrumentId, Money, Price, Quantity};
use crate::wire::TraceContext;

/// Whether a [`Reason`] favors or disfavors the action it's attached to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ReasonPolarity {
    /// Favors the action.
    Positive,
    /// Disfavors the action.
    Negative,
    /// Neither.
    Neutral,
}

/// Which subsystem a [`Reason`] originated from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ReasonGroup {
    /// Pre-trade risk gate (`honba-mon`).
    Risk,
    /// Strategy/indicator signal.
    Signal,
    /// Execution/order routing.
    Execution,
    /// Regulatory/compliance check.
    Compliance,
}

/// A single structured reason attached to a decision or refusal, for audit
/// trails and diagnostics.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct Reason {
    /// Originating subsystem.
    pub group: ReasonGroup,
    /// Whether this favors or disfavors the action.
    pub polarity: ReasonPolarity,
    /// Short machine-stable code (e.g. `"otr_breached"`).
    pub code: String,
    /// Human-readable detail.
    pub message: String,
}

/// Cross-subsystem execution-audit event.
///
/// Wraps [`OrderEvent`] (post-submission order lifecycle) and adds the
/// pre-trade risk-gate outcome, which happens *before* an order ever reaches
/// `OrderEvent`'s scope — a risk refusal never produces an order at all, so
/// it can't be represented as one of `OrderEvent`'s variants.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ExecutionEvent {
    /// An order request was submitted for risk evaluation.
    Submitted {
        /// This platform's order id.
        order_id: String,
        /// The instrument being traded.
        instrument: InstrumentId,
    },
    /// The pre-trade risk gate refused the order before it reached the
    /// exchange.
    RiskRefused {
        /// This platform's order id.
        order_id: String,
        /// Structured reasons for the refusal.
        reasons: Vec<Reason>,
    },
    /// A post-submission order lifecycle event.
    Order(OrderEvent),
}

/// Global or per-algo trading status.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum TradingState {
    /// Trading is active.
    Active,
    /// Trading is halted indefinitely.
    Halted {
        /// Why it was halted.
        reason: String,
    },
    /// Trading is paused, with an optional scheduled resume time.
    Paused {
        /// When trading is expected to resume, if known.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        resume_at: Option<DateTime<Utc>>,
    },
}

/// Tactical DDD aggregate domain events representing authoritative platform state transitions.
///
/// Immutable event stream payloads designed for distributed audit logging,
/// event-sourced ledger reconstruction, and asynchronous cross-bounded-context bus delivery.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum DomainEvent {
    /// Client order dispatched with registered `algo_id`, side, quantity, limit price, and timestamp.
    OrderPlaced {
        /// Platform order identifier.
        order_id: String,
        /// Client correlation identifier.
        client_order_id: Uuid,
        /// Target instrument traded.
        instrument: InstrumentId,
        /// Registered algorithmic strategy identifier.
        algo_id: String,
        /// Direction of order.
        side: OrderSide,
        /// Order quantity.
        quantity: Quantity,
        /// Limit price, if limit or stop-limit.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        price: Option<Price>,
        /// Event creation timestamp.
        timestamp: DateTime<Utc>,
        /// Distributed trace context.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        trace_context: Option<TraceContext>,
    },
    /// In-flight order price/quantity amendment with original and revised parameters.
    OrderModified {
        /// Platform order identifier.
        order_id: String,
        /// Client correlation identifier.
        client_order_id: Uuid,
        /// Target instrument.
        instrument: InstrumentId,
        /// Registered algo ID.
        algo_id: String,
        /// Pre-modification quantity.
        previous_quantity: Quantity,
        /// Revised quantity.
        new_quantity: Quantity,
        /// Pre-modification price.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        previous_price: Option<Price>,
        /// Revised price.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        new_price: Option<Price>,
        /// Event timestamp.
        timestamp: DateTime<Utc>,
        /// Distributed trace context.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        trace_context: Option<TraceContext>,
    },
    /// Order withdrawal with optional cancellation source and reason.
    OrderCancelled {
        /// Platform order identifier.
        order_id: String,
        /// Client correlation identifier.
        client_order_id: Uuid,
        /// Target instrument.
        instrument: InstrumentId,
        /// Associated algo identifier, if known.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        algo_id: Option<String>,
        /// Cancellation origin (e.g. user, rms, market_close).
        #[serde(default, skip_serializing_if = "Option::is_none")]
        source: Option<String>,
        /// Detail or rationale for cancellation.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        reason: Option<String>,
        /// Event timestamp.
        timestamp: DateTime<Utc>,
        /// Distributed trace context.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        trace_context: Option<TraceContext>,
    },
    /// Fill execution details referencing matched [`FillReport`].
    OrderFilled {
        /// Authoritative fill execution report.
        fill: Box<FillReport>,
        /// Fill event timestamp.
        timestamp: DateTime<Utc>,
        /// Distributed trace context.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        trace_context: Option<TraceContext>,
    },
    /// Pre-trade risk gate denial event carrying rule identifier, attempted value, and threshold limit.
    RiskBreached {
        /// Platform order identifier, if assigned.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        order_id: Option<String>,
        /// Target instrument evaluated.
        instrument: InstrumentId,
        /// Identifier of the risk rule that triggered rejection.
        rule_id: String,
        /// Numeric value that was attempted (e.g. order notional, quantity).
        attempted_value: Decimal,
        /// Maximum or minimum threshold allowed by the risk rule.
        threshold_limit: Decimal,
        /// Human-readable explanation.
        message: String,
        /// Event timestamp.
        timestamp: DateTime<Utc>,
        /// Distributed trace context.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        trace_context: Option<TraceContext>,
    },
    /// Emergency square-off or MIS auto-liquidation notification.
    PositionLiquidated {
        /// Target trading account.
        account_id: String,
        /// Liquidated instrument.
        instrument: InstrumentId,
        /// Direction of liquidation trade.
        side: PositionSide,
        /// Liquidated quantity.
        quantity: Quantity,
        /// Average exit price.
        price: Price,
        /// Reason for auto-liquidation (e.g. RMS intraday auto-square-off).
        reason: String,
        /// Event timestamp.
        timestamp: DateTime<Utc>,
        /// Distributed trace context.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        trace_context: Option<TraceContext>,
    },
    /// Intraday margin shortfall alert with required maintenance equity.
    MarginCallIssued {
        /// Account identifier.
        account_id: String,
        /// Total margin equity required to maintain open positions.
        required_maintenance_equity: Money,
        /// Current available account equity.
        current_equity: Money,
        /// Cash shortfall amount required to cure the call.
        shortfall: Money,
        /// Event timestamp.
        timestamp: DateTime<Utc>,
        /// Distributed trace context.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        trace_context: Option<TraceContext>,
    },
    /// Sizing determination event capturing sizing model inputs, computed quantities, and capping rules.
    SizingCalculated {
        /// Dynamic position sizing request.
        request: Box<PositionSizingRequest>,
        /// Sizing outcome and constraints.
        result: PositionSizingResult,
        /// Event timestamp.
        timestamp: DateTime<Utc>,
        /// Distributed trace context.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        trace_context: Option<TraceContext>,
    },
}

impl DomainEvent {
    /// Returns the creation timestamp of this event.
    pub fn timestamp(&self) -> DateTime<Utc> {
        match self {
            Self::OrderPlaced { timestamp, .. } => *timestamp,
            Self::OrderModified { timestamp, .. } => *timestamp,
            Self::OrderCancelled { timestamp, .. } => *timestamp,
            Self::OrderFilled { timestamp, .. } => *timestamp,
            Self::RiskBreached { timestamp, .. } => *timestamp,
            Self::PositionLiquidated { timestamp, .. } => *timestamp,
            Self::MarginCallIssued { timestamp, .. } => *timestamp,
            Self::SizingCalculated { timestamp, .. } => *timestamp,
        }
    }

    /// Returns the optional distributed trace context associated with this event.
    pub fn trace_context(&self) -> Option<TraceContext> {
        match self {
            Self::OrderPlaced { trace_context, .. } => *trace_context,
            Self::OrderModified { trace_context, .. } => *trace_context,
            Self::OrderCancelled { trace_context, .. } => *trace_context,
            Self::OrderFilled { trace_context, .. } => *trace_context,
            Self::RiskBreached { trace_context, .. } => *trace_context,
            Self::PositionLiquidated { trace_context, .. } => *trace_context,
            Self::MarginCallIssued { trace_context, .. } => *trace_context,
            Self::SizingCalculated { trace_context, .. } => *trace_context,
        }
    }
}
