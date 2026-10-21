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
    let cancel_all_decision = Decision::from_json_str(cancel_all_json).expect("should parse cancel all");
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
