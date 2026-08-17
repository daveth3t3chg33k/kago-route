//! Deploy-time validation — fail closed (spec §12, §13).
//!
//! Rejects flows with: unknown DSL (checked at parse), missing start node,
//! dangling `goto`/`next` targets, screen text over the carrier limit,
//! timeouts out of bounds, malformed expressions/regexes, and cycles without
//! an input node (no possible progress).

use std::collections::HashMap;

use crate::schema::expr::expr_syntax_ok;
use crate::schema::{Condition, Flow, Node, Recovery, ValidationKind};

pub const MAX_SCREEN_CHARS: usize = 160;
pub const MAX_SESSION_TIMEOUT: u32 = 180;
pub const MAX_STEP_TIMEOUT: u32 = 60;

/// Validate a flow, returning a list of human-readable problems.
pub fn validate_flow(flow: &Flow) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();

    if !flow.nodes.contains_key(&flow.start) {
        errors.push(format!("start node '{}' does not exist", flow.start));
    }
    if flow.timeouts.session < 1 || flow.timeouts.session > MAX_SESSION_TIMEOUT {
        errors.push(format!(
            "session timeout {}s must be 1..={}",
            flow.timeouts.session, MAX_SESSION_TIMEOUT
        ));
    }
    if flow.timeouts.step < 1 || flow.timeouts.step > MAX_STEP_TIMEOUT {
        errors.push(format!(
            "step timeout {}s must be 1..={}",
            flow.timeouts.step, MAX_STEP_TIMEOUT
        ));
    }

    for (id, node) in &flow.nodes {
        check_node(id, node, flow, &mut errors);
    }

    check_cycles(flow, &mut errors);

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn check_node(id: &str, node: &Node, flow: &Flow, errors: &mut Vec<String>) {
    match node {
        Node::Menu(m) => {
            check_screen(&m.text, id, errors);
            if m.options.is_empty() {
                errors.push(format!("node '{id}': menu has no options"));
            }
            for (key, value) in &m.options {
                if key.len() != 1 || !key.chars().next().is_some_and(|c| c.is_ascii_digit()) {
                    errors.push(format!("node '{id}': option key '{key}' must be a single digit"));
                }
                for branch in value.branches() {
                    check_target(id, &branch.goto, flow, errors);
                    for (var, expr) in &branch.compute {
                        check_expr(id, var, expr, errors);
                    }
                    for var in branch.set.keys() {
                        check_var_name(id, var, errors);
                    }
                    for cond in &branch.when {
                        check_condition(id, cond, errors);
                    }
                }
            }
            check_recovery(id, &m.on_invalid, flow, errors);
            check_recovery(id, &m.on_timeout, flow, errors);
        }
        Node::Input(i) => {
            check_screen(&i.prompt, id, errors);
            check_var_name(id, &i.variable, errors);
            check_target(id, &i.next, flow, errors);
            if let Some(val) = &i.validate {
                check_validation(id, val, errors);
            }
            check_recovery(id, &i.on_invalid, flow, errors);
            check_recovery(id, &i.on_timeout, flow, errors);
        }
        Node::Action(a) => {
            check_target(id, &a.next, flow, errors);
            for (var, expr) in &a.compute {
                check_expr(id, var, expr, errors);
            }
            for var in a.set.keys() {
                check_var_name(id, var, errors);
            }
        }
        Node::End(e) => check_screen(&e.text, id, errors),
    }
}

fn check_screen(text: &str, id: &str, errors: &mut Vec<String>) {
    if text.chars().count() > MAX_SCREEN_CHARS {
        errors.push(format!(
            "node '{id}': screen text {} chars exceeds limit of {MAX_SCREEN_CHARS}",
            text.chars().count()
        ));
    }
}

fn check_target(id: &str, target: &str, flow: &Flow, errors: &mut Vec<String>) {
    if !flow.nodes.contains_key(target) {
        errors.push(format!("node '{id}': target '{target}' does not exist"));
    }
}

fn check_expr(id: &str, var: &str, expr: &str, errors: &mut Vec<String>) {
    check_var_name(id, var, errors);
    if !expr_syntax_ok(expr) {
        errors.push(format!("node '{id}': invalid expression '{expr}'"));
    }
}

fn check_condition(id: &str, cond: &Condition, errors: &mut Vec<String>) {
    if cond.op == crate::schema::Op::Matches {
        if let Some(pattern) = &cond.value {
            if let Some(s) = pattern.as_str() {
                if regex::Regex::new(s).is_err() {
                    errors.push(format!("node '{id}': invalid regex '{s}' in condition"));
                }
            }
        }
    }
    if cond.op == crate::schema::Op::In {
        if !cond.value.as_ref().is_some_and(|v| v.is_array()) {
            errors.push(format!("node '{id}': 'in' condition requires an array value"));
        }
    }
}

fn check_recovery(id: &str, recovery: &Option<Recovery>, flow: &Flow, errors: &mut Vec<String>) {
    if let Some(r) = recovery {
        check_target(id, &r.goto, flow, errors);
        if let Some(text) = &r.text {
            if text.chars().count() > MAX_SCREEN_CHARS {
                errors.push(format!(
                    "node '{id}': recovery text {} chars exceeds limit of {MAX_SCREEN_CHARS}",
                    text.chars().count()
                ));
            }
        }
    }
}

fn check_validation(id: &str, v: &crate::schema::Validation, errors: &mut Vec<String>) {
    if v.kind == ValidationKind::Option && v.options.is_empty() {
        errors.push(format!("node '{id}': 'option' validation requires an options list"));
    }
    if let Some(min) = v.min {
        if let Some(max) = v.max {
            if min > max {
                errors.push(format!("node '{id}': validation min > max"));
            }
        }
    }
    if let Some(pattern) = &v.pattern {
        if regex::Regex::new(pattern).is_err() {
            errors.push(format!("node '{id}': invalid validation regex '{pattern}'"));
        }
    }
}

fn check_var_name(id: &str, name: &str, errors: &mut Vec<String>) {
    let ok = !name.is_empty()
        && !name.starts_with('$')
        && name.chars().next().is_some_and(|c| c.is_ascii_alphabetic())
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_');
    if !ok {
        errors.push(format!(
            "node '{id}': invalid variable name '{name}' (must match ^[a-zA-Z][a-zA-Z0-9_]*$)"
        ));
    }
}

// ── Cycle detection ──────────────────────────────────────────────────────

/// Reject cycles that contain no `input` node — such a loop can never make
/// progress (spec §1, §13). A cycle through an `input` node is fine: the user
/// can supply fresh input each lap.
fn check_cycles(flow: &Flow, errors: &mut Vec<String>) {
    let sccs = tarjan_scc(flow);
    for scc in &sccs {
        let has_input = scc
            .iter()
            .any(|id| matches!(flow.nodes.get(id), Some(Node::Input(_))));
        let non_trivial = scc.len() > 1
            || scc.iter().any(|id| has_self_loop(flow, id));
        if non_trivial && !has_input {
            let mut members: Vec<&str> = scc.iter().map(String::as_str).collect();
            members.sort();
            errors.push(format!(
                "cycle without an input node (no progress possible): {}",
                members.join(", ")
            ));
        }
    }
}

fn successors(flow: &Flow, id: &str) -> Vec<String> {
    match flow.nodes.get(id) {
        Some(Node::Menu(m)) => m
            .options
            .values()
            .flat_map(|v| v.branches())
            .map(|b| b.goto.clone())
            .collect(),
        Some(Node::Input(i)) => vec![i.next.clone()],
        Some(Node::Action(a)) => vec![a.next.clone()],
        Some(Node::End(_)) | None => vec![],
    }
}

fn has_self_loop(flow: &Flow, id: &str) -> bool {
    successors(flow, id).iter().any(|next| next == id)
}

/// Tarjan's strongly-connected-components (iterative-free, graph is small).
fn tarjan_scc(flow: &Flow) -> Vec<Vec<String>> {
    let mut index = 0usize;
    let mut indices: HashMap<String, usize> = HashMap::new();
    let mut low: HashMap<String, usize> = HashMap::new();
    let mut on_stack: HashMap<String, bool> = HashMap::new();
    let mut stack: Vec<String> = Vec::new();
    let mut sccs: Vec<Vec<String>> = Vec::new();

    fn strongconnect(
        v: &str,
        flow: &Flow,
        index: &mut usize,
        indices: &mut HashMap<String, usize>,
        low: &mut HashMap<String, usize>,
        on_stack: &mut HashMap<String, bool>,
        stack: &mut Vec<String>,
        sccs: &mut Vec<Vec<String>>,
    ) {
        *index += 1;
        let v_index = *index;
        indices.insert(v.to_string(), v_index);
        low.insert(v.to_string(), v_index);
        stack.push(v.to_string());
        on_stack.insert(v.to_string(), true);

        for w in successors(flow, v) {
            if !indices.contains_key(&w) {
                strongconnect(&w, flow, index, indices, low, on_stack, stack, sccs);
                let w_low = *low.get(&w).unwrap();
                let v_low = *low.get(v).unwrap();
                low.insert(v.to_string(), v_low.min(w_low));
            } else if *on_stack.get(&w).unwrap_or(&false) {
                let w_index = *indices.get(&w).unwrap();
                let v_low = *low.get(v).unwrap();
                low.insert(v.to_string(), v_low.min(w_index));
            }
        }

        if *low.get(v).unwrap() == *indices.get(v).unwrap() {
            let mut component = Vec::new();
            loop {
                let w = stack.pop().unwrap();
                on_stack.insert(w.clone(), false);
                component.push(w.clone());
                if w == v {
                    break;
                }
            }
            sccs.push(component);
        }
    }

    for id in flow.nodes.keys() {
        if !indices.contains_key(id) {
            strongconnect(id, flow, &mut index, &mut indices, &mut low, &mut on_stack, &mut stack, &mut sccs);
        }
    }
    sccs
}

// ── Tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::load_flow;
    use serde_json::json;

    /// Build a tiny flow programmatically for negative tests.
    fn mini_flow(start: &str, nodes: Vec<(&str, Node)>) -> Flow {
        Flow {
            id: "mini".into(),
            name: "mini".into(),
            description: None,
            version: 1,
            start: start.into(),
            timeouts: crate::schema::Timeouts { session: 120, step: 20 },
            variables: vec![],
            webhooks: None,
            nodes: nodes.into_iter().map(|(k, v)| (k.to_string(), v)).collect(),
        }
    }

    fn menu(text: &str, options: Vec<(&str, &str)>) -> Node {
        Node::Menu(crate::schema::MenuNode {
            text: text.into(),
            options: options
                .into_iter()
                .map(|(k, goto)| {
                    (
                        k.to_string(),
                        crate::schema::OptionValue::One(crate::schema::Branch {
                            when: vec![],
                            set: Default::default(),
                            compute: Default::default(),
                            goto: goto.to_string(),
                        }),
                    )
                })
                .collect(),
            on_invalid: None,
            on_timeout: None,
        })
    }

    fn end(text: &str) -> Node {
        Node::End(crate::schema::EndNode { text: text.into(), payments: None })
    }

    fn input_node(variable: &str, next: &str) -> Node {
        Node::Input(crate::schema::InputNode {
            prompt: "?".into(),
            variable: variable.into(),
            validate: None,
            on_invalid: None,
            on_timeout: None,
            next: next.into(),
        })
    }

    #[test]
    fn embedded_demo_is_valid() {
        let flow = load_flow(None).expect("embedded demo must validate");
        assert_eq!(flow.id, "farmer-order");
        assert_eq!(flow.start, "welcome");
    }

    #[test]
    fn dangling_target_rejected() {
        let flow = mini_flow(
            "a",
            vec![("a", menu("x", vec![("1", "missing")]))],
        );
        let err = validate_flow(&flow).unwrap_err();
        assert!(err.iter().any(|e| e.contains("missing")));
    }

    #[test]
    fn oversized_screen_rejected() {
        let long = "x".repeat(200);
        let flow = mini_flow("a", vec![("a", menu(&long, vec![("1", "b")])), ("b", end("ok"))]);
        assert!(validate_flow(&flow).is_err());
    }

    #[test]
    fn timeout_out_of_bounds_rejected() {
        let mut flow = mini_flow(
            "a",
            vec![("a", menu("x", vec![("1", "b")])), ("b", end("ok"))],
        );
        flow.timeouts.session = 200;
        assert!(validate_flow(&flow).is_err());
        flow.timeouts.session = 120;
        flow.timeouts.step = 61;
        assert!(validate_flow(&flow).is_err());
    }

    #[test]
    fn menu_self_loop_without_input_rejected() {
        let flow = mini_flow("a", vec![("a", menu("x", vec![("1", "a")]))]);
        let err = validate_flow(&flow).unwrap_err();
        assert!(err.iter().any(|e| e.contains("cycle")));
    }

    #[test]
    fn menu_action_loop_without_input_rejected() {
        let action = Node::Action(crate::schema::ActionNode {
            set: Default::default(),
            compute: Default::default(),
            next: "a".into(),
        });
        let flow = mini_flow(
            "a",
            vec![("a", menu("x", vec![("1", "b")])), ("b", action)],
        );
        assert!(validate_flow(&flow).is_err());
    }

    #[test]
    fn cycle_through_input_is_allowed() {
        // a → input → b → a is fine: the user can enter data each lap.
        let flow = mini_flow(
            "a",
            vec![
                ("a", menu("x", vec![("1", "inp")])),
                ("inp", input_node("n", "b")),
                ("b", menu("y", vec![("1", "a")])),
            ],
        );
        assert!(validate_flow(&flow).is_ok());
    }

    #[test]
    fn invalid_expression_rejected() {
        let action = Node::Action(crate::schema::ActionNode {
            set: Default::default(),
            compute: [("total".to_string(), "1 +".to_string())].into_iter().collect(),
            next: "b".into(),
        });
        let flow = mini_flow(
            "a",
            vec![("a", action), ("b", end("ok"))],
        );
        assert!(validate_flow(&flow).is_err());
    }

    #[test]
    fn option_validation_requires_options_list() {
        let input = Node::Input(crate::schema::InputNode {
            prompt: "?".into(),
            variable: "x".into(),
            validate: Some(crate::schema::Validation {
                kind: ValidationKind::Option,
                min: None,
                max: None,
                min_length: None,
                max_length: None,
                pattern: None,
                options: vec![],
                message: None,
            }),
            on_invalid: None,
            on_timeout: None,
            next: "b".into(),
        });
        let flow = mini_flow("a", vec![("a", input), ("b", end("ok"))]);
        assert!(validate_flow(&flow).is_err());
    }

    #[test]
    fn json_variant_of_demo_flow() {
        // docs/examples/airtime.json must parse (JSON path) and validate.
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../../docs/examples/airtime.json"
        );
        let flow = load_flow(Some(path)).expect("airtime.json must load");
        assert_eq!(flow.id, "airtime");
    }

    #[test]
    fn yaml_flow_without_timeouts_uses_defaults() {
        // balance-check.yaml omits the whole `timeouts` block; the loader must
        // apply the 120s/20s defaults instead of failing validation.
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../../docs/examples/balance-check.yaml"
        );
        let flow = load_flow(Some(path)).expect("balance-check.yaml must load");
        assert_eq!(flow.timeouts.session, 120);
        assert_eq!(flow.timeouts.step, 20);
    }

    #[test]
    fn json_value_helpers() {
        use crate::schema::{loose_eq, value_display};
        assert!(loose_eq(&json!(5), &json!("5")));
        assert!(!loose_eq(&json!("5"), &json!("6")));
        assert_eq!(value_display(&json!(7000.0)), "7000");
        assert_eq!(value_display(&json!(2.5)), "2.5");
    }
}
