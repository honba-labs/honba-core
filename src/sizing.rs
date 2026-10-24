//! Canonical dynamic position sizing grammar, risk budgets, and deterministic math.
//!
//! Decouples alpha signal generation ("what to trade") from position sizing
//! ("how much to trade"). Pure functional execution with 100% exact 128-bit decimal
//! arithmetic (`rust_decimal::Decimal`), strict zero-I/O purity, and zero floats.

use rust_decimal::Decimal;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::error::CoreError;
use crate::instrument::PositionSide;
use crate::types::{InstrumentId, Money, Price, Quantity};

/// Permissible dynamic position sizing models.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum SizingMethod {
    /// Risk a fixed fraction/percentage of account equity per trade (e.g. 0.01 for 1%).
    FixedFractional {
        /// Risk fraction of equity (e.g. 0.01 represents 1%).
        risk_pct: Decimal,
    },
    /// Size inversely proportional to instrument volatility (ATR or standard deviation).
    VolatilityTarget {
        /// Target annualized volatility (e.g. 0.15 for 15%).
        target_annual_vol: Decimal,
        /// Lookback window in bars used for volatility computation.
        lookback_bars: usize,
    },
    /// Optimal growth sizing with damping fraction (e.g. half-Kelly).
    KellyCriterion {
        /// Kelly multiplier fraction (e.g. 0.5 for half-Kelly).
        fraction: Decimal,
        /// Expected historical win rate in `[0, 1]`.
        win_rate: Decimal,
        /// Payoff ratio (average win / average loss).
        payoff_ratio: Decimal,
    },
    /// Allocate risk budget equally across all active signal candidates.
    EqualRiskContribution,
    /// Allocate a fixed cash notional dynamically converted to lot-rounded quantity.
    FixedNotional {
        /// Fixed cash amount to allocate.
        notional: Money,
    },
}

/// Risk parameters and sizing constraints for a trade or portfolio.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct RiskBudget {
    /// Total account equity available for risk allocation.
    pub equity: Money,
    /// Default risk fraction per trade (e.g. 0.01 for 1%).
    pub risk_pct: Decimal,
    /// Maximum position notional cash cap, if configured.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_position_notional: Option<Money>,
    /// Maximum position share or contract quantity cap, if configured.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_position_qty: Option<Quantity>,
    /// Maximum portfolio aggregate risk percentage, if configured.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_portfolio_risk_pct: Option<Decimal>,
    /// Instrument minimum lot size (e.g. 1 for equities, 25/50 for index derivatives).
    pub min_lot_size: Quantity,
    /// Scrip freeze limit (exchange-mandated single order quantity ceiling), if configured.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub freeze_limit_qty: Option<Quantity>,
}

impl RiskBudget {
    /// Creates a new risk budget with baseline parameters.
    pub fn new(equity: Money, risk_pct: Decimal, min_lot_size: Quantity) -> Self {
        Self {
            equity,
            risk_pct,
            max_position_notional: None,
            max_position_qty: None,
            max_portfolio_risk_pct: None,
            min_lot_size,
            freeze_limit_qty: None,
        }
    }

    /// Set a maximum position notional cash cap.
    pub fn with_max_notional(mut self, max_notional: Money) -> Self {
        self.max_position_notional = Some(max_notional);
        self
    }

    /// Set a maximum position share or contract quantity cap.
    pub fn with_max_qty(mut self, max_qty: Quantity) -> Self {
        self.max_position_qty = Some(max_qty);
        self
    }

    /// Set a scrip freeze limit.
    pub fn with_freeze_limit(mut self, freeze_qty: Quantity) -> Self {
        self.freeze_limit_qty = Some(freeze_qty);
        self
    }

    /// Set a maximum aggregate portfolio risk percentage.
    pub fn with_max_portfolio_risk(mut self, max_risk: Decimal) -> Self {
        self.max_portfolio_risk_pct = Some(max_risk);
        self
    }
}

/// Request to compute dynamic position sizing for an instrument trade intent.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct PositionSizingRequest {
    /// Target instrument to size.
    pub instrument: InstrumentId,
    /// Side of the planned trade.
    pub side: PositionSide,
    /// Expected entry price.
    pub entry_price: Price,
    /// Stop loss price, if defined.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stop_loss: Option<Price>,
    /// Current instrument volatility (ATR or price standard deviation), if computed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub current_volatility: Option<Decimal>,
    /// Available account equity.
    pub account_equity: Money,
    /// Risk budget parameters and limits.
    pub risk_budget: RiskBudget,
    /// Dynamic position sizing model.
    pub sizing_method: SizingMethod,
}

impl PositionSizingRequest {
    /// Create a new sizing request.
    pub fn new(
        instrument: InstrumentId,
        side: PositionSide,
        entry_price: Price,
        account_equity: Money,
        risk_budget: RiskBudget,
        sizing_method: SizingMethod,
    ) -> Self {
        Self {
            instrument,
            side,
            entry_price,
            stop_loss: None,
            current_volatility: None,
            account_equity,
            risk_budget,
            sizing_method,
        }
    }

    /// Attach a stop loss price.
    pub fn with_stop_loss(mut self, stop_loss: Price) -> Self {
        self.stop_loss = Some(stop_loss);
        self
    }

    /// Attach a current volatility estimate.
    pub fn with_volatility(mut self, volatility: Decimal) -> Self {
        self.current_volatility = Some(volatility);
        self
    }
}

/// Output of dynamic position sizing.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct PositionSizingResult {
    /// Target instrument.
    pub instrument: InstrumentId,
    /// Computed executable quantity, strictly rounded down to lot size multiples.
    pub quantity: Quantity,
    /// Allocated cash notional (`entry_price * quantity`).
    pub allocated_notional: Money,
    /// Total capital at risk (`|entry_price - stop_loss| * quantity`).
    pub risk_at_stake: Money,
    /// Target portfolio weight (`allocated_notional / account_equity`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_weight: Option<Decimal>,
    /// Whether the calculated quantity was reduced by any limit or cap.
    pub capped_by_limit: bool,
    /// Human-readable explanation or capping detail, if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

/// Pure deterministic position sizing calculation using 128-bit decimal arithmetic.
///
/// Guaranteed zero floating-point drift, deterministic lot rounding, and strict
/// adherence to risk budget constraints.
pub fn calculate_position_size(
    request: &PositionSizingRequest,
) -> Result<PositionSizingResult, CoreError> {
    if !request.entry_price.is_positive() {
        return Err(CoreError::Validation(
            "entry price must be strictly positive".to_string(),
        ));
    }

    let equity = request.account_equity.amount;
    let entry_p = request.entry_price.value();
    let currency = request.account_equity.currency;

    if equity <= Decimal::ZERO {
        return Ok(PositionSizingResult {
            instrument: request.instrument.clone(),
            quantity: Quantity::ZERO,
            allocated_notional: Money::new(Decimal::ZERO, currency),
            risk_at_stake: Money::new(Decimal::ZERO, currency),
            target_weight: Some(Decimal::ZERO),
            capped_by_limit: false,
            reason: Some("account equity is zero or negative".to_string()),
        });
    }

    // 1. Compute raw target quantity before lot rounding and limits
    let raw_qty = match &request.sizing_method {
        SizingMethod::FixedFractional { risk_pct } => {
            let risk_amount = equity * *risk_pct;
            if let Some(sl) = request.stop_loss {
                let dist = (entry_p - sl.value()).abs();
                if dist > Decimal::ZERO {
                    risk_amount / dist
                } else {
                    risk_amount / entry_p
                }
            } else {
                risk_amount / entry_p
            }
        }
        SizingMethod::FixedNotional { notional } => notional.amount / entry_p,
        SizingMethod::VolatilityTarget {
            target_annual_vol, ..
        } => {
            if let Some(vol) = request.current_volatility {
                if vol > Decimal::ZERO {
                    let target_notional = equity * (*target_annual_vol / vol);
                    target_notional / entry_p
                } else {
                    Decimal::ZERO
                }
            } else {
                Decimal::ZERO
            }
        }
        SizingMethod::KellyCriterion {
            fraction,
            win_rate,
            payoff_ratio,
        } => {
            if *payoff_ratio > Decimal::ZERO && *win_rate > Decimal::ZERO {
                let kelly = *win_rate - ((Decimal::ONE - *win_rate) / *payoff_ratio);
                if kelly > Decimal::ZERO {
                    let damped_kelly = kelly * *fraction;
                    let risk_amount = equity * damped_kelly;
                    if let Some(sl) = request.stop_loss {
                        let dist = (entry_p - sl.value()).abs();
                        if dist > Decimal::ZERO {
                            risk_amount / dist
                        } else {
                            risk_amount / entry_p
                        }
                    } else {
                        risk_amount / entry_p
                    }
                } else {
                    Decimal::ZERO
                }
            } else {
                Decimal::ZERO
            }
        }
        SizingMethod::EqualRiskContribution => {
            let risk_amount = equity * request.risk_budget.risk_pct;
            if let Some(sl) = request.stop_loss {
                let dist = (entry_p - sl.value()).abs();
                if dist > Decimal::ZERO {
                    risk_amount / dist
                } else {
                    risk_amount / entry_p
                }
            } else {
                risk_amount / entry_p
            }
        }
    };

    let raw_qty = raw_qty.max(Decimal::ZERO);

    // 2. Lot rounding: strictly round down to multiple of min_lot_size
    let lot_size = if request.risk_budget.min_lot_size.value() > Decimal::ZERO {
        request.risk_budget.min_lot_size.value()
    } else {
        Decimal::ONE
    };

    let num_lots = (raw_qty / lot_size).floor();
    let mut rounded_qty = num_lots * lot_size;

    // 3. Apply bounds, caps, and freeze limits
    let mut capped = false;
    let mut reasons = Vec::new();

    // Check max_position_qty
    if let Some(max_qty) = request.risk_budget.max_position_qty {
        let max_val = max_qty.value();
        if rounded_qty > max_val {
            let allowed_lots = (max_val / lot_size).floor();
            rounded_qty = allowed_lots * lot_size;
            capped = true;
            reasons.push("capped by max_position_qty".to_string());
        }
    }

    // Check freeze_limit_qty
    if let Some(freeze) = request.risk_budget.freeze_limit_qty {
        let freeze_val = freeze.value();
        if rounded_qty > freeze_val {
            let allowed_lots = (freeze_val / lot_size).floor();
            rounded_qty = allowed_lots * lot_size;
            capped = true;
            reasons.push("capped by freeze_limit_qty".to_string());
        }
    }

    // Check max_position_notional
    if let Some(max_notional) = request.risk_budget.max_position_notional {
        let proposed_notional = rounded_qty * entry_p;
        if proposed_notional > max_notional.amount {
            let max_qty_for_notional = (max_notional.amount / entry_p).max(Decimal::ZERO);
            let allowed_lots = (max_qty_for_notional / lot_size).floor();
            rounded_qty = allowed_lots * lot_size;
            capped = true;
            reasons.push("capped by max_position_notional".to_string());
        }
    }

    let final_qty = Quantity::new(rounded_qty);
    let allocated_notional = Money::new(rounded_qty * entry_p, currency);

    let risk_at_stake = if let Some(sl) = request.stop_loss {
        let dist = (entry_p - sl.value()).abs();
        Money::new(dist * rounded_qty, currency)
    } else {
        allocated_notional
    };

    let target_weight = if equity > Decimal::ZERO {
        Some(allocated_notional.amount / equity)
    } else {
        None
    };

    let reason = if reasons.is_empty() {
        None
    } else {
        Some(reasons.join("; "))
    };

    Ok(PositionSizingResult {
        instrument: request.instrument.clone(),
        quantity: final_qty,
        allocated_notional,
        risk_at_stake,
        target_weight,
        capped_by_limit: capped,
        reason,
    })
}
