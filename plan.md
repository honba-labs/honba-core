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
- [x] **2.2 Execution & Order Wire Contracts**
  - [x] `OrderRequest`: Outbound order intent (symbol, side, order_type, qty, price, product, algo_id).
  - [x] `OrderAck`: Broker confirmation with client order ID (`clOrdId`) and exchange order ID.
  - [x] `FillReport`: Execution fill details (qty, price, timestamp, execution_id, fees).
  - [x] `CancelRequest`: Cancel request for existing order.
  - [x] `ModifyRequest`: Modify request for existing order.

### Phase 3: AI-Native Closed Decision Grammar (M2) — Week 5–6
- [x] **3.1 `Decision` Enum Implementation**
  - [x] Implement `Decision` with `#[serde(deny_unknown_fields)]`:
    - `PlaceOrder`, `ModifyOrder`, `CancelOrder`, `CancelAll`, `Hold`.
  - [x] Integrate mandatory SEBI `algo_id` validation in `PlaceOrder`.
- [x] **3.2 Serialization Invariants & Fuzzing**
  - [x] Property tests verifying serde serialization/deserialization across random vectors.
  - [x] Assert that malformed JSON or unknown keys fail deserialization immediately.

### Phase 4: Schema Export & Automated Codegen Pipeline (M3) — Week 7–8
- [x] **4.1 `schemars` Derivations**
  - [x] Ensure all domain types implement `JsonSchema`.
  - [x] Verify clean JSON Schema export for `schema/domain/domain_schema.json`.
- [x] **4.2 Golden Vector Alignment**
  - [x] Ensure golden conformance vectors in `schema/golden/` deserialize without discrepancies.

### Phase 5: Kokoro Legacy Symbol Absorption & Hexagonal Ports (M4)
`honba-kokoro` used to carry five placeholder crates (`honba-kokoro-{entities,messages,ports,types,risk}`) that several subsystems depended on for shared value objects, wire types, and port traits — an architectural misplacement, since Kokoro is the Heart (OMS/portfolio/ledger) bounded context, not the Shared Kernel. Those crates exported nothing and have now been deleted. Phase 5 gives their real symbols a home here instead.

- [x] **5.1 Multi-Currency & Validation**
  - [x] `Currency` extended to `Inr | Usd | Eur | Gbp`, with a `MinorUnit` table (`code`, `minor_exponent`, `minor_unit`, `from_code`/`FromStr`).
  - [x] `Money::new(amount, currency)` constructor.
  - [x] `Decision::ALGO_ID_FORMAT`: SEBI algo-id format enforced (4–32 chars, `[A-Za-z0-9_-]`) across `PlaceOrder`/`ModifyOrder`/`CancelOrder`/`CancelAll`.
- [x] **5.2 Instrument Master Data** (`src/instrument.rs`)
  - [x] `Instrument`, `InstrumentKind` (`Equity`/`Future`/`Option`/`Index`), `OptionType` (`Call`/`Put`).
  - [x] `PositionSide` (`Long`/`Short`) — distinct from `OrderSide`; the direction of a held/closed position, needed by the portfolio engine, screener, and analytics alike.
  - [x] Typed identifiers: `DatasetId`, `TradeId`, `VenueOrderId`.
- [x] **5.3 Order Lifecycle Events** (`src/order.rs`)
  - [x] `RejectReason`, `OrderEventKind`, `OrderEvent` — wraps the existing `OrderAck`/`FillReport` rather than re-deriving them.
- [x] **5.4 Portfolio Wire Contracts** (`src/portfolio.rs`)
  - [x] `PortfolioSnapshot` and its components: `PortfolioAccount`, `PortfolioMode`, `PortfolioSource`, `PortfolioPosition`, `PortfolioHolding`, `PortfolioGreeks`, `PortfolioTotals` — the Heart→House API→Face UI boundary contract.
- [x] **5.5 Signals & Screener** (`src/signal.rs`, `src/screener.rs`)
  - [x] `Signal`, `SignalDirection`, `SignalHorizon`, `Timeframe`, `SignalScanRequest`/`Response`.
  - [x] `ScreenerScanRequest`/`Response`, `ScreenerRow`, `ScreenerFilterGroup`/`Predicate`, `FilterOp`, `SortSpec`, `MetricDefinition`/`KeySpec`/`Ref`, `MetricPeriod` (alias of `Timeframe`).
- [x] **5.6 Cross-Subsystem Domain Events** (`src/events.rs`)
  - [x] `Reason`, `ReasonGroup`, `ReasonPolarity`, `ExecutionEvent`, `TradingState`.
- [x] **5.7 Hexagonal Port Traits** (`src/ports.rs`)
  - [x] `PortError`/`PortResult`, `ConnectorState`, `BarRequest`, `TickRequest`, `SubscriptionParams`, `DatedInstrument`, `MarketEvent`.
  - [x] Traits: `BarReader`, `TickReader`, `QuoteReader`, `DepthReader`, `MarketDataConnector`, `InstrumentMaster`, `ExecutionGateway` — synchronous signatures only (no `async`/Tokio dependency), object-safe for `Arc<dyn Trait>` injection per the Dependency Inversion principle.
  - [x] `SCHEMA_VERSION` bumped (wire contract surface changed).

**Deliberately not duplicated**: `Trade` (→ use `FillReport`), `Position` (→ use `PortfolioPosition`), and the legacy `BarType`/`BarSpecification`/`BarAggregation`/`QuoteTick`/`TradeTick`/`AggressorSide`/`PriceType`/`Product` names (→ superseded by the existing `Tick`/`Bar` design in `src/market.rs`, which predates this phase).

### Phase 6: Pending — Not Yet Done
The five legacy `honba-kokoro-*` crates are now deleted from disk entirely (they were untracked, zero-content placeholders). That makes the following concrete, not hypothetical:

- [ ] **6.1 REST/transport-envelope types** — `ApiResponse`, `ApiVersion`, `Endpoint`/`ENDPOINTS`, `ErrorCode`/`ErrorCategory`/`ErrorDetail`, `HttpMethod`, `ParamLocation`, `ResponseEnvelope`, `WRITE_PATHS`, `write_endpoints` used to live in the deleted `honba-kokoro-messages`. These belong in `honba-ya-api` (already describes itself as the transport-agnostic API surface), **not** `honba-core` — deliberately not added here. Nobody has created them there yet; any consumer still importing them currently has no real type to point at.
- [ ] **6.2 Re-point real consumers** — every subsystem that imported from the deleted crates needs its `Cargo.toml` and source repointed to the new `honba-core` homes (or, for 6.1's symbols, to `honba-ya-api` once that lands). Status per subsystem:
  - `honba-kao` — in progress (re-pointing `Cargo.toml` + source to `honba-core`; `QuoteTick`→`Tick::Quote`, `Trade`→`FillReport` call sites need manual adaptation since the shapes differ; 6.1's symbols can't be resolved yet).
  - `honba-me`, `honba-ya`, `honba-tamashii`, `honba-nou` — not started.
- [ ] **6.3 Shape mismatches found while re-pointing `honba-kao`** (carry forward once confirmed): the legacy `Trade` type had method-style fee/cost accessors (`.costs()`) that `FillReport` has no equivalent for (no fee field at all) — any consumer that needs fee data off a fill will need that sourced from elsewhere, not invented on `FillReport`. The legacy `Order`/`OrderId` types don't map 1:1 onto `OrderRequest`/`OrderAck`/`FillReport`/plain `String` ids either; each call site needs individual judgment.
- [ ] **6.4 `honba-ya-engine`** — unrelated to this migration, but currently blocks `honba-kao`'s build regardless: the crate no longer exists on disk (someone is independently retiring it, per the original risk-migration plan's own "retire `honba-ya-engine`" step). `honba-kao` depends on it and needs re-pointing to its replacement (likely `honba-ya-actors`) before it can compile end to end.

---

## 3. Verification & CI Test Suite

- [x] `cargo test -p honba-core`: Full unit test coverage for dimensional arithmetic and serialization.
- [x] `proptest`: Property tests verifying dimensional invariants ($\text{Price} \times \text{Quantity} = \text{Money}$).
- [x] AST lint checks ensuring zero usage of `f32`/`f64` throughout `honba-core/src/`.
