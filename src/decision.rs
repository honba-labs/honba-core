//! Typed AI/LLM Decision Engine types and strict validation harness.
//!
//! Language models are untrusted for direct execution. All LLM decisions
//! must strictly deserialize into the closed [`Decision`] enum, and pass
//! validation before ever reaching the risk gate or execution service.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::error::CoreError;
use crate::types::{Price, Quantity};
use crate::order::OrderSide;

/// Closed enum representing all permissible strategy decisions.
///
/// NOTE: Uses `deny_unknown_fields` to prevent LLMs from injecting hallucinations
/// or extra unauthorized parameters. This is the closed decision grammar
// that feeds into the risk gate and execution service.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "action", rename_all = "snake_case", deny_unknown_fields)]
pub enum Decision {
    /// Place a new order (buy or sell).
    ///
    /// SEBI Retail Algo Circular mandates a non-empty `algo_id`.
    PlaceOrder {
        /// Symbol to trade (e.g. "RELIANCE")
        symbol: String,
        /// Order side
        side: OrderSide,
        /// Desired quantity
        qty: Quantity,
        /// Limit price ceiling (optional; if None, market order requested)
        #[serde(default, skip_serializing_if = "Option::is_none")]
        price_limit: Option<Price>,
        /// Mandatory algo ID for SEBI audit trail
        algo_id: String,
        /// Optional reasoning from the model
        #[serde(default, skip_serializing_if = "Option::is_none")]
        rationale: Option<String>,
    },
    /// Modify an existing order's quantity or price.
    ModifyOrder {
        /// Instrument to modify
        instrument: crate::types::InstrumentId,
        /// New desired quantity (None leaves quantity unchanged)
        #[serde(default, skip_serializing_if = "Option::is_none")]
        new_qty: Option<Quantity>,
        /// New limit price if applicable (None leaves price unchanged)
        #[serde(default, skip_serializing_if = "Option::is_none")]
        new_price: Option<Price>,
        /// Mandatory algo ID for SEBI audit trail
        algo_id: String,
        /// Optional reasoning from the model
        #[serde(default, skip_serializing_if = "Option::is_none")]
        rationale: Option<String>,
    },
    /// Cancel an existing order.
    CancelOrder {
        /// Instrument to cancel
        instrument: crate::types::InstrumentId,
        /// Algo ID to cancel (if None, cancel all algos for this instrument)
        #[serde(default, skip_serializing_if = "Option::is_none")]
        algo_id: Option<String>,
        /// Optional reasoning from the model
        #[serde(default, skip_serializing_if = "Option::is_none")]
        rationale: Option<String>,
    },
    /// Emergency cancel-all decision
    CancelAll {
        /// Algo ID to cancel (if None, cancel all algos)
        #[serde(default, skip_serializing_if = "Option::is_none")]
        algo_id: Option<String>,
        /// Explanation
        #[serde(default, skip_serializing_if = "Option::is_none")]
        reason: Option<String>,
    },
    /// Hold decision (no market action)
    Hold {
        /// Explanation for holding
        #[serde(default, skip_serializing_if = "Option::is_none")]
        reason: Option<String>,
    },
}


impl Decision {
    /// Validate that the decision adheres to structural sanity rules:
    /// - Quantities must be positive.
    /// - Price limits must be positive.
    /// - Symbols must not be empty.
    /// - Algo IDs must not be empty.
    pub fn validate(&self) -> Result<(), CoreError> {
        match self {
            Self::PlaceOrder {
                symbol,
                side: _,
                qty,
                price_limit,
                algo_id,
                ..
            } => {
                if symbol.trim().is_empty() {
                    return Err(CoreError::Validation("Symbol cannot be empty".to_string()));
                }
                if algo_id.trim().is_empty() {
                    return Err(CoreError::Validation("algo_id cannot be empty".to_string()));
                }
                if !qty.is_positive() {
                    return Err(CoreError::Validation(format!(
                        "Quantity must be strictly positive, got {}",
                        qty
                    )));
                }
                if let Some(price) = price_limit {
                    if !price.is_positive() {
                        return Err(CoreError::Validation(format!(
                            "Price limit must be positive, got {}",
                            price
                        )));
                    }
                }
                Ok(())
            }
            Self::ModifyOrder {
                instrument: _,
                new_qty: _,
                new_price: _,
                algo_id,
                ..
            } => {
                if algo_id.trim().is_empty() {
                    return Err(CoreError::Validation("algo_id cannot be empty".to_string()));
                }
                Ok(())
            }
            Self::CancelOrder {
                instrument: _,
                algo_id,
                ..
            } => {
                if algo_id.as_ref().map_or(true, |a| a.trim().is_empty()) {
                    return Err(CoreError::Validation("algo_id cannot be empty".to_string()));
                }
                Ok(())
            }
            Self::CancelAll { algo_id, .. } => {
                if let Some(a) = algo_id {
                    if a.trim().is_empty() {
                        return Err(CoreError::Validation("algo_id cannot be empty".to_string()));
                    }
                }
                Ok(())
            }
            Self::Hold { .. } => Ok(()),
        }
    }

    /// Parse a JSON string into a validated Decision.
    pub fn from_json_str(s: &str) -> Result<Self, CoreError> {
        let decision: Self = serde_json::from_str(s)
            .map_err(|e| CoreError::Validation(format!("Failed to parse LLM decision: {e}")))?;
        decision.validate()?;
        Ok(decision)
    }
}
