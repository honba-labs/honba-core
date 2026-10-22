use chrono::Utc;
use honba_core::*;
use rust_decimal::Decimal;
use std::str::FromStr;
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
    let signal = Signal {
        instrument: instrument(),
        direction: SignalDirection::Bullish,
        horizon: SignalHorizon::Swing,
        timeframe: Timeframe::D1,
        strength: Decimal::from_str("0.75").unwrap(),
        generated_by: "momentum-alpha-v1".to_string(),
        timestamp: Utc::now(),
    };
    let response = SignalScanResponse {
        signals: vec![signal.clone()],
    };
    let json = serde_json::to_string(&response).unwrap();
    let decoded: SignalScanResponse = serde_json::from_str(&json).unwrap();
    assert_eq!(decoded.signals[0], signal);
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
