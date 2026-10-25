//! CI contract regression test asserting backward compatibility of generated schemas per SemVer guidelines.

use honba_core::*;
use schemars::schema_for;
use serde_json::Value;
use std::fs;

#[test]
fn test_all_canonical_types_implement_valid_json_schema() {
    let _s_trace = schema_for!(TraceContext);
    let _s_method = schema_for!(SizingMethod);
    let _s_budget = schema_for!(RiskBudget);
    let _s_req = schema_for!(PositionSizingRequest);
    let _s_res = schema_for!(PositionSizingResult);
    let _s_event = schema_for!(DomainEvent);
    let _s_signal = schema_for!(Signal);
    let _s_order_req = schema_for!(OrderRequest);
    let _s_order_ack = schema_for!(OrderAck);
    let _s_fill = schema_for!(FillReport);
    let _s_decision = schema_for!(Decision);
    let _s_price = schema_for!(Price);
    let _s_qty = schema_for!(Quantity);
    let _s_money = schema_for!(Money);
    let _s_country = schema_for!(CountryCode);
    let _s_session = schema_for!(SessionWindow);
    let _s_instrument = schema_for!(Instrument);
}

#[test]
fn test_schema_backward_compatibility_v7_regression() {
    let v7_str = fs::read_to_string("schema/compatibility/v7_domain_schema.json")
        .expect("schema/compatibility/v7_domain_schema.json must exist");
    let v7_json: Value = serde_json::from_str(&v7_str).expect("v7 schema must be valid JSON");

    let v7_properties = v7_json
        .get("properties")
        .and_then(Value::as_object)
        .expect("properties map in v7 schema");

    let v7_required = v7_json
        .get("required")
        .and_then(Value::as_array)
        .expect("required list in v7 schema");

    // Load current domain schema
    let current_str = fs::read_to_string("schema/domain/domain_schema.json")
        .expect("schema/domain/domain_schema.json must exist");
    let current_json: Value =
        serde_json::from_str(&current_str).expect("current domain schema must be valid JSON");

    let current_properties = current_json
        .get("properties")
        .and_then(Value::as_object)
        .expect("properties map in current schema");

    let current_required: Vec<&str> = current_json
        .get("required")
        .and_then(Value::as_array)
        .expect("required list in current schema")
        .iter()
        .filter_map(Value::as_str)
        .collect();

    // 1. Assert no v7 required type is removed
    for item in v7_required {
        let name = item.as_str().expect("type name string");
        assert!(
            current_required.contains(&name),
            "SemVer regression: required type '{name}' in v7 was removed from current required list"
        );
        assert!(
            current_properties.contains_key(name),
            "SemVer regression: property definition '{name}' in v7 was removed from current properties"
        );
    }

    // 2. Assert all properties inside existing objects retain their required fields
    for (type_name, v7_type_def) in v7_properties {
        let curr_type_def = current_properties.get(type_name).unwrap_or_else(|| {
            panic!("missing type definition for '{type_name}' in current schema")
        });

        if let Some(v7_fields) = v7_type_def.get("properties").and_then(Value::as_object) {
            let curr_fields = curr_type_def
                .get("properties")
                .and_then(Value::as_object)
                .unwrap_or_else(|| {
                    panic!("expected properties on '{type_name}' in current schema")
                });

            for (field_name, _) in v7_fields {
                assert!(
                    curr_fields.contains_key(field_name),
                    "SemVer regression: field '{field_name}' in '{type_name}' was removed in current schema"
                );
            }
        }
    }
}
