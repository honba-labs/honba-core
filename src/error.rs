//! Error types for honba-core.

use thiserror::Error;

/// Core domain errors.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum CoreError {
    /// Invalid exchange identifier
    #[error("unknown exchange: '{0}' (expected NSE, BSE, or MCX)")]
    InvalidExchange(String),

    /// Invalid decimal string for price
    #[error("failed to parse decimal price: '{0}'")]
    InvalidPrice(String),

    /// Negative price
    #[error("price cannot be negative: '{0}'")]
    NegativePrice(String),

    /// Validation error for decisions or requests
    #[error("validation failure: {0}")]
    Validation(String),
}
