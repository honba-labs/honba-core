//! Instrument reference/master data — distinct from [`crate::types::InstrumentId`],
//! which is only the lookup key.

use std::fmt;

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::types::{CountryCode, Currency, InstrumentId, Price, Quantity};

/// Canonical alias for [`Instrument`] highlighting that it serves as the full reference-data profile.
pub type InstrumentProfile = Instrument;

/// Broad instrument classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum InstrumentKind {
    /// Cash equity.
    Equity,
    /// Futures contract.
    Future,
    /// Options contract.
    Option,
    /// Index (not directly tradable; underlying for derivatives).
    Index,
}

/// Option right.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum OptionType {
    /// Call option.
    Call,
    /// Put option.
    Put,
}

/// Direction of a held or closed position.
///
/// Distinct from [`crate::order::OrderSide`], which is the side of a single
/// order or fill, not the resulting net position.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "UPPERCASE")]
pub enum PositionSide {
    /// Net long.
    Long,
    /// Net short.
    Short,
}

impl PositionSide {
    /// Returns the opposite position side.
    pub fn opposite(&self) -> Self {
        match self {
            Self::Long => Self::Short,
            Self::Short => Self::Long,
        }
    }
}

/// Full reference-data record / profile for a tradable instrument.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct Instrument {
    /// Canonical lookup key.
    pub id: InstrumentId,
    /// Broad classification.
    pub kind: InstrumentKind,
    /// Quoted / base currency for pricing and valuation.
    #[serde(default)]
    pub currency: Currency,
    /// Country / market jurisdiction code (e.g. IN, US, UK, JP).
    #[serde(default)]
    pub country: CountryCode,
    /// Exchange-mandated lot size.
    pub lot_size: Quantity,
    /// Minimum price movement.
    pub tick_size: Decimal,
    /// Expiry date, for futures and options.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expiry: Option<DateTime<Utc>>,
    /// Strike price, for options.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub strike: Option<Price>,
    /// Call or put, for options.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub option_type: Option<OptionType>,
    /// Underlying instrument, for derivatives.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub underlying: Option<InstrumentId>,
}

impl Instrument {
    /// Create a cash-equity instrument record.
    pub fn equity(id: InstrumentId, lot_size: Quantity, tick_size: Decimal) -> Self {
        Self {
            id,
            kind: InstrumentKind::Equity,
            currency: Currency::default(),
            country: CountryCode::default(),
            lot_size,
            tick_size,
            expiry: None,
            strike: None,
            option_type: None,
            underlying: None,
        }
    }

    /// Builder to set quoting currency.
    pub fn with_currency(mut self, currency: Currency) -> Self {
        self.currency = currency;
        self
    }

    /// Builder to set country jurisdiction.
    pub fn with_country(mut self, country: CountryCode) -> Self {
        self.country = country;
        self
    }
}

/// Typed identifier for a historical market-data dataset (distinct from any
/// single instrument or broker).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(transparent)]
pub struct DatasetId(pub String);

impl DatasetId {
    /// Wrap a raw dataset identifier.
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
}

impl fmt::Display for DatasetId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Typed identifier for a single executed trade/fill, distinct from the
/// platform's own order id.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(transparent)]
pub struct TradeId(pub String);

impl TradeId {
    /// Wrap a raw trade identifier.
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
}

impl fmt::Display for TradeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Exchange/broker-assigned order identifier, distinct from this platform's
/// own `order_id` (see [`crate::order::OrderAck::order_id`]).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(transparent)]
pub struct VenueOrderId(pub String);

impl VenueOrderId {
    /// Wrap a raw venue order identifier.
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
}

impl fmt::Display for VenueOrderId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
