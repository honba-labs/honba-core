//! Core value types for Indian equities (NSE, BSE, MCX) with Decimal precision.

use rust_decimal::Decimal;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

use crate::error::CoreError;

/// Supported exchanges in the Indian market.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "UPPERCASE")]
pub enum Exchange {
    /// National Stock Exchange of India
    Nse,
    /// Bombay Stock Exchange
    Bse,
    /// Multi Commodity Exchange of India
    Mcx,
}

impl fmt::Display for Exchange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Nse => write!(f, "NSE"),
            Self::Bse => write!(f, "BSE"),
            Self::Mcx => write!(f, "MCX"),
        }
    }
}

impl FromStr for Exchange {
    type Err = CoreError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "NSE" => Ok(Self::Nse),
            "BSE" => Ok(Self::Bse),
            "MCX" => Ok(Self::Mcx),
            other => Err(CoreError::InvalidExchange(other.to_string())),
        }
    }
}

/// Canonical instrument identifier across all feeds and brokers.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
pub struct InstrumentId {
    /// Exchange venue
    pub exchange: Exchange,
    /// Trading symbol (e.g., "RELIANCE", "TCS", "INFY")
    pub symbol: String,
    /// Broker-assigned instrument token (optional)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,
    /// International Securities Identification Number (optional)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub isin: Option<String>,
}

impl InstrumentId {
    /// Create a new instrument ID with exchange and symbol.
    pub fn new(exchange: Exchange, symbol: impl Into<String>) -> Self {
        Self {
            exchange,
            symbol: symbol.into().to_uppercase(),
            token: None,
            isin: None,
        }
    }

    /// Attach a broker token to the instrument ID.
    pub fn with_token(mut self, token: impl Into<String>) -> Self {
        self.token = Some(token.into());
        self
    }

    /// Attach an ISIN to the instrument ID.
    pub fn with_isin(mut self, isin: impl Into<String>) -> Self {
        self.isin = Some(isin.into());
        self
    }
}

impl fmt::Display for InstrumentId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.exchange, self.symbol)
    }
}

/// Exact decimal price (never floating-point).
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Serialize,
    Deserialize,
    JsonSchema,
    Default,
)]
#[serde(transparent)]
pub struct Price(pub Decimal);

impl Price {
    /// Zero price.
    pub const ZERO: Price = Price(Decimal::ZERO);

    /// Create a new Price from an exact Decimal.
    pub const fn new(d: Decimal) -> Self {
        Self(d)
    }

    /// Parse price in Rupees (e.g. "2450.75").
    pub fn from_rupees(s: &str) -> Result<Self, CoreError> {
        let d = Decimal::from_str(s).map_err(|_| CoreError::InvalidPrice(s.to_string()))?;
        if d < Decimal::ZERO {
            return Err(CoreError::NegativePrice(s.to_string()));
        }
        Ok(Self(d))
    }

    /// Check if price is positive.
    pub fn is_positive(&self) -> bool {
        self.0 > Decimal::ZERO
    }

    /// Check if price conforms to a given tick size (e.g., 0.05 paise).
    pub fn is_tick_aligned(&self, tick_size: Decimal) -> bool {
        if tick_size <= Decimal::ZERO {
            return false;
        }
        (self.0 % tick_size) == Decimal::ZERO
    }
}

impl fmt::Display for Price {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Exact decimal quantity (supports whole shares and lots).
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Serialize,
    Deserialize,
    JsonSchema,
    Default,
)]
#[serde(transparent)]
pub struct Quantity(pub Decimal);

impl Quantity {
    /// Zero quantity.
    pub const ZERO: Quantity = Quantity(Decimal::ZERO);

    /// Create quantity from integer units.
    pub fn from_u64(n: u64) -> Self {
        Self(Decimal::from(n))
    }

    /// Create quantity from Decimal.
    pub const fn new(d: Decimal) -> Self {
        Self(d)
    }

    /// Returns true if quantity is strictly greater than zero.
    pub fn is_positive(&self) -> bool {
        self.0 > Decimal::ZERO
    }

    /// Check if quantity is a multiple of a lot size.
    pub fn is_lot_multiple(&self, lot_size: u64) -> bool {
        if lot_size == 0 {
            return false;
        }
        let lot_dec = Decimal::from(lot_size);
        (self.0 % lot_dec) == Decimal::ZERO
    }
}

impl fmt::Display for Quantity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Currency enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema, Default)]
pub enum Currency {
    /// Indian Rupee
    #[default]
    Inr,
}

impl fmt::Display for Currency {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Inr => write!(f, "INR"),
        }
    }
}

/// Explicit monetary amount with currency.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct Money {
    /// Exact amount
    pub amount: Decimal,
    /// Currency
    pub currency: Currency,
}

impl Money {
    /// Create amount in INR.
    pub fn inr(amount: Decimal) -> Self {
        Self {
            amount,
            currency: Currency::Inr,
        }
    }

    /// Calculate notional value from price and quantity.
    pub fn notional(price: Price, qty: Quantity) -> Self {
        Self {
            amount: price.0 * qty.0,
            currency: Currency::Inr,
        }
    }
}

/// Daily scrip-wise or index circuit limit / price band.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct PriceBand {
    /// Lower price limit (lower circuit)
    pub lower: Price,
    /// Upper price limit (upper circuit)
    pub upper: Price,
    /// Previous close or baseline reference price
    pub reference_price: Price,
    /// Band percentage if known (e.g., 0.05 for 5%, 0.10 for 10%, 0.20 for 20%)
    pub band_pct: Option<Decimal>,
}

impl PriceBand {
    /// Create a price band with explicit bounds.
    pub fn new(lower: Price, upper: Price, reference_price: Price) -> Self {
        Self {
            lower,
            upper,
            reference_price,
            band_pct: None,
        }
    }

    /// Check if a given price falls within the inclusive circuit band [lower, upper].
    pub fn contains(&self, price: Price) -> bool {
        price.0 >= self.lower.0 && price.0 <= self.upper.0
    }
}
