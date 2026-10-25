//! Core value types for Indian equities (NSE, BSE, MCX) with Decimal precision.

use rust_decimal::Decimal;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

use crate::error::CoreError;

/// Supported exchanges in the Indian market.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "UPPERCASE")]
pub enum Exchange {
    /// National Stock Exchange of India
    Nse,
    /// Bombay Stock Exchange
    Bse,
    /// Multi Commodity Exchange of India
    Mcx,
}

impl Exchange {
    /// The exchange code as it appears on the wire (`"NSE"`, `"BSE"`, `"MCX"`).
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Nse => "NSE",
            Self::Bse => "BSE",
            Self::Mcx => "MCX",
        }
    }
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
///
/// Ordered by `(exchange, symbol, token, isin)` so it can key a `BTreeMap`
/// and sort a dataset deterministically.
#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
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

    /// Access the underlying exact decimal value.
    pub const fn value(&self) -> Decimal {
        self.0
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

    /// Access the underlying exact decimal value.
    pub const fn value(&self) -> Decimal {
        self.0
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

/// The display names of a currency's minor unit (`paisa` / `paise`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
pub struct MinorUnit {
    /// Singular name of the minor unit (`paisa`, `cent`, `penny`).
    pub singular: &'static str,
    /// Plural name of the minor unit (`paise`, `cents`, `pence`).
    pub plural: &'static str,
}

/// Currency enum.
///
/// The minor-unit table lives here, in the kernel, so every consumer (Rust
/// crates, the Python SDK through `honba._honba.currency_minor_units`) reads one
/// table instead of keeping its own copy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema, Default)]
pub enum Currency {
    /// Indian Rupee
    #[default]
    Inr,
    /// United States Dollar
    Usd,
    /// Euro
    Eur,
    /// British Pound
    Gbp,
}

impl Currency {
    /// Every supported currency, in a stable order.
    pub const ALL: &'static [Currency] =
        &[Currency::Inr, Currency::Usd, Currency::Eur, Currency::Gbp];

    /// The ISO 4217 code (`INR`, `USD`, `EUR`, `GBP`).
    pub const fn code(self) -> &'static str {
        match self {
            Self::Inr => "INR",
            Self::Usd => "USD",
            Self::Eur => "EUR",
            Self::Gbp => "GBP",
        }
    }

    /// The number of decimal places in the major unit: one major unit is
    /// `10**minor_exponent` minor units.
    pub const fn minor_exponent(self) -> u8 {
        match self {
            Self::Inr | Self::Usd | Self::Eur | Self::Gbp => 2,
        }
    }

    /// The display names of this currency's minor unit.
    pub const fn minor_unit(self) -> MinorUnit {
        match self {
            Self::Inr => MinorUnit {
                singular: "paisa",
                plural: "paise",
            },
            Self::Usd | Self::Eur => MinorUnit {
                singular: "cent",
                plural: "cents",
            },
            Self::Gbp => MinorUnit {
                singular: "penny",
                plural: "pence",
            },
        }
    }

    /// Parses an ISO 4217 code, case-insensitively.
    pub fn from_code(code: &str) -> Result<Self, CoreError> {
        match code.to_uppercase().as_str() {
            "INR" => Ok(Self::Inr),
            "USD" => Ok(Self::Usd),
            "EUR" => Ok(Self::Eur),
            "GBP" => Ok(Self::Gbp),
            other => Err(CoreError::InvalidCurrency(other.to_string())),
        }
    }
}

impl fmt::Display for Currency {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.code())
    }
}

impl FromStr for Currency {
    type Err = CoreError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_code(s)
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
    /// Create an amount in a given currency.
    pub const fn new(amount: Decimal, currency: Currency) -> Self {
        Self { amount, currency }
    }

    /// Create amount in INR.
    pub fn inr(amount: Decimal) -> Self {
        Self {
            amount,
            currency: Currency::Inr,
        }
    }

    /// Zero amount in INR — the default for fields with no cost data yet
    /// (e.g. [`crate::order::FillReport::fee`] before a cost calculation runs).
    pub fn zero_inr() -> Self {
        Self::inr(Decimal::ZERO)
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

/// Supported sovereign countries / market jurisdictions.
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
#[serde(rename_all = "UPPERCASE")]
pub enum CountryCode {
    /// India (IN)
    #[default]
    In,
    /// United States of America (US)
    Us,
    /// United Kingdom (UK / GB)
    Uk,
    /// Japan (JP)
    Jp,
}

impl CountryCode {
    /// Two-letter country code (`"IN"`, `"US"`, `"UK"`, `"JP"`).
    pub const fn code(&self) -> &'static str {
        match self {
            Self::In => "IN",
            Self::Us => "US",
            Self::Uk => "UK",
            Self::Jp => "JP",
        }
    }
}

impl fmt::Display for CountryCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.code())
    }
}

impl FromStr for CountryCode {
    type Err = CoreError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "IN" => Ok(Self::In),
            "US" => Ok(Self::Us),
            "UK" | "GB" => Ok(Self::Uk),
            "JP" => Ok(Self::Jp),
            other => Err(CoreError::InvalidCountry(other.to_string())),
        }
    }
}
