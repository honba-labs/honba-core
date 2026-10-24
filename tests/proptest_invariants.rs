//! Property-based testing for dimensional invariants, sizing bounds, and wire contracts.

use chrono::Utc;
use honba_core::*;
use proptest::prelude::*;
use rust_decimal::Decimal;
use std::str::FromStr;
use uuid::Uuid;

proptest! {
    // 1. Dimensional invariant: Price * Quantity = Money
    #[test]
    fn prop_dimensional_price_quantity_money(
        price_units in 1u64..100_000,
        price_paise in 0u64..100,
        qty_units in 1u64..50_000,
    ) {
        let price_dec = Decimal::from(price_units) + (Decimal::from(price_paise) / Decimal::from(100));
        let price = Price::new(price_dec);
        let qty = Quantity::from_u64(qty_units);

        let notional = Money::notional(price, qty);
        prop_assert_eq!(notional.amount, price.value() * qty.value());
        prop_assert_eq!(notional.currency, Currency::Inr);
    }

    // 2. Position sizing bounds: risk_at_stake <= equity * risk_pct and quantity <= max_position_qty
    #[test]
    fn prop_position_sizing_bounds(
        equity_rupees in 10_000u64..10_000_000,
        risk_pct_bps in 10u64..500, // 0.1% to 5.0%
        entry_rupees in 10u64..20_000,
        stop_dist_pct in 1u64..20, // 1% to 20% stop distance
        lot_size in 1u64..100,
        max_qty_lots in 1u64..200,
    ) {
        let equity = Money::inr(Decimal::from(equity_rupees));
        let risk_pct = Decimal::from(risk_pct_bps) / Decimal::from(10_000);
        let min_lot = Quantity::from_u64(lot_size);
        let max_qty = Quantity::from_u64(lot_size * max_qty_lots);

        let budget = RiskBudget::new(equity, risk_pct, min_lot)
            .with_max_qty(max_qty);

        let entry = Price::new(Decimal::from(entry_rupees));
        let stop_distance = Decimal::from(entry_rupees) * (Decimal::from(stop_dist_pct) / Decimal::from(100));
        let stop_loss_val = (Decimal::from(entry_rupees) - stop_distance).max(Decimal::from_str("0.05").unwrap());
        let stop_loss = Price::new(stop_loss_val);

        let instrument = InstrumentId::new(Exchange::Nse, "PROPSYM");
        let req = PositionSizingRequest::new(
            instrument,
            PositionSide::Long,
            entry,
            equity,
            budget,
            SizingMethod::FixedFractional { risk_pct },
        ).with_stop_loss(stop_loss);

        let res = calculate_position_size(&req).unwrap();

        // Check quantity is non-negative and lot-aligned
        prop_assert!(res.quantity.value() >= Decimal::ZERO);
        prop_assert!(res.quantity.is_lot_multiple(lot_size));

        // Invariant 1: quantity <= max_position_qty
        prop_assert!(res.quantity.value() <= max_qty.value());

        // Invariant 2: risk_at_stake <= equity * risk_pct (exact or strictly under due to lot-rounding)
        let max_allowed_risk = equity.amount * risk_pct;
        prop_assert!(res.risk_at_stake.amount <= max_allowed_risk);
    }

    // 3. TraceContext wire invariance & W3C traceparent roundtrip
    #[test]
    fn prop_trace_context_wire_invariance(
        trace_id in proptest::array::uniform16(any::<u8>()),
        span_id in proptest::array::uniform8(any::<u8>()),
        flags in any::<u8>(),
    ) {
        // Trace ID and Span ID cannot be all zeros per W3C specification
        let mut t = trace_id;
        if t == [0u8; 16] {
            t[0] = 1;
        }
        let mut s = span_id;
        if s == [0u8; 8] {
            s[0] = 1;
        }

        let ctx = TraceContext::new(t, s, flags);
        let header = ctx.to_traceparent();
        let parsed = TraceContext::from_traceparent(&header).unwrap();
        prop_assert_eq!(ctx, parsed);

        // JSON Serde invariance
        let json = serde_json::to_string(&ctx).unwrap();
        let from_json: TraceContext = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(ctx, from_json);
    }

    // 4. DomainEvent serialization roundtrips
    #[test]
    fn prop_domain_event_serde_roundtrip(
        qty in 1u64..1000,
        price in 1u64..50000,
        trace_id in proptest::array::uniform16(any::<u8>()),
        span_id in proptest::array::uniform8(any::<u8>()),
    ) {
        let mut t = trace_id;
        if t == [0u8; 16] { t[0] = 1; }
        let mut s = span_id;
        if s == [0u8; 8] { s[0] = 1; }
        let trace_ctx = TraceContext::new(t, s, 1);

        let event = DomainEvent::OrderPlaced {
            order_id: "O-PROP".to_string(),
            client_order_id: Uuid::new_v4(),
            instrument: InstrumentId::new(Exchange::Nse, "TCS"),
            algo_id: "test-algo".to_string(),
            side: OrderSide::Buy,
            quantity: Quantity::from_u64(qty),
            price: Some(Price::new(Decimal::from(price))),
            timestamp: Utc::now(),
            trace_context: Some(trace_ctx),
        };

        let json = serde_json::to_string(&event).unwrap();
        let decoded: DomainEvent = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(event, decoded);
    }
}
