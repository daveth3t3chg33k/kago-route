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

# Simulate an Africa's Talking USSD callback (form-encoded).
# The compose setup enables a dev secret, so send the key header:
curl -s -X POST http://localhost:8080/ussd/callback \
  -H "X-KagoRoute-Key: dev_secret_change_me" \
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

The dashboard polls the engine's read endpoints (`/health`, `/flow`, `/sessions`) every few
seconds and renders a live overview: engine health, session store/database status, the loaded
flow's metadata, and active USSD sessions.

```bash
cd dashboard
cp .env.local.example .env.local   # optional: point NEXT_PUBLIC_ENGINE_URL elsewhere
npm install
npm run dev          # http://localhost:3000
```

`NEXT_PUBLIC_ENGINE_URL` defaults to `http://localhost:8080` and is inlined at build time
(the Dockerfile passes it as a build `ARG`). CORS on the engine must include the dashboard
origin (default `CORS_ALLOWED_ORIGINS` already covers `http://localhost:3000`).

## Engine API (v0 scaffold)

| Method | Path               | Description                                    |
|--------|--------------------|------------------------------------------------|
| GET    | `/health`          | Liveness + session store, database & flow status |
| GET    | `/flow`            | Metadata of the loaded menu schema              |
| GET    | `/flow/schema`     | Full loaded schema as a `FlowDocument` (for the visual builder) |
| GET    | `/sessions`        | Live USSD sessions (id + cached variables), for the dashboard |
| POST   | `/ussd/callback`   | Inbound carrier/aggregator USSD callback; accepts `application/x-www-form-urlencoded` or `application/json` with `sessionId`, `serviceCode`, `phoneNumber`, `text`; walks the loaded menu schema and replies `CON`/`END` text. |

### Security

- **Callback auth:** set `USSD_CALLBACK_SECRET` (single tenant) or `USSD_CALLBACK_SECRETS`
  (comma-separated, one key per tenant). `/ussd/callback` then requires the `X-KagoRoute-Key`
  header to match one of them (constant-time comparison). Enforced by a middleware that runs
  *before* the handler, so unauthorized bodies are never read. Unset → unauthenticated with a
  warning at boot (dev only).
- **Body limit:** callback bodies are capped at `USSD_MAX_BODY_BYTES` (default 4096); over the
  limit → `413`.
- **CORS:** restricted to `CORS_ALLOWED_ORIGINS` (comma-separated; default
  `http://localhost:3000`). Disallowed browser origins (and their preflights) get no CORS
  headers, so browsers block them; requests without an `Origin` header (carriers, curl) pass
  through untouched.

> **Note:** only `POST /ussd/callback` is authenticated. The read endpoints (`/health`,
> `/flow`, `/sessions`) are intentionally open for the dev dashboard; `/sessions` exposes
> cached session variables (e.g. order amounts), so gate it behind the flows/auth API before
> running on a network real tenants can reach.

### Menu schemas

The engine loads a flow from `FLOW_SCHEMA_PATH` (JSON or YAML) at boot and validates it
fail-closed (dangling targets, screen-length limits, timeouts, cycles). If unset, an embedded
demo flow (the `farmer-order` example) is served. See
[`docs/ussd-menu-schema.md`](docs/ussd-menu-schema.md) for the DSL and
[`docs/examples/`](docs/examples/) for flows you can point `FLOW_SCHEMA_PATH` at.

## Roadmap

1. ✅ **Menu schema DSL** — spec in [`docs/ussd-menu-schema.md`](docs/ussd-menu-schema.md),
   formal JSON Schema in [`docs/menu-schema.schema.json`](docs/menu-schema.schema.json),
   example flows in [`docs/examples/`](docs/examples/).
2. ✅ **Engine** — schema-driven session walker: parses the DSL with serde (Appendix B),
   validates fail-closed at boot, replays cumulative carrier text through the graph, and
   replies `CON`/`END`. Session variables & loop-guard state in Redis (in-memory fallback).
3. **Outbound webhooks** — async relay of completed flows into client backends (idempotent, retried).
4. ⏳ **Dashboard** — live overview shell is done (engine health + flow + live sessions);
   visual menu builder, payload logs, and API-key management remain.
5. **M-Pesa (Daraja STK push)** — fire STK push from `payments.mpesa`, consume callback, relay receipt.
6. **Billing** — tiered subscriptions + per-session usage markup.

## License

Proprietary — internal scaffold. © KagoRoute.
