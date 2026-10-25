//! Error types for honba-core.

use thiserror::Error;

/// Core domain errors.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum CoreError {
    /// Invalid exchange identifier
    #[error("unknown exchange: '{0}' (expected NSE, BSE, or MCX)")]
    InvalidExchange(String),

    /// Unknown country code
    #[error("unknown country: '{0}' (expected IN, US, UK, or JP)")]
    InvalidCountry(String),

    /// Invalid decimal string for price
    #[error("failed to parse decimal price: '{0}'")]
    InvalidPrice(String),

    /// Negative price
    #[error("price cannot be negative: '{0}'")]
    NegativePrice(String),

    /// Unknown currency code
    #[error("unknown currency: '{0}' (expected INR, USD, EUR, or GBP)")]
    InvalidCurrency(String),

    /// Validation error for decisions or requests
    #[error("validation failure: {0}")]
    Validation(String),
}
