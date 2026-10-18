use chrono::Utc;
use honba_core::*;
use rust_decimal::Decimal;
use std::str::FromStr;

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
        "action": "buy",
        "symbol": "TCS",
        "exchange": "NSE",
        "qty": "10",
        "price_limit": "3850.50",
        "algo_id": "momentum-alpha-v1",
        "rationale": "breakout above 20-day high"
    }"#;

    let decision = Decision::from_json_str(valid_json).expect("should parse successfully");
    assert!(matches!(decision, Decision::Buy { .. }));

    // Reject unknown fields (LLM hallucination safety)
    let bad_field_json = r#"{
        "action": "buy",
        "symbol": "TCS",
        "qty": "10",
        "algo_id": "algo-1",
        "hallucinated_param": 42
    }"#;
    assert!(Decision::from_json_str(bad_field_json).is_err());

    // Reject negative quantity
    let negative_qty_json = r#"{
        "action": "buy",
        "symbol": "TCS",
        "qty": "-5",
        "algo_id": "algo-1"
    }"#;
    assert!(Decision::from_json_str(negative_qty_json).is_err());
}

#[test]
fn test_canonical_ticks() {
    let inst = InstrumentId::new(Exchange::Nse, "RELIANCE");
    let now = Utc::now();

    let ltp_tick = Tick::Ltp {
        instrument: inst.clone(),
        price: Price::from_rupees("2500.00").unwrap(),
        volume: Quantity::from_u64(100),
        exchange_ts: now,
        receive_ts: now,
    };

    assert_eq!(ltp_tick.instrument(), &inst);
    assert_eq!(
        ltp_tick.last_price(),
        Some(Price::from_rupees("2500.00").unwrap())
    );
}
