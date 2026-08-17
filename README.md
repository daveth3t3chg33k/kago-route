# KagoRoute

**Mirror your web and smartphone logic onto feature phones (*kabambe*) via USSD and SMS.**

KagoRoute is a developer-first integration layer for the African mobile ecosystem. Instead of
writing telco scripts, managing 180-second carrier timeouts, or hardcoding `CON`/`END` state
machines, developers publish a simple JSON/YAML **menu schema** and KagoRoute handles the rest:

- routes USSD sessions through local aggregators (**Africa's Talking**, **Celcom Africa**) and
  **Safaricom Daraja** (M-Pesa STK push),
- keeps session state in **Redis** and unified transaction data in **PostgreSQL**,
- webhooks completed flows back into your existing cloud backend (Postgres, Node, Python)
  so accounts, inventory, and ledgers stay in sync across smartphones *and* feature phones.

## Architecture

```
                      ┌──────────────────────────────────────────────┐
                      │               KagoRoute Engine               │
   feature phone      │  ┌──────────┐   ┌──────────┐   ┌──────────┐  │      ┌──────────────┐
   *123#  ───►  carrier ──►  callback  ──►  session   ──►  menu      │  ──►  client backend │
   (USSD)          aggregator    handler   store      walker      │      (webhooks)      │
                      │  (Axum)      │   (Redis)    │ (CON/END)  │      └──────────────┘
                      │              │              │            │
                      │              └──────┬───────┴────────────┤
                      │              ┌──────┴──────┐             │
                      │              │  PostgreSQL  │             │
                      │              │  (logs, tx)  │             │
                      │              └─────────────┘             │
                      └────────────────────────────────────────────┘
                                        ▲
                                        │ REST + WebSockets
                              ┌─────────┴─────────┐
                              │   Next.js Dashboard│  visual menu builder,
                              │   (developer UI)   │  live traffic, payload logs
                              └───────────────────┘
```

## Repository layout

```
.
├── docs/                    Menu-schema DSL spec, JSON Schema, example flows
├── engine/                  Rust workspace — the USSD/SMS engine (Axum + Tokio)
│   └── crates/engine/       core engine crate
│       ├── src/             config, routes, session store, demo menu walker
│       └── migrations/      SQL migrations (PostgreSQL)
├── dashboard/               Next.js (TypeScript + Tailwind) developer dashboard
├── docker-compose.yml       local dev: Postgres + Redis + engine + dashboard
└── README.md
```

## Quick start (Docker)

```bash
docker compose up --build
```

| Service   | URL / port        | Notes                                   |
|-----------|-------------------|-----------------------------------------|
| Engine    | http://localhost:8080 | `GET /health`, `POST /ussd/callback` |
| Dashboard | http://localhost:3000 | Next.js developer UI                  |
| Postgres  | localhost:5432     | `kago` / `kago`, database `kagoroute`   |
| Redis     | localhost:6379     | session cache                           |

Smoke test the engine:

```bash
curl -s http://localhost:8080/health

# Simulate an Africa's Talking USSD callback (form-encoded)
curl -s -X POST http://localhost:8080/ussd/callback \
  -d "sessionId=test-1&serviceCode=*123*2%23&phoneNumber=254712345678&text="
```

The engine replies `CON <menu>` / `END <message>` exactly like a carrier expects.

## Local development (without Docker)

### Engine (Rust)

```bash
cd engine
cargo run            # needs Redis + Postgres on localhost, or it degrades gracefully
```

`DATABASE_URL` / `REDIS_URL` / `PORT` / `RUST_LOG` are read from the environment or `.env`
(see `.env.example`). If Redis is unreachable the engine falls back to an in-memory session
store; if Postgres is unreachable it runs without persistence and logs a warning — so you can
hack on the callback handler with zero infrastructure.

### Dashboard (Next.js)

```bash
cd dashboard
npm install
npm run dev          # http://localhost:3000
```

## Engine API (v0 scaffold)

| Method | Path               | Description                                    |
|--------|--------------------|------------------------------------------------|
| GET    | `/health`          | Liveness + session store & database status     |
| POST   | `/ussd/callback`   | Inbound carrier/aggregator USSD callback; accepts `application/x-www-form-urlencoded` or `application/json` with `sessionId`, `serviceCode`, `phoneNumber`, `text`; replies `CON`/`END` text. |

## Roadmap

1. ✅ **Menu schema DSL** — spec in [`docs/ussd-menu-schema.md`](docs/ussd-menu-schema.md),
   formal JSON Schema in [`docs/menu-schema.schema.json`](docs/menu-schema.schema.json),
   example flows in [`docs/examples/`](docs/examples/).
2. **Engine** — schema-driven session walker (parse the DSL with serde per Appendix B),
   tenant isolation, adapter trait (Africa's Talking first).
3. **Outbound webhooks** — async relay of completed flows into client backends (idempotent, retried).
4. **Dashboard** — visual menu builder, live traffic monitor, payload logs, API-key management.
5. **M-Pesa (Daraja STK push)** — start payment from USSD flow, consume callback, relay receipt.
6. **Billing** — tiered subscriptions + per-session usage markup.

## License

Proprietary — internal scaffold. © KagoRoute.
