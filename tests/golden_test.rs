use honba_core::*;
use serde_json;
use std::fs;

#[test]
fn test_golden_vector_tick_ltp() {
    let json = fs::read_to_string("schema/golden/tick_ltp_golden.json").unwrap();
    let tick: Tick = serde_json::from_str(&json).unwrap();
    match tick {
        Tick::Ltp { instrument, price, volume, .. } => {
            assert_eq!(instrument.symbol, "RELIANCE");
            assert_eq!(instrument.exchange, Exchange::Nse);
            assert!(price.is_positive());
            assert!(volume.is_positive());
        }
        _ => panic!("Expected Ltp variant"),
    }
}

#[test]
fn test_golden_vector_tick_quote() {
    let json = fs::read_to_string("schema/golden/tick_quote_golden.json").unwrap();
    let tick: Tick = serde_json::from_str(&json).unwrap();
    match tick {
        Tick::Quote { instrument, bid, ask, bid_qty, ask_qty, .. } => {
            assert_eq!(instrument.symbol, "TCS");
            assert!(bid.is_positive());
            assert!(ask.is_positive());
            assert!(bid_qty.is_positive());
            assert!(ask_qty.is_positive());
        }
        _ => panic!("Expected Quote variant"),
    }
}

#[test]
fn test_golden_vector_price_band() {
    let json = fs::read_to_string("schema/golden/price_band_golden.json").unwrap();
    let band: PriceBand = serde_json::from_str(&json).unwrap();
    assert!(band.contains(Price::from_rupees("1000.00").unwrap()));
    assert!(!band.contains(Price::from_rupees("899.95").unwrap()));
    assert!(!band.contains(Price::from_rupees("1100.05").unwrap()));
}

#[test]
fn test_golden_vector_decision_placeorder() {
    let json = fs::read_to_string("schema/golden/decision_placeorder_golden.json").unwrap();
    let decision: Decision = serde_json::from_str(&json).unwrap();
    match decision {
        Decision::PlaceOrder { symbol, side, qty, price_limit, algo_id, .. } => {
            assert_eq!(symbol, "RELIANCE");
            assert_eq!(side, OrderSide::Buy);
            assert!(qty.is_positive());
            assert!(algo_id != "");
            assert!(price_limit.is_some_and(|p| p.is_positive()));
        }
        _ => panic!("Expected PlaceOrder variant"),
    }
}

#[test]
fn test_golden_vector_decision_hold() {
    let json = fs::read_to_string("schema/golden/decision_hold_golden.json").unwrap();
    let decision: Decision = serde_json::from_str(&json).unwrap();
    match decision {
        Decision::Hold { reason } => {
            assert!(reason.is_some());
        }
        _ => panic!("Expected Hold variant"),
    }
}

#[test]
fn test_golden_vector_decision_cancelall() {
    let json = fs::read_to_string("schema/golden/decision_cancelall_golden.json").unwrap();
    let decision: Decision = serde_json::from_str(&json).unwrap();
    match decision {
        Decision::CancelAll { algo_id, .. } => {
            assert!(algo_id.is_some());
        }
        _ => panic!("Expected CancelAll variant"),
    }
}
