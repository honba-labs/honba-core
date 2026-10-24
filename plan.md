# Master Plan: 本場・核 honba-core

> **The shared kernel of the entire platform.**  
> Houses canonical value objects, domain contracts, closed decision grammars, dynamic position sizing contracts, and wire primitives. Strict zero-I/O purity, zero external runtime dependencies, 100% exact 128-bit decimal arithmetic (`rust_decimal::Decimal`), and `schemars` type derivations driving platform-wide contract codegen.

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
               │  - Position Sizing Contracts: SizingRequest / Result   │
               └──────────────────────────┬─────────────────────────────┘
                                          │
         ┌──────────────────┬─────────────┼──────────────┬──────────────────┐
         ▼                  ▼             ▼              ▼                  ▼
    honba-kokoro        honba-mon       honba-me       honba-te          honba-ya
    (Domain Heart)     (Risk Gate)     (Market Eye)  (Execution Hand)  (Central House)
```

---

## 1. Architectural Invariants

1. **Zero External Dependencies**: Beyond `rust_decimal`, `serde`, `schemars`, and standard library utilities, `honba-core` contains zero dependencies. No Tokio, no HTTP clients, no database drivers.
2. **Zero Floats Policy**: All prices, quantities, cash balances, risk percentages, and financial metrics are represented as `rust_decimal::Decimal`. Generic floats (`f32`/`f64`) are strictly forbidden.
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
  - [x] Ensure all domain types implement `JsonSchema``.
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

### Phase 6: Subsystem Transition & Legacy Absorption Alignment (COMPLETED)
The five legacy `honba-kokoro-*` placeholder crates were excised across the workspace. All integration and contract concerns have been addressed:

- [x] **6.1 REST/transport-envelope types** (COMPLETED)
  - `ApiResponse`, `ApiVersion`, `Endpoint`/`ENDPOINTS`, `ErrorCode`/`ErrorDetail`, `HttpMethod`, `ParamLocation`, `ResponseEnvelope`, `WRITE_PATHS`, `write_endpoints`, and `TradingState` are established and exported in `honba-ya-api` (`crates/honba-ya-api/src/wire.rs`).
  - Strict DDD Shared Kernel boundary preserved: `honba-core` remains free of transport-envelope and HTTP concerns, maintaining zero-I/O and zero-framework purity.
- [x] **6.2 Re-point real consumers** (COMPLETED)
  - All workspace subsystems that previously referenced `honba-kokoro-*` placeholder crates have been repointed to canonical `honba-core` value objects/wire contracts (and `honba-ya-api` for transport models):
    - `honba-me`: Excised all legacy kokoro dependencies across all member crates; market feed contracts standardized on `honba-core` (`cargo check` clean).
    - `honba-ya`: Excised all usages of `honba-kokoro-*`, `honba_messages`, `honba_entities`, `honba_risk`, `honba_ports` across all 10 member crates (`cargo check` clean).
    - `honba-tamashii`: Workspace dependencies cleaned; PyO3 & Python bindings verified against `honba-core` (`cargo check` & tests clean).
    - `honba-nou`: Migrated all decision, risk, and analytics crates to `honba-core` and `honba-mon` (`cargo check` clean).
    - `honba-kao`: Workspace manifests updated to depend on `honba-core` and `honba-ya-api`.
    - `honba-kokoro` & `honba-te`: Core engine and execution layers unified under `honba-core`.\n- [x] **6.3 Shape mismatches resolved on `FillReport` and Orders** (COMPLETED)
  - **Fill transaction fees**: Addressed directly in `honba-core` (commit `f63a3c7`). `FillReport` now carries `fee: Money` (defaulting to zero via `Money::zero_inr()`) and an optional itemized `fee_breakdown: Option<FeeBreakdown>` (`brokerage`, `stt`, `exchange_charges`, `gst`, `stamp_duty`, `sebi_charges`). Builder methods `.with_fee()` and `.with_fee_breakdown()` provide ergonomic construction while keeping calculation logic in `honba-me-market::india::costs`.
  - **Enum size optimization**: Boxed `OrderEvent::Filled(Box<FillReport>)` to prevent enum bloat and pass clippy lint checks. Bumped `SCHEMA_VERSION` to 7.
  - **Order & Identifier alignment**: Standardized `OrderRequest`, `OrderAck`, `FillReport`, `OrderStatus`, and typed identifiers (`InstrumentId`, `VenueOrderId`, `TradeId`, `DatasetId`) across the platform.
- [x] **6.4 Retirement of `honba-ya-engine`** (COMPLETED)
  - `honba-ya-engine` has been retired across the workspace, replaced by `honba-ya-actors` (authoritative actor execution runtime), `honba-ya-backtest` (institutional event-driven backtesting), and `honba-ya-persistence` (SEBI regulatory audit trail). Downstream consumers previously referencing the legacy engine now resolve types against `honba-core` and `honba-ya-persistence`.

### Phase 7: Canonical Dynamic Position Sizing Contracts & Hexagonal Port (M7) (COMPLETED)
> **First-class dynamic position sizing grammar, risk budgets, and deterministic math in the Shared Kernel.**  
> Decouples alpha signal generation ("what to trade") from position sizing ("how much to trade"). Replaces static order sizes with pure, 128-bit decimal sizing models: fixed fractional risk, volatility targeting (ATR), Kelly criterion, and equal risk contribution.

- [x] **7.1 Position Sizing Domain Models & Value Objects (`src/sizing.rs`)**
  - [x] `SizingMethod` enum (`#[serde(deny_unknown_fields)]`):
    - `FixedFractional { risk_pct: Decimal }`: Risk a fixed % of account equity per trade (e.g. 1%).
    - `VolatilityTarget { target_annual_vol: Decimal, lookback_bars: usize }`: Size inversely to instrument volatility (ATR or standard deviation).
    - `KellyCriterion { fraction: Decimal, win_rate: Decimal, payoff_ratio: Decimal }`: Optimal growth sizing with damping fraction (e.g. half-Kelly).
    - `EqualRiskContribution`: Allocate risk budget equally across all active signal candidates.
    - `FixedNotional { notional: Money }`: Allocate fixed cash notional dynamically converted to lot-rounded quantity.
  - [x] `RiskBudget` struct:
    - Account equity (`equity: Money`), risk per trade (`risk_pct: Decimal`), maximum position notional cap (`max_position_notional: Option<Money>`), maximum position share/contract cap (`max_position_qty: Option<Quantity>`), maximum portfolio aggregate risk (`max_portfolio_risk_pct: Option<Decimal>`), instrument minimum lot size (`min_lot_size: Quantity`), and scrip freeze limit (`freeze_limit_qty: Option<Quantity>`).
  - [x] `PositionSizingRequest` wire struct:
    - `instrument: InstrumentId`, `side: PositionSide`, `entry_price: Price`, `stop_loss: Option<Price>`, `current_volatility: Option<Decimal>`, `account_equity: Money`, `risk_budget: RiskBudget`, `sizing_method: SizingMethod`.
  - [x] `PositionSizingResult` wire struct:
    - `instrument: InstrumentId`, `quantity: Quantity` (strictly rounded to lot/tick step), `allocated_notional: Money`, `risk_at_stake: Money` ($|P_{entry} - P_{stop}| \times Q$), `target_weight: Option<Decimal>`, `capped_by_limit: bool`, `reason: Option<String>`.
- [x] **7.2 Trade Signal Contract Enrichment (`src/signal.rs`)**
  - [x] Enrich `Signal` with optional execution and risk levels:
    - `entry_price: Option<Price>`, `stop_loss: Option<Price>`, `take_profit: Option<Price>`, `volatility_atr: Option<Decimal>`.
    - Enables upstream signal strategies to emit stop distances directly consumable by downstream position sizing steps.
- [x] **7.3 Hexagonal Position Sizer Port Trait (`src/ports.rs`)**
  - [x] Define object-safe, zero-I/O `PositionSizer` trait:
    ```rust
    pub trait PositionSizer: Send + Sync {
        fn calculate_size(&self, request: &PositionSizingRequest) -> Result<PositionSizingResult, PortError>;
    }
    ```
  - [x] Pure functional execution: zero floating-point drift, deterministic 128-bit decimal arithmetic (`rust_decimal`), provided via `CanonicalPositionSizer` and `calculate_position_size`.
- [x] **7.4 Schemars & Domain Schema Codegen**
  - [x] Ensure all sizing types derive `JsonSchema` and export to `schema/domain/domain_schema.json`.
  - [x] Automated schema codegen driving pipeline validation in Python (`honba-tamashii` / `honba-strategies`) and Web GUI (`honba-kao`).

### Phase 8: Tactical DDD Domain Events, Observability Trace Carrier & SemVer Contract Freeze (M8) (COMPLETED)
> **Closing tactical DDD gaps, enabling distributed observability without runtime coupling, and enforcing semantic version stability for downstream Python & Rust consumers.**

- [x] **8.1 Tactical DDD First-Class Domain Events (`src/events.rs`)**
  - [x] Implement explicit domain event aggregate envelope `DomainEvent`:
    - `OrderPlaced`: Client order dispatched with registered `algo_id`, side, quantity, limit price, and timestamp.
    - `OrderModified`: In-flight order price/quantity amendment with original and revised parameters.
    - `OrderCancelled`: Order withdrawal with optional cancellation source and reason.
    - `OrderFilled`: Fill execution details referencing matched `FillReport`.
    - `RiskBreached`: Pre-trade risk gate denial event carrying rule identifier, attempted value, and threshold limit.
    - `PositionLiquidated`: Emergency square-off or MIS auto-liquidation notification.
    - `MarginCallIssued`: Intraday margin shortfall alert with required maintenance equity.
    - `SizingCalculated`: Sizing determination event capturing sizing model inputs, computed quantities, and capping rules.
  - [x] Ensure all domain events derive `Serialize`, `Deserialize`, `JsonSchema`, `Clone`, `Debug`, `PartialEq`.
- [x] **8.2 Distributed Observability Wire Context (`src/wire.rs`)**
  - [x] Define zero-dependency W3C trace context carrier:
    ```rust
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
    pub struct TraceContext {
        pub trace_id: [u8; 16],
        pub span_id: [u8; 8],
        pub trace_flags: u8,
    }
    ```
  - [x] Embed optional `trace_context: Option<TraceContext>` in `OrderRequest`, `OrderAck`, `FillReport`, and `DomainEvent` envelopes.
  - [x] Enables distributed OpenTelemetry tracing spans across actors, persistence, and external bridges with zero external crate dependencies.
- [x] **8.3 Semantic Versioning & Schema Stability Verification**
  - [x] Establish `schema/compatibility/` golden snapshots of exported JSON schemas (`v7_domain_schema.json`, `v8_domain_schema.json`).
  - [x] Add CI contract regression test asserting backward compatibility of generated schemas per SemVer guidelines (`tests/schema_compatibility_test.rs`).
  - [x] Guarantee that type modifications in `honba-core` follow semantic versioning rules, providing clear deprecation cycles for `honba-tamashii` and `honba-strategies`.

---

## 3. Verification & CI Test Suite

- [x] `cargo test -p honba-core`: Full unit test coverage for dimensional arithmetic and serialization.
- [x] `proptest`: Property tests verifying dimensional invariants ($\text{Price} \times \text{Quantity} = \text{Money}$).
- [x] `proptest`: Property tests verifying position sizing bounds (ensuring $\text{risk\_at\_stake} \le \text{equity} \times \text{risk\_pct}$ and $\text{quantity} \le \text{max\_position\_qty}$).
- [x] `proptest`: Property tests verifying Domain Event serialization roundtrips and TraceContext wire invariance.
- [x] Schema backward compatibility test asserting that no previously released schema fields are removed without major version bump.
- [x] AST lint checks ensuring zero usage of `f32`/`f64` throughout `honba-core/src/`.
