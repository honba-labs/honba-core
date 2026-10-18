# 本場・核 honba-core

> *The canonical core of Honba.*

Shared core domain models, exact decimal arithmetic, typed market data feeds, canonical orders, and strictly-validated AI decision types for the Honba Indian equities trading platform (NSE / BSE / MCX).

## Features
- **Canonical Instruments**: `InstrumentId`, `Exchange` (NSE, BSE, MCX), ISIN and token mapping.
- **Exact Decimal Money**: `Price(Decimal)` and `Quantity(Decimal)` eliminating floating-point rounding errors.
- **Typed Market Data**: `Tick::Ltp`, `Tick::Quote` (with `PriceBand`), `Tick::Depth`, and `Bar`.
- **Order State & Execution**: `OrderRequest` (mandating `algo_id` per SEBI guidelines), `OrderAck`, `FillReport`.
- **AI Decision Harness**: Closed, strictly validated `Decision` enum with `deny_unknown_fields` protecting against LLM hallucinations.
- **Microstructure Compliance**: Scrip-wise `PriceBand` and circuit limit definitions.

## Testing
```bash
cargo test
```

## License
AGPL-3.0
