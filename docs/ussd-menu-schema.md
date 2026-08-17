# KagoRoute USSD Menu Schema — Specification

**Version:** 1.0 (DSL: `kagoroute/menu/1.0`)
**Status:** Draft for implementation
**Scope:** The declarative format developers use to describe USSD menu flows.

A *flow* describes a USSD session as a directed graph of **nodes**. When a carrier/aggregator
delivers a callback, the KagoRoute engine walks the graph, captures inputs into session
**variables**, and replies with `CON` (keep the session open) or `END` (terminate) — exactly as
the telco expects. No state machines, no telco scripting.

> The companion document [`menu-schema.schema.json`](menu-schema.schema.json) is the formal,
> machine-checkable definition of this spec. Example flows live in
> [`examples/`](examples/).

---

## 1. Design principles

1. **Declarative.** The flow is data. There is no control flow in code.
2. **Cumulative-text friendly.** Carriers send the *cumulative* input (`"1"`, then `"1*2"`,
   then `"1*2*500"`) on every callback. The engine can re-derive position from the cumulative
   text alone; the session store (Redis) is an optimization and a variable cache, **not** the
   source of truth. A flow that is correct against cumulative text stays correct even if Redis
   is flushed mid-session.
3. **A tiny, validated surface.** Four node types, one branching mechanism, one deliberately
   small expression language. Every field is validated by the engine at deploy time.
4. **Immutable versions.** A deployed flow version never changes. Edits produce a new version.
   In-flight sessions stay pinned to the version they started on.
5. **Fail closed.** Schemas with dangling `goto` targets, unknown node types, or cycles without
   progress are **rejected at deploy time**, not discovered in production.

---

## 2. Conventions

- YAML 1.2 or JSON. Both are equivalent in meaning (YAML is a superset); YAML is preferred for
  hand-authored flows, JSON for programmatic generation.
- All keys are `camelCase`.
- Node ids and variable names match `^[a-zA-Z][a-zA-Z0-9_]{0,63}$`.
- System variable names are reserved with a `$` prefix (`$phone`, `$sessionId`, ...). User
  variable names must **not** start with `$`.
- Screen text uses `\n` for line breaks. A literal `{` is written `{{`. `{variable}` is
  interpolation (see §8).

---

## 3. Top-level document

```yaml
schema: "kagoroute/menu/1.0"   # REQUIRED. DSL identifier + version; unknown values are rejected.
flow: { ... }                  # REQUIRED. The flow definition (§4).
```

Nothing else is allowed at the top level.

---

## 4. Flow object

```yaml
flow:
  id: "farmer-order"             # REQUIRED. Stable id, unique per tenant.
  name: "Farmer Supply Order"    # Display name for the dashboard.
  description: "..."             # Optional.
  version: 4                     # REQUIRED. Integer ≥ 1, monotonically increasing per id.
  start: "welcome"               # REQUIRED. Id of the entry node.
  timeouts:                      # Optional. Carrier budgets in seconds.
    session: 120                 #   Total session budget. MUST be ≤ 180 (carrier limit).
    step: 20                     #   Per-callback inactivity budget (default 20, cap 60).
  variables:                     # Optional. Declarations; aid validation, docs, and the
    - name: "total"              #   dashboard (engine does not require pre-declaration).
      type: "int"                #   Optional: int | float | text | phone | amount.
    - name: "phone"
      source: "caller"           #   Optional: "caller" = captured from the dialing number.
  webhooks:                      # Optional. Outbound integration (§10).
    onComplete:
      url: "https://api.example.com/v1/ussd/complete"
      secret: "whsec_..."        #   Used to sign requests (HMAC-SHA256).
      events: ["complete", "payment.result"]   # Default: ["complete"].
  nodes: { ... }                 # REQUIRED. Map of node id → node definition (§5).
```

Additional top-level keys in `flow` are rejected by validation.

---

## 5. Nodes

Every node except `action` carries `type` plus `onInvalid` / `onTimeout` recovery
(optional, §13). Actions are synchronous and fast, so they have no recovery path. Node
`type` is one of:

| type      | Renders      | Behaviour                                                        |
|-----------|--------------|------------------------------------------------------------------|
| `menu`    | `CON`        | Show numbered options; branch on the digit entered.              |
| `input`   | `CON`        | Capture free text into a variable, with validation.              |
| `action`  | *(none)*     | Set / compute variables, then advance. Must finish fast (§12).   |
| `end`     | `END`        | Final screen; terminates the session. May trigger a payment.     |

### 5.1 `menu`

```yaml
product:
  type: menu
  text: "Select product:\n1. Maize seed (KES 3,500/bag)\n2. Fertilizer (KES 2,200/bag)"
  options:
    "1": [{ set: { product: "maize-seed", unitPrice: 3500 }, goto: "qty" }]
    "2": [{ set: { product: "fertilizer", unitPrice: 2200 }, goto: "qty" }]
```

- `text` — the screen shown as `CON <text>`.
- `options` — map of option digit → branch (§6). Option keys are single digits (`0`–`9`).
  A value may be a single branch object or an array of branches (for `when` chains).
- Unknown digit → `onInvalid` (default: re-render this menu; loop guard applies, §13).

### 5.2 `input`

```yaml
qty:
  type: input
  prompt: "How many bags of {product}? (1-50)"
  variable: "qty"
  validate:
    type: int
    min: 1
    max: 50
  onInvalid: { text: "Enter a whole number between 1 and 50.", goto: "qty" }
  next: "totals"
```

- `prompt` — the screen shown as `CON <prompt>`.
- `variable` — the session variable that receives the input.
- `validate` — see §7.
- `next` — node to advance to once input is valid.
- The engine reads the **last `*`-segment** of the cumulative text as the candidate input.
  (Consequence: free-text inputs cannot contain `*` — a carrier constraint, §12.)

### 5.3 `action`

```yaml
totals:
  type: action
  compute:
    total: "unitPrice * qty"
  next: "confirm"
```

- No screen is rendered; the node executes `set` / `compute` (§9) and advances via `next`.
- **Latency contract:** an `action` runs inside the carrier's step window and must complete in
  under ~1 s. Anything slower (HTTP calls to a client backend, M-Pesa initiation, ...) belongs
  in a webhook or a payment, not an `action`.

### 5.4 `end`

```yaml    stk_standard:
      type: end
      text: "A payment request has been sent to your phone.\nEnter your M-Pesa PIN to confirm."
      payments:
        mpesa:
          shortCode: "483242"
          amountExpr: "total"
          phoneExpr: "$phone"
          accountRef: "TUMA-{qty}"
          transactionDesc: "Farm inputs"
```

- `text` — the final screen, rendered as `END <text>`. Session ends.
- `payments.mpesa` — optional STK-push intent. The engine fires Daraja's *Lipa na M-Pesa
  Online* **asynchronously after** the `END` reply (never inside the callback), then delivers
  the payment result via the `payment.result` webhook event (§10).

---

## 6. Options, branching & conditions

Branches are how a flow changes course. A branch:

```yaml
{ when: <condition|list>, set: { var: value, ... }, compute: { var: expr, ... }, goto: "<nodeId>" }
```

- `goto` — **REQUIRED** and must reference an existing node.
- `set` — literal assignments applied when the branch matches (`value` may reference system
  vars, e.g. `targetPhone: "$phone"`).
- `compute` — expression assignments applied when the branch matches (§9).
- `when` — a single condition or an ordered list of conditions. Conditions **within one
  branch are ANDed**; branches **within an `options` value are ORed** and evaluated in array
  order — the **first match wins**; a branch with **no `when`** is the default catch-all. If
  none match and there is no catch-all, the menu falls to `onInvalid`.

```yaml
confirm:
  type: menu
  text: "Order: {qty} x {product} = KES {total}\n1. Pay via M-Pesa\n2. Cancel"
  options:
    "1":
      - when: { var: "total", op: "gte", value: 10000 }
        goto: "stk_flagged"
      - goto: "stk_standard"
    "2": [{ goto: "welcome" }]
```

### Condition

```yaml
{ var: "total", op: "gte", value: 10000 }
```

| op          | value type | meaning                                   |
|-------------|------------|-------------------------------------------|
| `eq`        | any        | equals (loose compare: `"5"` == `5`)      |
| `neq`       | any        | not equals                                |
| `gt` / `gte`| number     | greater than (or equal)                   |
| `lt` / `lte`| number     | less than (or equal)                      |
| `contains`  | string     | substring test                            |
| `startsWith`| string     | prefix test                               |
| `matches`   | string     | regex match (engine uses Rust `regex`)    |
| `isSet`     | *(none)*   | variable exists and is non-empty          |
| `in`        | array      | variable is one of the listed values      |

`var` may reference a user variable or a system variable (`$phone`, ...). A missing variable
evaluates as `null` — `isSet` is the intended guard.

---

## 7. Validation reference

```yaml
validate:
  type: int | float | text | phone | amount | option   # REQUIRED
  min: 1            # numeric lower bound (int/float/amount)
  max: 20000        # numeric upper bound
  minLength: 1      # string length bounds (text)
  maxLength: 30
  pattern: "^\\+?254\\d{9}$"   # regex (text)
  options: ["a", "b"]          # enumerated values (option)
  message: "custom error"      # optional; overrides the default message
```

Semantics per `type`:

| type     | accepted input                                   | default error message                  |
|----------|--------------------------------------------------|----------------------------------------|
| `int`    | optional leading `-`, digits only                | `Enter a whole number.`                |
| `float`  | decimal number (`.` separator)                   | `Enter a number.`                      |
| `amount` | non-negative number, ≤ 2 decimals                | `Enter a valid amount.`                |
| `phone`  | 9–15 digits, optional leading `+`                | `Enter a valid phone number.`          |
| `text`   | anything (bounds/pattern apply)                  | `Invalid input.`                       |
| `option` | one of `options`                                 | `Choose a valid option.`               |

**Normalization:** `phone` is normalized to E.164 (`2547...`) on capture and written back to
the variable. `int`/`float`/`amount` are stored as numbers in the session variables (and thus
usable in `compute`).

---

## 8. Variables & interpolation

Variables are scoped to the session and cached in Redis with TTL = session timeout.

**Sources**

1. **Input capture** — `input` node writes its validated value to `variable`.
2. **Branches** — `set` (literals) and `compute` (expressions) on a matching branch.
3. **Actions** — `set` / `compute` maps on an `action` node.
4. **System variables** (read-only, `$` prefix):

   | name           | value                                             |
   |----------------|---------------------------------------------------|
   | `$phone`       | dialing number, normalized to E.164               |
   | `$sessionId`   | carrier session id                                |
   | `$serviceCode` | shortcode the user dialed (e.g. `*483*42#`)       |
   | `$flowId`      | flow id                                           |
   | `$flowVersion` | version of the flow serving this session          |
   | `$lastInput`   | the raw last segment received                     |

**Interpolation**

Any screen text (`text`, `prompt`, `onInvalid.text`, ...) supports `{var}`:
- A missing variable renders as the empty string and logs a warning at the engine.
- A literal `{` is escaped as `{{`.- Values are rendered as their canonical string: numbers unformatted, `amount` as an integer
  or 2-decimal string, `phone` as E.164.

**Example:** `text: "Order: {qty} x {product} = KES {total}"` with
`qty=2, product=maize-seed, total=7000` renders `Order: 2 x maize-seed = KES 7000`.

---

## 9. Expressions (`compute`)

Deliberately tiny — just enough for totals, discounts, and thresholds:

```
expr  := term (("+" | "-") term)*
term  := factor (("*" | "/") factor)*
factor:= number | var | "(" expr ")"
```

- Division truncates toward zero (integer semantics).
- Unknown variables evaluate as `0` (documented, and caught by a warning).
- Whitespace is ignored. `var` is any in-scope variable name, with or without `$`.
- Literal strings are **not** supported in expressions — use `set` for those.

**Examples:** `"unitPrice * qty"`, `"(subtotal + delivery) * 0.9"`.

---

## 10. Webhooks & payments

### Webhooks

When a session ends (an `end` node was reached, or a timeout terminated the session), the
engine POSTs to `flow.webhooks.onComplete.url`:

```json
{
  "event": "complete",
  "flowId": "farmer-order",
  "flowVersion": 4,
  "sessionId": "at_abc123",
  "phone": "254712345678",
  "serviceCode": "*483*42#",
  "variables": { "product": "maize-seed", "qty": 5, "total": 17500 },
  "completedAt": "2026-08-17T12:00:00Z",
  "payment": { "status": "pending" }
}
```

- **Signing:** header `X-KagoRoute-Signature` = HMAC-SHA256 over the raw request body, keyed
  with `secret`. Clients MUST verify it.
- **Idempotency:** header `X-KagoRoute-Idempotency` = `flowId:flowVersion:sessionId`. Clients
  may deduplicate on it.
- **Retries:** exponential backoff (1s, 5s, 30s, 5m, 15m), 5 attempts, only for
  transport/5xx failures (not for 4xx — those are treated as accepted-configuration errors and
  stop retrying).
- **Events:** `complete` (session ended normally), `invalid` (killed by the loop guard, §13),
  `timeout` (carrier timed the session out), `payment.result` (a Daraja STK-push callback
  arrived for a session that ended with `payments.mpesa`). The `payment.result` payload adds
  `payment: { status: "success"|"failed"|"cancelled", receipt, amount, transactionTime }`.

When a payment is present, the engine fires `complete` immediately (payment `pending`) and
`payment.result` later — the two are distinct events on the same idempotency namespace.

### Payments (M-Pesa STK push)

Defined on an `end` node (§5.4):

```yaml
payments:
  mpesa:
    shortCode: "483242"          # paybill used for the STK request
    amountExpr: "total"          # expression evaluating to the amount in KES
    phoneExpr: "$phone"          # expression evaluating to the paying phone
    accountRef: "TUMA-{qty}-{product}"   # optional; passed to Daraja
    transactionDesc: "Farm inputs order" # optional
```

- The engine derives `password`/`timestamp` from shortcode + passkey + timestamp and the
  result is consumed via the tenant's Daraja `CallBackURL` (managed by the engine; the client
  never touches Daraja directly).
- **Daraja field limits** (enforced by the JSON Schema): `accountRef` ≤ 12 chars,
  `transactionDesc` ≤ 13 chars — keep interpolation short (`TUMA-{qty}`, not
  `TUMA-{qty}-{product}`).

---

## 11. Versioning & deployment

- `flow.version` is a positive integer, monotonically increasing per `flow.id`.
- **Immutability:** once deployed (published), a version's definition never changes. Any edit
  in the dashboard or via API creates a new version.
- **Pinning:** sessions are pinned to the version that served their first callback. A user
  dialing again later gets the latest deployed version. Deploys are therefore always safe
  mid-traffic.
- **Resolution:** the engine serves `start` of the latest deployed version for new sessions;
  `flow.start.versionOverride` (optional, future) will support canary/percentage rollouts.
- **History:** all versions are retained in PostgreSQL (`flow_versions`) with an audit trail
  (who/when), used by the dashboard's version inspector and rollback.

---

## 12. Limits & carrier constraints

| Constraint                    | Limit                                          |
|-------------------------------|------------------------------------------------|
| Screen text                   | ≤ 160 chars (some carriers 182); **recommend ≤ 140** |
| Menu options                  | ≤ 9 digits (`1`–`9`; `0` conventionally "back")      || Session duration                   | ≤ 180 s total (carrier); default flow timeout 120 s   |
| Step timeout                       | ≤ 60 s (schema cap; default 20)                        |
| Free-text input length        | ≤ 30 chars                                     |
| Input segments                | inputs cannot contain `*` (field separator)    |
| Concurrent sessions per phone | 1 — USSD is modal                             |
| Charset                       | GSM/ASCII; emoji and most non-GSM glyphs unsupported |
| `action` latency              | < 1 s (runs inside the step window)           |
| Expression depth              | bounded (engine-imposed, documented cap: 32)  |

The engine **rejects** a schema at deploy time if any screen text exceeds the carrier limit,
if a `goto`/`next` target is missing, or if `session` timeout > 180.

---

## 13. Error handling

- **Invalid input** → `onInvalid` node (default: re-render the current node).
- **Loop guard:** after 5 consecutive repeats of the same node via `onInvalid`, the engine
  replies `END <generic goodbye>` and fires the `invalid` webhook event. Prevents dead loops.
- **Step timeout** → carrier ends the session; the engine fires the `timeout` webhook event.
- **Dangling references** → schema rejected at deploy (fail closed), §12.
- **Unknown DSL version** → schema rejected.
- **Render overflow** → schema rejected at deploy (text > limit), §12.

```yaml
onInvalid: { text: "Custom message", goto: "nodeId" }   # text optional
onTimeout: { goto: "nodeId" }                           # optional per node
```

---

## 14. Annotated walkthrough — `examples/farmer-order.yaml`

User dials `*483*42#`. Cumulative text and the engine trace:

| # | cumulative text | node        | reply                                                        |
|---|-----------------|-------------|--------------------------------------------------------------|
| 1 | `""`            | `welcome`   | `CON Tuma Farm Supplies\n1. Order inputs\n0. Exit`           |
| 2 | `"1"`           | `product`   | `CON Select product:\n1. Maize seed...\n2. Fertilizer...`    |
| 3 | `"1*1"`         | `qty`       | `CON How many bags of maize-seed? (1-50)`                    |
| 4 | `"1*1*5"`       | `totals` → `confirm` | `CON Order: 5 x maize-seed = KES 17500\n1. Pay via M-Pesa\n2. Cancel` |
| 5 | `"1*1*5*1"`     | `stk_flagged` | `END A payment request has been sent to your phone...`    |

Notes:

- Step 4's callback renders no visible `action` screen: `totals` computes silently, the
  walker lands on `confirm`, and its `CON` is the reply the user actually sees.
- At step 5 the `1` input selects the branch `when: total >= 10000` (17500), so the user
  lands on `stk_flagged`, not `stk_standard`. The engine replies `END` and fires the STK
  push asynchronously, then delivers `payment.result`.
- If the user pressed `2` at step 5, cumulative text would be `"1*1*5*1*2"` → `welcome`
  again (fresh session context).

---

## Appendix A — Formal JSON Schema

See [`menu-schema.schema.json`](menu-schema.schema.json) (draft-07). It enforces: top-level
shape, flow field types, node discrimination by `type`, option/branch structure, condition
operators, validation fields, id/variable naming patterns, and unknown-key rejection
(`additionalProperties: false`).

Validate examples with:

```bash
npx --yes ajv-cli validate -s docs/menu-schema.schema.json -d docs/examples/*.yaml -d docs/examples/*.json
```

---

## Appendix B — Rust type sketch (engine)

The engine will parse the DSL with `serde`; the internal-tagged `Node` enum mirrors the
`type` discriminator. This sketch is the target shape for `crates/engine/src/schema/`:

```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FlowDocument {
    pub schema: String, // "kagoroute/menu/1.0"
    pub flow: Flow,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Flow {
    pub id: String,
    pub name: String,
    #[serde(default)] pub description: Option<String>,
    pub version: u32,
    pub start: String,
    #[serde(default)] pub timeouts: Timeouts,
    #[serde(default)] pub variables: Vec<VariableDecl>,
    #[serde(default)] pub webhooks: Option<Webhooks>,
    pub nodes: HashMap<String, Node>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VariableDecl {
    pub name: String,
    #[serde(default, rename = "type")] pub kind: Option<String>, // int | float | text | phone | amount
    #[serde(default)] pub source: Option<String>,                // "caller"
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum Node {
    Menu(MenuNode),
    Input(InputNode),
    Action(ActionNode),
    End(EndNode),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MenuNode {
    pub text: String,
    pub options: HashMap<String, Vec<Branch>>, // single-object shorthand normalized to vec
    #[serde(default)] pub on_invalid: Option<Recovery>,
    #[serde(default)] pub on_timeout: Option<Recovery>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InputNode {
    pub prompt: String,
    pub variable: String,
    #[serde(default)] pub validate: Option<Validation>,
    #[serde(default)] pub on_invalid: Option<Recovery>,
    pub next: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ActionNode {
    #[serde(default)] pub set: HashMap<String, serde_json::Value>,
    #[serde(default)] pub compute: HashMap<String, String>,
    pub next: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EndNode {
    pub text: String,
    #[serde(default)] pub payments: Option<Payments>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Branch {
    #[serde(default)] pub when: Vec<Condition>, // single condition normalized to vec
    #[serde(default)] pub set: HashMap<String, serde_json::Value>,
    #[serde(default)] pub compute: HashMap<String, String>,
    pub goto: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Condition {
    pub var: String,
    pub op: Op,
    #[serde(default)] pub value: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Op {
    Eq, Neq, Gt, Gte, Lt, Lte,
    Contains, StartsWith, Matches, IsSet, In,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Validation {
    pub kind: ValidationKind,
    #[serde(default)] pub min: Option<f64>,
    #[serde(default)] pub max: Option<f64>,
    #[serde(default)] pub min_length: Option<usize>,
    #[serde(default)] pub max_length: Option<usize>,
    #[serde(default)] pub pattern: Option<String>,
    #[serde(default)] pub options: Option<Vec<String>>,
    #[serde(default)] pub message: Option<String>,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ValidationKind { Int, Float, Text, Phone, Amount, Option }

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Recovery {
    #[serde(default)] pub text: Option<String>,
    pub goto: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Payments {
    #[serde(default)] pub mpesa: Option<MpesaStkPush>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MpesaStkPush {
    pub short_code: String,
    pub amount_expr: String,
    pub phone_expr: String,
    #[serde(default)] pub account_ref: Option<String>,
    #[serde(default)] pub transaction_desc: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Timeouts {
    #[serde(default = "default_session_timeout")] pub session: u32, // 120
    #[serde(default = "default_step_timeout")] pub step: u32,       // 20
}
```

Notes for implementation:

- `#[serde(tag = "type")]` gives the `type: menu|input|action|end` discrimination for free.
- JSON Schema allows a branch value to be a single object or an array; normalize to `Vec<Branch>`
  with a custom `Deserialize` (or a `#[serde(untagged)]` wrapper enum).
- Deploy-time validation pass (fail closed): every `goto`/`next` exists; timeouts ≤ 180;
  screen text ≤ 160; expression AST depth ≤ 32; node id / variable-name patterns.

---

## Appendix C — Terminology

| term            | meaning                                                            |
|-----------------|--------------------------------------------------------------------|
| session         | one end-user interaction with the shortcode (a dial + keypresses)  |
| callback        | carrier/aggregator HTTP POST delivering the cumulative input       |
| `CON` / `END`   | the two USSD reply envelopes: continue vs terminate                |
| node            | a step in the flow graph (`menu`, `input`, `action`, `end`)        |
| branch          | a menu-option destination, optionally conditioned (`when`)         |
| variable        | session-scoped value (input capture, `set`, `compute`, system `$`) |
| flow version    | immutable, monotonically increasing revision of a flow             |
| STK push        | M-Pesa *Lipa na M-Pesa Online* initiation (Daraja)                 |
| webhook         | outbound POST to the client backend on session/payment events      |
| tenant          | a client organization; flows and API keys are tenant-scoped        |
