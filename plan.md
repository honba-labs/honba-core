# Master Plan: 本場・核 honba-core

> **The shared kernel of the entire platform.**  
> Houses canonical value objects, domain contracts, closed decision grammars, and wire primitives. Strict zero-I/O purity, zero external runtime dependencies, 100% exact 128-bit decimal arithmetic (`rust_decimal::Decimal`), and `schemars` type derivations driving platform-wide contract codegen.

---

## 0. Executive Summary & Design Scope

`honba-core` ("Kernel") is the foundational Shared Kernel in the Domain-Driven Design (DDD) topology. It contains no stateful aggregates, no network sockets, no clock calls, and no file I/O. Every other subsystem depends on `honba-core`:

```text
               ┌────────────────────────────────────────────────────────┐
               │              honba-core (Shared Kernel)                │
               │  - Value Objects: Price, Quantity, Money, Currency     │
               │  - Identifiers: InstrumentId, AccountId                │
               │  - Market Contracts: Tick, Bar, QuoteDepth, PriceBand  │
               │  - Wire Contracts: OrderRequest, OrderAck, FillReport  │
               │  - AI Decision Grammar: Decision enum                  │
               └───────────────────────────┬────────────────────────────┘
                                           │
         ┌──────────────────┬──────────────┼──────────────┬──────────────────┐
         ▼                  ▼              ▼              ▼                  ▼
    honba-kokoro        honba-mon       honba-me       honba-te          honba-ya
    (Domain Heart)     (Risk Gate)     (Market Eye)  (Execution Hand)  (Central House)
```

---

## 1. Architectural Invariants

1. **Zero External Dependencies**: Beyond `rust_decimal`, `serde`, `schemars`, and standard library utilities, `honba-core` contains zero dependencies. No Tokio, no HTTP clients, no database drivers.
2. **Zero Floats Policy**: All prices, quantities, cash balances, and financial metrics are represented as `rust_decimal::Decimal`. Generic floats (`f32`/`f64`) are strictly forbidden.
3. **Immutability & Value Semantics**: Primitive types implement `Copy` or lightweight `Clone`, `Eq`, `Hash`, and `Ord` where applicable.
4. **Single Source of Truth for Contracts**: All types derive `JsonSchema` via `schemars`, driving automated codegen of OpenAPI 3.1 specs, JSON schemas, MCP tool definitions, and Python type stubs.

---

## 2. Phase-by-Phase Implementation Roadmap

### Phase 1: Canonical Value Objects & Primitives (M0) — Week 1–2
- [x] **1.1 Dimensional Primitives** (COMPLETED)
  - [x] `Price(Decimal)`: Exact price representation with strict positive invariants.
  - [x] `Quantity(Decimal)`: Share or contract count with non-negative invariants.
  - [x] `Money { amount: Decimal, currency: Currency }`: Currency-aware financial amounts.
  - [x] `Currency`: Minor-unit exponent definition (e.g. INR minor exponent = 2).
- [x] **1.2 Identifiers & Market Structures** (COMPLETED)
  - [x] `InstrumentId { symbol, exchange, asset_class }`.
  - [x] `Exchange` enum: `NSE`, `BSE`, `NFO`, `MCX`.
  - [x] `AssetClass` enum: `Equity`, `Future`, `Option`, `MutualFund`.
  - [x] `ProductType` enum: `CncDelivery`, `MisIntraday`, `NrmlDerivative`, `MtfMargin`.

### Phase 2: Market Data & Wire Contracts (M1) — Week 3–4
- [x] **2.1 Canonical Market Data Contracts** (COMPLETED)
  - [x] `Tick { symbol, timestamp, ltp, volume, bid, ask, band }`.
  - [x] `Bar { symbol, start_time, open, high, low, close, volume }`.
  - [x] `PriceBand { lower, upper }`: Scrip-wise operational circuit filters.
- [ ] **2.2 Execution & Order Wire Contracts**
  - [ ] `OrderRequest`: Outbound order intent (symbol, side, order_type, qty, price, product, algo_id).
  - [ ] `OrderAck`: Broker confirmation with client order ID (`clOrdId`) and exchange order ID.
  - [ ] `FillReport`: Execution fill details (qty, price, timestamp, execution_id, fees).
  - [ ] `CancelRequest` & `ModifyRequest`.

### Phase 3: AI-Native Closed Decision Grammar (M2) — Week 5–6
- [ ] **3.1 `Decision` Enum Implementation**
  - [ ] Implement `Decision` with `#[serde(deny_unknown_fields)]`:
    - `PlaceOrder`, `ModifyOrder`, `CancelOrder`, `CancelAll`, `Hold`.
  - [ ] Integrate mandatory SEBI `algo_id` validation in `PlaceOrder`.
- [ ] **3.2 Serialization Invariants & Fuzzing**
  - [ ] Property test serde serialization/deserialization across random vectors.
  - [ ] Assert that malformed JSON or unknown keys fail deserialization immediately.

### Phase 4: Schema Export & Automated Codegen Pipeline (M3) — Week 7–8
- [ ] **4.1 `schemars` Derivations**
  - [ ] Ensure all domain types implement `JsonSchema`.
  - [ ] Verify clean JSON Schema export for `schema/domain/domain_schema.json`.
- [ ] **4.2 Golden Vector Alignment**
  - [ ] Ensure golden conformance vectors in `schema/golden/` deserialize without discrepancies.

---

## 3. Verification & CI Test Suite

- [ ] `cargo test -p honba-core`: Full unit test coverage for dimensional arithmetic and serialization.
- [ ] `proptest`: Property tests verifying dimensional invariants ($\text{Price} \times \text{Quantity} = \text{Money}$).
- [ ] AST lint checks ensuring zero usage of `f32`/`f64` throughout `honba-core/src/`.
