use chrono::{Datelike, Utc};
use honba_core::*;
use rust_decimal::Decimal;
use std::str::FromStr;
use std::sync::Arc;
use uuid::Uuid;

fn instrument() -> InstrumentId {
    InstrumentId::new(Exchange::Nse, "RELIANCE")
}

#[test]
fn test_exact_price_math() {
    let p1 = Price::from_rupees("2450.75").unwrap();
    let p2 = Price::from_rupees("0.05").unwrap();
    assert_eq!(
        Price::new(p1.0 + p2.0),
        Price::from_rupees("2450.80").unwrap()
    );

    // Tick alignment
    let tick = Decimal::from_str("0.05").unwrap();
    assert!(p1.is_tick_aligned(tick));

    let unaligned = Price::from_rupees("2450.73").unwrap();
    assert!(!unaligned.is_tick_aligned(tick));
}

#[test]
fn test_price_band_conformance() {
    let lower = Price::from_rupees("900.00").unwrap();
    let upper = Price::from_rupees("1100.00").unwrap();
    let ref_p = Price::from_rupees("1000.00").unwrap();

    let band = PriceBand::new(lower, upper, ref_p);
    assert!(band.contains(Price::from_rupees("900.00").unwrap()));
    assert!(band.contains(Price::from_rupees("1000.00").unwrap()));
    assert!(band.contains(Price::from_rupees("1100.00").unwrap()));
    assert!(!band.contains(Price::from_rupees("899.95").unwrap()));
    assert!(!band.contains(Price::from_rupees("1100.05").unwrap()));
}

#[test]
fn test_decision_serde_and_strict_validation() {
    let valid_json = r#"{
        "action": "place_order",
        "symbol": "TCS",
        "side": "BUY",
        "qty": "10",
        "algo_id": "momentum-alpha-v1",
        "rationale": "breakout above 20-day high"
    }"#;

    let decision = Decision::from_json_str(valid_json).expect("should parse successfully");
    assert!(matches!(decision, Decision::PlaceOrder { .. }));

    // Reject unknown fields (LLM hallucination safety)
    let bad_field_json = r#"{
        "action": "place_order",
        "symbol": "TCS",
        "qty": "10",
        "algo_id": "algo-1",
        "hallucinated_param": 42
    }"#;
    assert!(Decision::from_json_str(bad_field_json).is_err());

    // Reject negative quantity
    let negative_qty_json = r#"{
        "action": "place_order",
        "symbol": "TCS",
        "qty": "-5",
        "algo_id": "algo-1"
    }"#;
    assert!(Decision::from_json_str(negative_qty_json).is_err());

    // Test ModifyOrder
    let modify_json = r#"{
        "action": "modify_order",
        "instrument": {
            "exchange": "NSE",
            "symbol": "RELIANCE"
        },
        "new_qty": "20",
        "algo_id": "modify-algo-1",
        "rationale": "adjust quantity"
    }"#;
    let modify_decision = Decision::from_json_str(modify_json).expect("should parse modify");
    assert!(matches!(modify_decision, Decision::ModifyOrder { .. }));

    // Test CancelOrder
    let cancel_json = r#"{
        "action": "cancel_order",
        "instrument": {
            "exchange": "NSE",
            "symbol": "RELIANCE"
        },
        "algo_id": "cancel-algo-1",
        "rationale": "order filled"
    }"#;
    let cancel_decision = Decision::from_json_str(cancel_json).expect("should parse cancel");
    assert!(matches!(cancel_decision, Decision::CancelOrder { .. }));

    // Test CancelAll
    let cancel_all_json = r#"{
        "action": "cancel_all",
        "algo_id": "all-algo-1"
    }"#;
    let cancel_all_decision =
        Decision::from_json_str(cancel_all_json).expect("should parse cancel all");
    assert!(matches!(cancel_all_decision, Decision::CancelAll { .. }));

    // Test Hold
    let hold_json = r#"{
        "action": "hold",
        "reason": "waiting for better price"
    }"#;
    let hold_decision = Decision::from_json_str(hold_json).expect("should parse hold");
    assert!(matches!(hold_decision, Decision::Hold { .. }));

    // Reject unknown fields for all variants
    let bad_json = r#"{
        "action": "place_order",
        "symbol": "TCS",
        "side": "BUY",
        "qty": "10",
        "algo_id": "algo-1",
        "unknown_field": true
    }"#;
    assert!(Decision::from_json_str(bad_json).is_err());
}

#[test]
fn test_price_serde_roundtrip() {
    // Test Price roundtrip
    let price = Price::from_rupees("2450.75").unwrap();
    let json = serde_json::to_string(&price).unwrap();
    let decoded: Price = serde_json::from_str(&json).unwrap();
    assert_eq!(price, decoded);

    // Test Quantity roundtrip
    let qty = Quantity::from_u64(100);
    let json = serde_json::to_string(&qty).unwrap();
    let decoded: Quantity = serde_json::from_str(&json).unwrap();
    assert_eq!(qty, decoded);

    // Test Money roundtrip
    let money = Money::inr(Decimal::from(123456789u64));
    let json = serde_json::to_string(&money).unwrap();
    let decoded: Money = serde_json::from_str(&json).unwrap();
    assert_eq!(money, decoded);
}

#[test]
fn test_priceband_serde_roundtrip() {
    let lower = Price::from_rupees("900.00").unwrap();
    let upper = Price::from_rupees("1100.00").unwrap();
    let ref_p = Price::from_rupees("1000.00").unwrap();
    let band = PriceBand::new(lower, upper, ref_p);
    let json = serde_json::to_string(&band).unwrap();
    let decoded: PriceBand = serde_json::from_str(&json).unwrap();
    assert_eq!(band, decoded);
    assert!(band.contains(Price::from_rupees("1000.00").unwrap()));
    assert!(!band.contains(Price::from_rupees("899.95").unwrap()));
}

#[test]
fn test_decision_validation_invariants() {
    // Empty algo_id should be rejected
    let json = r#"{
        "action": "place_order",
        "symbol": "TCS",
        "side": "BUY",
        "qty": "10",
        "algo_id": "",
        "rationale": ""
    }"#;
    assert!(Decision::from_json_str(json).is_err());

    // Unknown fields should be rejected (deny_unknown_fields)
    let json = r#"{
        "action": "place_order",
        "symbol": "TCS",
        "side": "BUY",
        "qty": "10",
        "algo_id": "algo-1",
        "unknown_field": true
    }"#;
    assert!(Decision::from_json_str(json).is_err());

    // Negative quantity should be rejected
    let json = r#"{
        "action": "place_order",
        "symbol": "TCS",
        "side": "BUY",
        "qty": "-5",
        "algo_id": "algo-1"
    }"#;
    assert!(Decision::from_json_str(json).is_err());

    // Empty symbol should be rejected
    let json = r#"{
        "action": "place_order",
        "symbol": "",
        "side": "BUY",
        "qty": "10",
        "algo_id": "algo-1"
    }"#;
    assert!(Decision::from_json_str(json).is_err());

    // CancelAll with empty algo_id should be rejected
    let json = r#"{
        "action": "cancel_all",
        "algo_id": ""
    }"#;
    assert!(Decision::from_json_str(json).is_err());

    // Hold should always be valid
    let json = r#"{"action": "hold"}"#;
    assert!(Decision::from_json_str(json).is_ok());
}

#[test]
fn test_currency_minor_unit_table() {
    // The kernel owns the table: every consumer reads it from here.
    assert_eq!(Currency::Inr.code(), "INR");
    assert_eq!(Currency::Inr.minor_exponent(), 2);
    assert_eq!(Currency::Inr.minor_unit().singular, "paisa");
    assert_eq!(Currency::Inr.minor_unit().plural, "paise");
    assert_eq!(Currency::Gbp.minor_unit().plural, "pence");

    // Display is the ISO code, whatever the serde name is.
    assert_eq!(Currency::Usd.to_string(), "USD");

    // Codes parse case-insensitively; an unknown code is refused.
    assert_eq!(Currency::from_code("inr").unwrap(), Currency::Inr);
    assert_eq!(Currency::from_code("GBP").unwrap(), Currency::Gbp);
    assert!(Currency::from_code("JPY").is_err());

    // Every currency in the table has a code and a minor unit.
    for currency in Currency::ALL {
        assert!(!currency.minor_unit().singular.is_empty());
        assert_eq!(Currency::from_code(currency.code()).unwrap(), *currency);
    }
}

#[test]
fn test_money_carries_currency() {
    let money = Money::new(Decimal::from_str("1234.56").unwrap(), Currency::Inr);
    assert_eq!(money.currency, Currency::Inr);
    assert_eq!(money.amount, Decimal::from_str("1234.56").unwrap());

    let notional = Money::notional(
        Price::from_rupees("2450.75").unwrap(),
        Quantity::from_u64(10),
    );
    assert_eq!(notional.amount, Decimal::from_str("24507.50").unwrap());
    assert_eq!(notional.currency, Currency::Inr);
}

#[test]
fn test_position_side_and_instrument() {
    assert_eq!(PositionSide::Long.opposite(), PositionSide::Short);
    assert_eq!(PositionSide::Short.opposite(), PositionSide::Long);

    let eq = Instrument::equity(
        instrument(),
        Quantity::from_u64(1),
        Decimal::from_str("0.05").unwrap(),
    );
    assert_eq!(eq.kind, InstrumentKind::Equity);
    assert!(eq.expiry.is_none());
    assert!(eq.option_type.is_none());

    let json = serde_json::to_string(&eq).unwrap();
    let decoded: Instrument = serde_json::from_str(&json).unwrap();
    assert_eq!(eq, decoded);
}

#[test]
fn test_typed_ids_display_and_serde() {
    let dataset = DatasetId::new("nse-eq-2024");
    assert_eq!(dataset.to_string(), "nse-eq-2024");
    let json = serde_json::to_string(&dataset).unwrap();
    assert_eq!(serde_json::from_str::<DatasetId>(&json).unwrap(), dataset);

    let trade = TradeId::new("T-1");
    let venue_order = VenueOrderId::new("VENUE-1");
    assert_eq!(trade.to_string(), "T-1");
    assert_eq!(venue_order.to_string(), "VENUE-1");
}

#[test]
fn test_order_event_kind_and_reject_reason() {
    let ack = OrderAck::new("O-1", Uuid::new_v4(), OrderStatus::New, Utc::now());
    let acked = OrderEvent::Acked(ack);
    assert_eq!(acked.kind(), OrderEventKind::Acked);

    let rejected = OrderEvent::Rejected {
        order_id: "O-1".to_string(),
        reason: RejectReason::InsufficientMargin,
    };
    assert_eq!(rejected.kind(), OrderEventKind::Rejected);

    let json = serde_json::to_string(&rejected).unwrap();
    let decoded: OrderEvent = serde_json::from_str(&json).unwrap();
    assert_eq!(rejected, decoded);
}

#[test]
fn test_portfolio_snapshot_roundtrip() {
    let snapshot = PortfolioSnapshot {
        account: PortfolioAccount {
            account_id: "ACC-1".to_string(),
            mode: PortfolioMode::Paper,
            source: PortfolioSource::Simulated,
        },
        positions: vec![PortfolioPosition {
            instrument: instrument(),
            side: PositionSide::Long,
            quantity: Quantity::from_u64(10),
            avg_price: Price::from_rupees("2450.75").unwrap(),
            unrealized_pnl: Money::inr(Decimal::from(500)),
            realized_pnl: Money::inr(Decimal::ZERO),
        }],
        holdings: vec![],
        totals: PortfolioTotals {
            cash: Money::inr(Decimal::from(100_000)),
            margin_used: Money::inr(Decimal::from(20_000)),
            margin_available: Money::inr(Decimal::from(80_000)),
            total_pnl: Money::inr(Decimal::from(500)),
            net_worth: Money::inr(Decimal::from(124_507)),
        },
        greeks: None,
        timestamp: Utc::now(),
    };

    let json = serde_json::to_string(&snapshot).unwrap();
    let decoded: PortfolioSnapshot = serde_json::from_str(&json).unwrap();
    assert_eq!(snapshot, decoded);
    assert_eq!(snapshot.positions[0].side, PositionSide::Long);
}

#[test]
fn test_signal_scan_roundtrip() {
    let signal = Signal::new(
        instrument(),
        SignalDirection::Bullish,
        SignalHorizon::Swing,
        Timeframe::D1,
        Decimal::from_str("0.75").unwrap(),
        "momentum-alpha-v1",
        Utc::now(),
    );
    let response = SignalScanResponse {
        signals: vec![signal.clone()],
    };
    let json = serde_json::to_string(&response).unwrap();
    let decoded: SignalScanResponse = serde_json::from_str(&json).unwrap();
    assert_eq!(decoded.signals[0], signal);
}

#[test]
fn test_signal_enrichment_and_builder() {
    let ep = Price::from_rupees("2500.00").unwrap();
    let sl = Price::from_rupees("2400.00").unwrap();
    let tp = Price::from_rupees("2700.00").unwrap();
    let atr = Decimal::from_str("35.50").unwrap();

    let signal = Signal::new(
        instrument(),
        SignalDirection::Bullish,
        SignalHorizon::Swing,
        Timeframe::D1,
        Decimal::from_str("0.85").unwrap(),
        "breakout-v2",
        Utc::now(),
    )
    .with_entry_price(ep)
    .with_stop_loss(sl)
    .with_take_profit(tp)
    .with_volatility_atr(atr);

    assert_eq!(signal.entry_price, Some(ep));
    assert_eq!(signal.stop_loss, Some(sl));
    assert_eq!(signal.take_profit, Some(tp));
    assert_eq!(signal.volatility_atr, Some(atr));

    let json = serde_json::to_string(&signal).unwrap();
    let decoded: Signal = serde_json::from_str(&json).unwrap();
    assert_eq!(signal, decoded);
}

#[test]
fn test_screener_scan_request_roundtrip() {
    let predicate = ScreenerFilterPredicate {
        metric: MetricRef(MetricKeySpec {
            metric: "rsi_14".to_string(),
            period: MetricPeriod::D1,
        }),
        op: FilterOp::Gt,
        value: Decimal::from(70),
        value_upper: None,
    };
    let request = ScreenerScanRequest {
        universe: Some(vec![instrument()]),
        filters: ScreenerFilterGroup {
            predicates: vec![predicate],
            match_all: true,
        },
        sort: None,
        limit: Some(50),
    };
    let json = serde_json::to_string(&request).unwrap();
    let decoded: ScreenerScanRequest = serde_json::from_str(&json).unwrap();
    assert_eq!(decoded, request);

    let row = ScreenerRow {
        instrument: instrument(),
        metrics: std::collections::BTreeMap::from([("rsi_14".to_string(), Decimal::from(72))]),
    };
    let response = ScreenerScanResponse {
        rows: vec![row],
        total_matched: 1,
    };
    assert_eq!(response.rows.len(), 1);
    assert_eq!(response.total_matched, 1);
}

#[test]
fn test_execution_event_and_trading_state() {
    let refused = ExecutionEvent::RiskRefused {
        order_id: "O-1".to_string(),
        reasons: vec![Reason {
            group: ReasonGroup::Risk,
            polarity: ReasonPolarity::Negative,
            code: "otr_breached".to_string(),
            message: "order-to-trade ratio exceeded".to_string(),
        }],
    };
    let json = serde_json::to_string(&refused).unwrap();
    let decoded: ExecutionEvent = serde_json::from_str(&json).unwrap();
    assert_eq!(refused, decoded);

    let halted = TradingState::Halted {
        reason: "circuit breach".to_string(),
    };
    assert_ne!(halted, TradingState::Active);
}

#[test]
fn test_port_error_display_and_connector_state() {
    assert_eq!(
        PortError::NotFound("RELIANCE".to_string()).to_string(),
        "not found: RELIANCE"
    );
    assert_eq!(PortError::Timeout.to_string(), "operation timed out");

    let degraded = ConnectorState::Degraded {
        reason: "stale heartbeat".to_string(),
    };
    let json = serde_json::to_string(&degraded).unwrap();
    assert_eq!(
        serde_json::from_str::<ConnectorState>(&json).unwrap(),
        degraded
    );
}

/// A minimal `QuoteReader` implementation, to prove the port traits are
/// object-safe (usable as `Arc<dyn Trait>`) as AGENTS.md §2.1 requires.
struct FixedQuote(Tick);

impl QuoteReader for FixedQuote {
    fn latest_quote(&self, _instrument: &InstrumentId) -> PortResult<Tick> {
        Ok(self.0.clone())
    }
}

#[test]
fn test_port_trait_is_object_safe() {
    let tick = Tick::Ltp {
        instrument: instrument(),
        price: Price::from_rupees("2450.75").unwrap(),
        volume: Quantity::from_u64(100),
        exchange_ts: Utc::now(),
        receive_ts: Utc::now(),
    };
    let reader: std::sync::Arc<dyn QuoteReader> = std::sync::Arc::new(FixedQuote(tick.clone()));
    assert_eq!(reader.latest_quote(&instrument()).unwrap(), tick);
}

#[test]
fn test_fill_report_fee_defaults_to_zero() {
    let fill = FillReport::new(
        "F-1",
        "O-1",
        Uuid::new_v4(),
        instrument(),
        OrderSide::Buy,
        Price::from_rupees("2450.75").unwrap(),
        Quantity::from_u64(10),
        "momentum-alpha-v1",
        Utc::now(),
    );
    assert_eq!(fill.fee, Money::zero_inr());
    assert!(fill.fee_breakdown.is_none());
}

#[test]
fn test_fee_breakdown_total_and_with_fee_breakdown() {
    let breakdown = FeeBreakdown {
        brokerage: Money::inr(Decimal::from_str("20.00").unwrap()),
        stt: Money::inr(Decimal::from_str("12.50").unwrap()),
        exchange_charges: Money::inr(Decimal::from_str("1.50").unwrap()),
        gst: Money::inr(Decimal::from_str("3.87").unwrap()),
        stamp_duty: Money::inr(Decimal::from_str("1.00").unwrap()),
        sebi_charges: Money::inr(Decimal::from_str("0.10").unwrap()),
    };
    assert_eq!(
        breakdown.total(),
        Money::inr(Decimal::from_str("38.97").unwrap())
    );

    let fill = FillReport::new(
        "F-2",
        "O-2",
        Uuid::new_v4(),
        instrument(),
        OrderSide::Sell,
        Price::from_rupees("1200.00").unwrap(),
        Quantity::from_u64(5),
        "momentum-alpha-v1",
        Utc::now(),
    )
    .with_fee_breakdown(breakdown);

    // `with_fee_breakdown` sets the total fee from the itemized breakdown,
    // so the two never drift apart.
    assert_eq!(fill.fee, breakdown.total());
    assert_eq!(fill.fee_breakdown, Some(breakdown));

    let flat = FillReport::new(
        "F-3",
        "O-3",
        Uuid::new_v4(),
        instrument(),
        OrderSide::Buy,
        Price::from_rupees("500.00").unwrap(),
        Quantity::from_u64(1),
        "momentum-alpha-v1",
        Utc::now(),
    )
    .with_fee(Money::inr(Decimal::from_str("5.00").unwrap()));
    assert_eq!(flat.fee, Money::inr(Decimal::from_str("5.00").unwrap()));
    assert!(flat.fee_breakdown.is_none());
}

#[test]
fn test_trace_context_w3c_roundtrip() {
    let trace_id = [
        0x4b, 0xf9, 0x2f, 0x35, 0x77, 0xb3, 0x4d, 0xa6, 0xa3, 0xce, 0x92, 0x9d, 0x0e, 0x0e, 0x47,
        0x36,
    ];
    let span_id = [0x00, 0xf0, 0x67, 0xaa, 0x0b, 0xa9, 0x02, 0xb7];
    let ctx = TraceContext::new(trace_id, span_id, TraceContext::SAMPLED);

    let header = ctx.to_traceparent();
    assert_eq!(
        header,
        "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01"
    );

    let parsed = TraceContext::from_traceparent(&header).unwrap();
    assert_eq!(ctx, parsed);
    assert!(parsed.is_sampled());

    let new_span = [0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08];
    let child = ctx.child_span(new_span);
    assert_eq!(child.trace_id, trace_id);
    assert_eq!(child.span_id, new_span);
    assert_eq!(child.trace_flags, ctx.trace_flags);

    // Serde roundtrip
    let json = serde_json::to_string(&ctx).unwrap();
    let decoded: TraceContext = serde_json::from_str(&json).unwrap();
    assert_eq!(ctx, decoded);
}

#[test]
fn test_order_wire_trace_context_roundtrip() {
    let trace_ctx = TraceContext::new([1u8; 16], [2u8; 8], 1);
    let order_req = OrderRequest::new(
        instrument(),
        OrderSide::Buy,
        OrderType::Market,
        Quantity::from_u64(10),
        "algo-1",
    )
    .with_trace_context(trace_ctx);

    assert_eq!(order_req.trace_context, Some(trace_ctx));
    let json = serde_json::to_string(&order_req).unwrap();
    let decoded: OrderRequest = serde_json::from_str(&json).unwrap();
    assert_eq!(decoded.trace_context, Some(trace_ctx));

    let ack = OrderAck::new(
        "O-1",
        order_req.client_order_id,
        OrderStatus::New,
        Utc::now(),
    )
    .with_trace_context(trace_ctx);
    let json_ack = serde_json::to_string(&ack).unwrap();
    let decoded_ack: OrderAck = serde_json::from_str(&json_ack).unwrap();
    assert_eq!(decoded_ack.trace_context, Some(trace_ctx));

    let fill = FillReport::new(
        "F-1",
        "O-1",
        order_req.client_order_id,
        instrument(),
        OrderSide::Buy,
        Price::from_rupees("2500.00").unwrap(),
        Quantity::from_u64(10),
        "algo-1",
        Utc::now(),
    )
    .with_trace_context(trace_ctx);
    let json_fill = serde_json::to_string(&fill).unwrap();
    let decoded_fill: FillReport = serde_json::from_str(&json_fill).unwrap();
    assert_eq!(decoded_fill.trace_context, Some(trace_ctx));
}

#[test]
fn test_position_sizing_fixed_fractional() {
    let equity = Money::inr(Decimal::from_str("100000.00").unwrap()); // 1,00,000 INR
    let risk_budget = RiskBudget::new(
        equity,
        Decimal::from_str("0.01").unwrap(),
        Quantity::from_u64(1),
    ); // 1% risk = 1,000 INR

    let entry = Price::from_rupees("2500.00").unwrap();
    let stop = Price::from_rupees("2450.00").unwrap(); // per share risk = 50 INR

    let req = PositionSizingRequest::new(
        instrument(),
        PositionSide::Long,
        entry,
        equity,
        risk_budget,
        SizingMethod::FixedFractional {
            risk_pct: Decimal::from_str("0.01").unwrap(),
        },
    )
    .with_stop_loss(stop);

    let res = calculate_position_size(&req).unwrap();
    // 1000 / 50 = 20 shares
    assert_eq!(res.quantity, Quantity::from_u64(20));
    assert_eq!(
        res.allocated_notional,
        Money::inr(Decimal::from_str("50000.00").unwrap())
    );
    assert_eq!(
        res.risk_at_stake,
        Money::inr(Decimal::from_str("1000.00").unwrap())
    );
    assert!(!res.capped_by_limit);
}

#[test]
fn test_position_sizing_capping_and_limits() {
    let equity = Money::inr(Decimal::from_str("1000000.00").unwrap());
    let risk_budget = RiskBudget::new(
        equity,
        Decimal::from_str("0.02").unwrap(),
        Quantity::from_u64(25),
    )
    .with_max_qty(Quantity::from_u64(100))
    .with_freeze_limit(Quantity::from_u64(150))
    .with_max_notional(Money::inr(Decimal::from_str("200000.00").unwrap()));

    let entry = Price::from_rupees("1000.00").unwrap();
    let stop = Price::from_rupees("950.00").unwrap(); // risk per share = 50 INR
    // Risk amount = 20,000 INR -> raw shares = 400.
    // Lot size is 25.
    // But max_qty cap is 100, which is smaller than 400.

    let req = PositionSizingRequest::new(
        instrument(),
        PositionSide::Long,
        entry,
        equity,
        risk_budget,
        SizingMethod::FixedFractional {
            risk_pct: Decimal::from_str("0.02").unwrap(),
        },
    )
    .with_stop_loss(stop);

    let res = calculate_position_size(&req).unwrap();
    assert_eq!(res.quantity, Quantity::from_u64(100));
    assert!(res.capped_by_limit);
    assert!(res.reason.unwrap().contains("capped by max_position_qty"));
}

#[test]
fn test_position_sizer_hexagonal_port_object_safe() {
    let equity = Money::inr(Decimal::from_str("50000.00").unwrap());
    let budget = RiskBudget::new(
        equity,
        Decimal::from_str("0.01").unwrap(),
        Quantity::from_u64(1),
    );
    let req = PositionSizingRequest::new(
        instrument(),
        PositionSide::Long,
        Price::from_rupees("100.00").unwrap(),
        equity,
        budget,
        SizingMethod::FixedNotional {
            notional: Money::inr(Decimal::from_str("1000.00").unwrap()),
        },
    );

    let sizer: Arc<dyn PositionSizer> = Arc::new(CanonicalPositionSizer);
    let res = sizer.calculate_size(&req).unwrap();
    assert_eq!(res.quantity, Quantity::from_u64(10));
}

#[test]
fn test_domain_event_serde_roundtrip() {
    let trace_ctx = TraceContext::new([3u8; 16], [4u8; 8], 1);
    let now = Utc::now();

    let event = DomainEvent::OrderPlaced {
        order_id: "O-100".to_string(),
        client_order_id: Uuid::new_v4(),
        instrument: instrument(),
        algo_id: "momentum-1".to_string(),
        side: OrderSide::Buy,
        quantity: Quantity::from_u64(50),
        price: Some(Price::from_rupees("2450.00").unwrap()),
        timestamp: now,
        trace_context: Some(trace_ctx),
    };

    assert_eq!(event.timestamp(), now);
    assert_eq!(event.trace_context(), Some(trace_ctx));

    let json = serde_json::to_string(&event).unwrap();
    let decoded: DomainEvent = serde_json::from_str(&json).unwrap();
    assert_eq!(event, decoded);

    let breached = DomainEvent::RiskBreached {
        order_id: Some("O-101".to_string()),
        instrument: instrument(),
        rule_id: "max_notional_per_order".to_string(),
        attempted_value: Decimal::from(500_000),
        threshold_limit: Decimal::from(200_000),
        message: "Order value 500000 exceeds limit 200000".to_string(),
        timestamp: now,
        trace_context: None,
    };
    let json_b = serde_json::to_string(&breached).unwrap();
    let decoded_b: DomainEvent = serde_json::from_str(&json_b).unwrap();
    assert_eq!(breached, decoded_b);
}

#[test]
fn test_country_code_display_parsing_and_serde() {
    assert_eq!(CountryCode::In.code(), "IN");
    assert_eq!(CountryCode::Us.code(), "US");
    assert_eq!(CountryCode::Uk.code(), "UK");
    assert_eq!(CountryCode::Jp.code(), "JP");

    assert_eq!(CountryCode::from_str("IN").unwrap(), CountryCode::In);
    assert_eq!(CountryCode::from_str("us").unwrap(), CountryCode::Us);
    assert_eq!(CountryCode::from_str("UK").unwrap(), CountryCode::Uk);
    assert_eq!(CountryCode::from_str("GB").unwrap(), CountryCode::Uk);
    assert_eq!(CountryCode::from_str("jp").unwrap(), CountryCode::Jp);
    assert!(CountryCode::from_str("XYZ").is_err());

    let json = serde_json::to_string(&CountryCode::In).unwrap();
    assert_eq!(json, "\"IN\"");
    let decoded: CountryCode = serde_json::from_str(&json).unwrap();
    assert_eq!(decoded, CountryCode::In);
}

#[test]
fn test_instrument_profile_with_currency_and_country() {
    let mut inst = Instrument::equity(
        instrument(),
        Quantity::from_u64(10),
        Decimal::from_str("0.05").unwrap(),
    );
    assert_eq!(inst.currency, Currency::Inr);
    assert_eq!(inst.country, CountryCode::In);

    inst = inst
        .with_currency(Currency::Usd)
        .with_country(CountryCode::Us);
    assert_eq!(inst.currency, Currency::Usd);
    assert_eq!(inst.country, CountryCode::Us);

    // Serialization roundtrip
    let json = serde_json::to_string(&inst).unwrap();
    let decoded: InstrumentProfile = serde_json::from_str(&json).unwrap();
    assert_eq!(inst, decoded);

    // Backward-compatibility: JSON without currency or country defaults to INR and IN
    let legacy_json = serde_json::json!({
        "id": {
            "exchange": "NSE",
            "symbol": "TCS"
        },
        "kind": "equity",
        "lot_size": "1",
        "tick_size": "0.05"
    });
    let parsed: Instrument = serde_json::from_value(legacy_json).unwrap();
    assert_eq!(parsed.currency, Currency::Inr);
    assert_eq!(parsed.country, CountryCode::In);
}

#[derive(Debug)]
struct MockTradingCalendar {
    offset: chrono::FixedOffset,
    holidays: Vec<chrono::NaiveDate>,
}

impl TradingCalendar for MockTradingCalendar {
    fn is_trading_day(&self, date: chrono::NaiveDate) -> bool {
        let weekday = date.weekday();
        if weekday == chrono::Weekday::Sat || weekday == chrono::Weekday::Sun {
            return false;
        }
        !self.is_holiday(date)
    }

    fn is_holiday(&self, date: chrono::NaiveDate) -> bool {
        self.holidays.contains(&date)
    }

    fn session_for_date(&self, date: chrono::NaiveDate) -> Option<SessionWindow> {
        if self.is_trading_day(date) {
            Some(SessionWindow::new(
                chrono::NaiveTime::from_hms_opt(9, 15, 0).unwrap(),
                chrono::NaiveTime::from_hms_opt(15, 30, 0).unwrap(),
            ))
        } else {
            None
        }
    }

    fn holidays(&self) -> Vec<chrono::NaiveDate> {
        self.holidays.clone()
    }

    fn timezone_offset(&self) -> chrono::FixedOffset {
        self.offset
    }
}

#[test]
fn test_trading_calendar_port_object_safe() {
    let holiday = chrono::NaiveDate::from_ymd_opt(2026, 1, 26).unwrap();
    let cal: Arc<dyn TradingCalendar> = Arc::new(MockTradingCalendar {
        offset: chrono::FixedOffset::east_opt(5 * 3600 + 30 * 60).unwrap(),
        holidays: vec![holiday],
    });

    let trading_day = chrono::NaiveDate::from_ymd_opt(2026, 1, 27).unwrap();
    assert!(cal.is_trading_day(trading_day));
    assert!(cal.is_holiday(holiday));
    assert!(!cal.is_trading_day(holiday));

    // Next trading day from Friday Jan 23 is Tuesday Jan 27 (since Jan 24/25 are weekend and Jan 26 is holiday)
    let fri = chrono::NaiveDate::from_ymd_opt(2026, 1, 23).unwrap();
    assert_eq!(cal.next_trading_day(fri), trading_day);

    // Market open check
    let open_utc = chrono::DateTime::parse_from_rfc3339("2026-01-27T04:00:00Z")
        .unwrap()
        .with_timezone(&Utc); // 04:00 UTC is 09:30 IST
    assert!(cal.is_market_open(open_utc));

    let closed_utc = chrono::DateTime::parse_from_rfc3339("2026-01-27T11:00:00Z")
        .unwrap()
        .with_timezone(&Utc); // 11:00 UTC is 16:30 IST
    assert!(!cal.is_market_open(closed_utc));
}
