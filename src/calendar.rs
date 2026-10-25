//! Exchange trading calendar interface and session specifications.
//!
//! Hexagonal port interface ([`TradingCalendar`]) decoupling trading engines,
//! risk gates, and backtesters from specific exchange holiday tables and session hours.

use chrono::{DateTime, FixedOffset, NaiveDate, NaiveTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Session window on a trading date.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct SessionWindow {
    /// Session opening time (local exchange time).
    pub open: NaiveTime,
    /// Session closing time (local exchange time).
    pub close: NaiveTime,
}

impl SessionWindow {
    /// Create a new session window with explicit open and close times.
    pub const fn new(open: NaiveTime, close: NaiveTime) -> Self {
        Self { open, close }
    }

    /// Check if a local time falls within this session window (inclusive).
    pub fn contains(&self, time: NaiveTime) -> bool {
        time >= self.open && time <= self.close
    }
}

/// Abstract exchange / venue trading calendar interface.
///
/// Object-safe for dynamic dispatch (`Arc<dyn TradingCalendar>`).
/// Zero-I/O: calendar state and rules are in-memory and deterministic.
pub trait TradingCalendar: Send + Sync {
    /// Check if a date is a recognized trading day (open session).
    fn is_trading_day(&self, date: NaiveDate) -> bool;

    /// Check if a date is a declared market holiday.
    fn is_holiday(&self, date: NaiveDate) -> bool;

    /// Get the applicable session window for a date, if any.
    fn session_for_date(&self, date: NaiveDate) -> Option<SessionWindow>;

    /// Declared holiday dates as a flat list.
    fn holidays(&self) -> Vec<NaiveDate>;

    /// Local market timezone offset relative to UTC.
    fn timezone_offset(&self) -> FixedOffset;

    /// Check if a UTC timestamp falls within open market hours.
    fn is_market_open(&self, utc_time: DateTime<Utc>) -> bool {
        let local_time = utc_time.with_timezone(&self.timezone_offset());
        let date = local_time.date_naive();
        if let Some(session) = self.session_for_date(date) {
            session.contains(local_time.time())
        } else {
            false
        }
    }

    /// Find the next trading day strictly after `date`.
    fn next_trading_day(&self, mut date: NaiveDate) -> NaiveDate {
        loop {
            date += chrono::Duration::days(1);
            if self.is_trading_day(date) {
                return date;
            }
        }
    }
}
