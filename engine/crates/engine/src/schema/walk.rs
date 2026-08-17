//! The session walker: replays the cumulative carrier `text` through the flow
//! graph and produces a `CON`/`END` reply (spec §5, §6, §13).
//!
//! Design: cumulative text is the source of truth — the walker re-derives the
//! user's position and all variables by replaying the graph from `start` on
//! every callback. The session store is used for the loop-guard counter and as
//! a variable cache (spec §1: Redis is an optimization, not the source of
//! truth).

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use serde_json::Value;

use crate::schema::expr::{eval_conditions, interpolate, resolve_set_value, Context};
use crate::schema::input::validate_input;
use crate::schema::{Flow, Node};
use crate::session::SessionStore;

/// Maximum consecutive invalid repeats at one node before forced termination.
pub const LOOP_GUARD_LIMIT: u32 = 5;
const MAX_STEPS: usize = 256;

pub struct WalkRequest<'a> {
    pub flow: &'a Arc<Flow>,
    pub text: &'a str,
    pub phone: &'a str,
    pub session_id: &'a str,
    pub service_code: &'a str,
}

pub struct WalkOutcome {
    /// Raw reply body: starts with `CON ` or `END `.
    pub body: String,
    pub ended: bool,
    /// Node id at the end of the walk (or the `goto` target after an error).
    pub node_id: String,
    /// Session variables after this callback.
    pub variables: HashMap<String, Value>,
}

#[derive(Debug)]
struct PendingError {
    message: String,
}

pub async fn walk(store: &SessionStore, req: &WalkRequest<'_>) -> WalkOutcome {
    let prefix = format!("session:{}", req.session_id);
    let ttl = Duration::from_secs(req.flow.timeouts.session as u64 + 60);

    // Cumulative text → segments; empty input means "first screen".
    let segments: Vec<&str> = req.text.split('*').filter(|s| !s.is_empty()).collect();

    let mut vars: HashMap<String, Value> = HashMap::new();
    let mut current = req.flow.start.clone();
    let mut idx = 0usize;
    let mut pending_error: Option<PendingError> = None;
    let mut steps = 0usize;

    loop {
        steps += 1;
        if steps > MAX_STEPS {
            tracing::warn!(session = %req.session_id, "walker exceeded step budget");
            return finish(store, &prefix, &ttl, vars, current.clone(), "END Walk failed. Please try again.", true).await;
        }

        let Some(node) = req.flow.nodes.get(&current) else {
            return finish(store, &prefix, &ttl, vars, current.clone(), "END Walk failed. Please try again.", true).await;
        };

        match node {
            Node::Menu(m) => {
                if idx < segments.len() {
                    let choice = segments[idx];
                    idx += 1;

                    // Find the first matching branch (ORed in order).
                    let picked = m.options.get(choice).and_then(|opt| {
                        opt.branches().into_iter().find(|b| {
                            let ctx = ctx_for(req, &vars);
                            b.when.is_empty() || eval_conditions(&b.when, &ctx)
                        })
                    });

                    match picked {
                        Some(branch) => {
                            apply_branch(branch, &mut vars, req);
                            clear_guard(store, &prefix).await;
                            pending_error = None;
                            current = branch.goto.clone();
                        }
                        None => {
                            let (message, goto) = menu_recovery(m, &current);
                            pending_error = Some(PendingError { message });
                            current = goto;
                            let count = bump_guard(store, &prefix, &current, &ttl).await;
                            if count > LOOP_GUARD_LIMIT {
                                return finish(store, &prefix, &ttl, vars, current.clone(), "END Too many invalid attempts. Please try again later.", true).await;
                            }
                        }
                    }
                } else {
                    // No more input: render the menu (or the pending error).
                    let body = match pending_error.take() {
                        Some(e) => format!("CON {}", e.message),
                        None => {
                            let ctx = ctx_for(req, &vars);
                            format!("CON {}", interpolate(&m.text, &ctx))
                        }
                    };
                    return finish(store, &prefix, &ttl, vars, current.clone(), body, false).await;
                }
            }

            Node::Input(i) => {
                if idx < segments.len() {
                    let raw = segments[idx];
                    idx += 1;

                    let result = match &i.validate {
                        Some(v) => validate_input(v, raw),
                        None => Ok(Value::String(raw.to_string())),
                    };

                    match result {
                        Ok(value) => {
                            vars.insert(i.variable.clone(), value);
                            clear_guard(store, &prefix).await;
                            pending_error = None;
                            current = i.next.clone();
                        }
                        Err(message) => {
                            let (recovery_message, goto) = input_recovery(i, &current, message);
                            pending_error = Some(PendingError { message: recovery_message });
                            current = goto;
                            let count = bump_guard(store, &prefix, &current, &ttl).await;
                            if count > LOOP_GUARD_LIMIT {
                                return finish(store, &prefix, &ttl, vars, current.clone(), "END Too many invalid attempts. Please try again later.", true).await;
                            }
                        }
                    }
                } else {
                    let body = match pending_error.take() {
                        Some(e) => format!("CON {}", e.message),
                        None => {
                            let ctx = ctx_for(req, &vars);
                            format!("CON {}", interpolate(&i.prompt, &ctx))
                        }
                    };
                    return finish(store, &prefix, &ttl, vars, current.clone(), body, false).await;
                }
            }

            Node::Action(a) => {
                for (name, value) in &a.set {
                    let ctx = ctx_for(req, &vars);
                    vars.insert(name.clone(), resolve_set_value(value, &ctx));
                }
                for (name, expr) in &a.compute {
                    let ctx = ctx_for(req, &vars);
                    vars.insert(name.clone(), Value::from(crate::schema::expr::eval_expr(expr, &ctx)));
                }
                current = a.next.clone();
            }

            Node::End(e) => {
                let ctx = ctx_for(req, &vars);
                let body = format!("END {}", interpolate(&e.text, &ctx));
                return finish(store, &prefix, &ttl, vars, current.clone(), body, true).await;
            }
        }
    }
}

// ── Helpers ──────────────────────────────────────────────────────────────

fn ctx_for<'a>(req: &'a WalkRequest<'_>, vars: &'a HashMap<String, Value>) -> Context<'a> {
    let last_input = req
        .text
        .split('*')
        .filter(|s| !s.is_empty())
        .last()
        .unwrap_or("");
    Context::new(
        vars,
        req.phone,
        req.session_id,
        req.service_code,
        &req.flow.id,
        req.flow.version,
        last_input,
    )
}

fn apply_branch(branch: &crate::schema::Branch, vars: &mut HashMap<String, Value>, req: &WalkRequest<'_>) {
    for (name, value) in &branch.set {
        let ctx = ctx_for(req, vars);
        vars.insert(name.clone(), resolve_set_value(value, &ctx));
    }
    for (name, expr) in &branch.compute {
        let ctx = ctx_for(req, vars);
        vars.insert(name.clone(), Value::from(crate::schema::expr::eval_expr(expr, &ctx)));
    }
}

/// Menu invalid-input recovery: custom `onInvalid` or re-render self.
fn menu_recovery(m: &crate::schema::MenuNode, node_id: &str) -> (String, String) {
    match &m.on_invalid {
        Some(r) => (
            r.text
                .clone()
                .unwrap_or_else(|| "Invalid option. Please try again.".to_string()),
            r.goto.clone(),
        ),
        None => ("Invalid option. Please try again.".to_string(), node_id.to_string()),
    }
}

/// Input invalid-input recovery: `onInvalid` (preferred) or validation message
/// or a generic one; `goto` defaults to the same node.
fn input_recovery(
    i: &crate::schema::InputNode,
    node_id: &str,
    validation_message: String,
) -> (String, String) {
    match &i.on_invalid {
        Some(r) => (
            r.text
                .clone()
                .unwrap_or(validation_message),
            r.goto.clone(),
        ),
        None => (validation_message, node_id.to_string()),
    }
}

/// Persist variables and return the outcome.
async fn finish(
    store: &SessionStore,
    prefix: &str,
    ttl: &Duration,
    vars: HashMap<String, Value>,
    node_id: String,
    body: impl Into<String>,
    ended: bool,
) -> WalkOutcome {
    if let Ok(json) = serde_json::to_string(&vars) {
        store.set(&format!("{prefix}:vars"), &json, *ttl).await;
    }
    WalkOutcome {
        body: body.into(),
        ended,
        node_id,
        variables: vars,
    }
}

async fn bump_guard(store: &SessionStore, prefix: &str, node: &str, ttl: &Duration) -> u32 {
    let key = format!("{prefix}:guard");
    let previous = store.get(&key).await.unwrap_or_default();
    let (last_node, count) = match previous.split_once('|') {
        Some((n, c)) => (n.to_string(), c.parse::<u32>().unwrap_or(0)),
        None => (String::new(), 0),
    };
    let count = if last_node == node { count + 1 } else { 1 };
    store.set(&key, &format!("{node}|{count}"), *ttl).await;
    count
}

async fn clear_guard(store: &SessionStore, prefix: &str) {
    store.delete(&format!("{prefix}:guard")).await;
}

// ── Tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::load_flow;
    use crate::session::memory::MemoryStore;

    async fn run(
        store: &SessionStore,
        flow: &Arc<Flow>,
        session_id: &str,
        text: &str,
    ) -> WalkOutcome {
        walk(
            store,
            &WalkRequest {
                flow,
                text,
                phone: "254712345678",
                session_id,
                service_code: "*483*42#",
            },
        )
        .await
    }

    fn demo() -> Arc<Flow> {
        load_flow(None).expect("demo loads")
    }

    #[tokio::test]
    async fn first_screen_opens_with_con() {
        let store = SessionStore::Memory(MemoryStore::default());
        let flow = demo();
        let out = run(&store, &flow, "t1", "").await;
        assert!(!out.ended);
        assert!(out.body.starts_with("CON "));
        assert!(out.body.contains("Tuma Farm Supplies"));
        assert_eq!(out.node_id, "welcome");
    }

    #[tokio::test]
    async fn select_product_advances() {
        let store = SessionStore::Memory(MemoryStore::default());
        let flow = demo();
        let out = run(&store, &flow, "t1", "1").await;
        assert!(out.body.starts_with("CON "));
        assert!(out.body.contains("Select product"));
    }

    #[tokio::test]
    async fn product_set_lands_on_qty_prompt() {
        let store = SessionStore::Memory(MemoryStore::default());
        let flow = demo();
        let out = run(&store, &flow, "t1", "1*1").await;
        assert!(out.body.contains("How many bags of maize-seed"));
        assert_eq!(out.variables.get("product").unwrap(), &Value::String("maize-seed".into()));
    }

    #[tokio::test]
    async fn action_compute_then_confirm_screen() {
        let store = SessionStore::Memory(MemoryStore::default());
        let flow = demo();
        // 1*1*2 → maize-seed, qty 2 → total 7000 → confirm
        let out = run(&store, &flow, "t1", "1*1*2").await;
        assert!(!out.ended);
        assert!(out.body.contains("KES 7000"));
        assert_eq!(out.node_id, "confirm");
        assert_eq!(out.variables.get("total").unwrap(), &Value::from(7000.0));
    }

    #[tokio::test]
    async fn large_order_goes_to_flagged_branch() {
        let store = SessionStore::Memory(MemoryStore::default());
        let flow = demo();
        // qty 5 → total 17500 ≥ 10000 → stk_flagged (END)
        let out = run(&store, &flow, "t1", "1*1*5*1").await;
        assert!(out.ended);
        assert!(out.body.starts_with("END "));
        assert!(out.body.contains("agent will call"));
        assert_eq!(out.node_id, "stk_flagged");
    }

    #[tokio::test]
    async fn small_order_goes_to_standard_branch() {
        let store = SessionStore::Memory(MemoryStore::default());
        let flow = demo();
        // qty 2 → total 7000 < 10000 → stk_standard (END)
        let out = run(&store, &flow, "t1", "1*1*2*1").await;
        assert!(out.ended);
        assert!(out.body.contains("M-Pesa PIN"));
        assert_eq!(out.node_id, "stk_standard");
    }

    #[tokio::test]
    async fn fertilizer_flow() {
        let store = SessionStore::Memory(MemoryStore::default());
        let flow = demo();
        let out = run(&store, &flow, "t1", "1*2*3").await;
        assert!(out.body.contains("fertilizer"));
        assert!(out.body.contains("KES 6600"));
    }

    #[tokio::test]
    async fn invalid_menu_option_shows_error() {
        let store = SessionStore::Memory(MemoryStore::default());
        let flow = demo();
        let out = run(&store, &flow, "t1", "9").await;
        assert!(!out.ended);
        assert!(out.body.contains("Invalid option"));
    }

    #[tokio::test]
    async fn invalid_qty_shows_custom_message() {
        let store = SessionStore::Memory(MemoryStore::default());
        let flow = demo();
        let out = run(&store, &flow, "t1", "1*1*abc").await;
        assert!(!out.ended);
        assert!(out.body.contains("Enter a whole number between 1 and 50."));
    }

    #[tokio::test]
    async fn qty_out_of_range_retries() {
        let store = SessionStore::Memory(MemoryStore::default());
        let flow = demo();
        let out = run(&store, &flow, "t1", "1*1*99").await;
        assert!(out.body.contains("between 1 and 50"));
        // Then a valid qty continues the flow.
        let out = run(&store, &flow, "t1", "1*1*99*3").await;
        assert!(out.body.contains("KES 10500"));
    }

    #[tokio::test]
    async fn loop_guard_terminates_after_limit() {
        let store = SessionStore::Memory(MemoryStore::default());
        let flow = demo();
        let mut text = String::from("9");
        let mut last = run(&store, &flow, "t1", &text).await;
        for _ in 1..LOOP_GUARD_LIMIT {
            text.push_str("*9");
            last = run(&store, &flow, "t1", &text).await;
            assert!(!last.ended, "should still be open at repeat {}", i);
        }
        // One more invalid repeat trips the guard.
        text.push_str("*9");
        let out = run(&store, &flow, "t1", &text).await;
        assert!(out.ended);
        assert!(out.body.contains("Too many invalid attempts"));
    }

    #[tokio::test]
    async fn exit_option_ends_session() {
        let store = SessionStore::Memory(MemoryStore::default());
        let flow = demo();
        let out = run(&store, &flow, "t1", "0").await;
        assert!(out.ended);
        assert!(out.body.contains("Thank you"));
        assert_eq!(out.node_id, "farewell");
    }

    #[tokio::test]
    async fn cancel_from_confirm_returns_to_welcome() {
        let store = SessionStore::Memory(MemoryStore::default());
        let flow = demo();
        let out = run(&store, &flow, "t1", "1*1*2*2").await;
        assert!(!out.ended);
        assert!(out.body.contains("Tuma Farm Supplies"));
    }

    #[tokio::test]
    async fn variables_are_cached_in_store() {
        let store = SessionStore::Memory(MemoryStore::default());
        let flow = demo();
        run(&store, &flow, "t1", "1*1*5*1").await;
        let cached = store.get("session:t1:vars").await.unwrap();
        assert!(cached.contains("\"total\""));
        assert!(cached.contains("17500"));
    }

    #[tokio::test]
    async fn sessions_are_isolated() {
        let store = SessionStore::Memory(MemoryStore::default());
        let flow = demo();
        let a = run(&store, &flow, "a", "1*1*2").await;
        let b = run(&store, &flow, "b", "").await;
        assert!(a.body.contains("KES 7000"));
        assert!(b.body.contains("Tuma Farm Supplies"));
        assert!(b.variables.is_empty());
    }
}
