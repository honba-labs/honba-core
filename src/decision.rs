//! Typed AI/LLM Decision Engine types and strict validation harness.
//!
//! Language models are untrusted for direct execution. All LLM decisions
//! must strictly deserialize into the closed [`Decision`] enum, and pass
//! validation before ever reaching the risk gate or execution service.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::error::CoreError;
use crate::types::{Exchange, Price, Quantity};

/// Closed enum representing all permissible strategy decisions.
///
/// NOTE: Uses `deny_unknown_fields` to prevent LLMs from injecting hallucinations
/// or extra unauthorized parameters.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "action", rename_all = "snake_case", deny_unknown_fields)]
pub enum Decision {
    /// Buy decision
    Buy {
        /// Symbol to purchase (e.g. "RELIANCE")
        symbol: String,
        /// Target exchange (defaults to NSE if omitted)
        #[serde(default = "default_nse")]
        exchange: Exchange,
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
    /// Sell decision
    Sell {
        /// Symbol to sell
        symbol: String,
        /// Target exchange
        #[serde(default = "default_nse")]
        exchange: Exchange,
        /// Desired quantity
        qty: Quantity,
        /// Limit price floor (optional; if None, market order requested)
        #[serde(default, skip_serializing_if = "Option::is_none")]
        price_limit: Option<Price>,
        /// Mandatory algo ID for SEBI audit trail
        algo_id: String,
        /// Optional reasoning from the model
        #[serde(default, skip_serializing_if = "Option::is_none")]
        rationale: Option<String>,
    },
    /// Hold decision (no market action)
    Hold {
        /// Explanation for holding
        #[serde(default, skip_serializing_if = "Option::is_none")]
        reason: Option<String>,
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
}

fn default_nse() -> Exchange {
    Exchange::Nse
}

impl Decision {
    /// Validate that the decision adheres to structural sanity rules:
    /// - Quantities must be positive.
    /// - Price limits must be positive.
    /// - Symbols must not be empty.
    /// - Algo IDs must not be empty.
    pub fn validate(&self) -> Result<(), CoreError> {
        match self {
            Self::Buy {
                symbol,
                qty,
                price_limit,
                algo_id,
                ..
            }
            | Self::Sell {
                symbol,
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
            Self::Hold { .. } | Self::CancelAll { .. } => Ok(()),
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
