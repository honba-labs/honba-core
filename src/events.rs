//! Cross-subsystem domain events (immutable, propagate state changes —
//! see `AGENTS.md` §2.2).

use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::order::OrderEvent;
use crate::types::InstrumentId;

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
