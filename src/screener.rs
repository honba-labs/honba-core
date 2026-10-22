//! Screener/scan wire contracts.

use std::collections::BTreeMap;

use rust_decimal::Decimal;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::signal::Timeframe;
use crate::types::InstrumentId;

/// Granularity for a screener metric's lookback window.
///
/// Reuses [`Timeframe`] rather than defining a second, incompatible period
/// enum — a metric's period and a bar's interval are the same concept.
pub type MetricPeriod = Timeframe;

/// Comparison operator for a screener filter predicate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum FilterOp {
    /// Equal to.
    Eq,
    /// Not equal to.
    Ne,
    /// Greater than.
    Gt,
    /// Greater than or equal to.
    Gte,
    /// Less than.
    Lt,
    /// Less than or equal to.
    Lte,
    /// Inclusively between two values.
    Between,
}

/// A lightweight key naming a metric at a given period (e.g. `"rsi"` @ `D1`).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
pub struct MetricKeySpec {
    /// Metric name (e.g. `"rsi_14"`, `"volume"`).
    pub metric: String,
    /// Lookback granularity.
    pub period: MetricPeriod,
}

/// Describes a metric the screener exposes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct MetricDefinition {
    /// The metric's key.
    pub key: MetricKeySpec,
    /// Human-readable description.
    pub description: String,
    /// Unit, if applicable (e.g. `"%"`, `"INR"`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unit: Option<String>,
}

/// A reference to a metric value, used inside filters and sort specs.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(transparent)]
pub struct MetricRef(pub MetricKeySpec);

/// One filter condition in a screener scan.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ScreenerFilterPredicate {
    /// Metric being filtered.
    pub metric: MetricRef,
    /// Comparison operator.
    pub op: FilterOp,
    /// Comparison value.
    pub value: Decimal,
    /// Upper bound, only used when `op` is [`FilterOp::Between`].
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value_upper: Option<Decimal>,
}

/// A group of filter predicates, combined with AND or OR.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ScreenerFilterGroup {
    /// The predicates in this group.
    pub predicates: Vec<ScreenerFilterPredicate>,
    /// `true` to AND all predicates together, `false` to OR them.
    pub match_all: bool,
}

/// How to order screener results.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct SortSpec {
    /// Metric to sort by.
    pub metric: MetricRef,
    /// `true` for descending order.
    pub descending: bool,
}

/// Request to scan instruments against a set of metric filters.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ScreenerScanRequest {
    /// Instruments to scan. `None` means the full universe.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub universe: Option<Vec<InstrumentId>>,
    /// Filter predicates to apply.
    pub filters: ScreenerFilterGroup,
    /// Sort order for the result set.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sort: Option<SortSpec>,
    /// Maximum rows to return.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
}

/// One instrument's row in a screener result, keyed by metric name.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ScreenerRow {
    /// The matching instrument.
    pub instrument: InstrumentId,
    /// Metric name → value, for every metric the request asked about.
    pub metrics: BTreeMap<String, Decimal>,
}

/// The matching rows for a [`ScreenerScanRequest`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ScreenerScanResponse {
    /// Matching rows, after sort/limit.
    pub rows: Vec<ScreenerRow>,
    /// Total rows that matched the filter, before `limit` was applied.
    pub total_matched: u32,
}
