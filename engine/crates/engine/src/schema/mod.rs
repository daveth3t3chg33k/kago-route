//! Menu-schema DSL: types, loading, validation, and the session walker.
//!
//! The types below mirror `docs/ussd-menu-schema.md` (Appendix B) and
//! `docs/menu-schema.schema.json`. The engine parses JSON or YAML, validates
//! the flow fail-closed at boot, then replays the cumulative carrier `text`
//! through the graph on every callback.

pub mod demo;
pub mod expr;
pub mod input;
pub mod validate;
pub mod walk;

use std::collections::HashMap;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// The DSL identifier every document must declare.
pub const DSL_IDENTIFIER: &str = "kagoroute/menu/1.0";

// ── Document ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FlowDocument {
    pub schema: String,
    pub flow: Flow,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Flow {
    pub id: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub version: u32,
    pub start: String,
    #[serde(default)]
    pub timeouts: Timeouts,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub variables: Vec<VariableDecl>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub webhooks: Option<Webhooks>,
    pub nodes: HashMap<String, Node>,
}

// ── Flow-level metadata ──────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Timeouts {
    #[serde(default = "default_session_timeout")]
    pub session: u32,
    #[serde(default = "default_step_timeout")]
    pub step: u32,
}

impl Default for Timeouts {
    /// `#[serde(default)]` on `Flow.timeouts` uses this when the whole block
    /// is omitted — must yield valid defaults (120s session, 20s step), not 0s.
    fn default() -> Self {
        Self {
            session: default_session_timeout(),
            step: default_step_timeout(),
        }
    }
}

fn default_session_timeout() -> u32 {
    120
}

fn default_step_timeout() -> u32 {
    20
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VariableDecl {
    pub name: String,
    #[serde(default, rename = "type", skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Webhooks {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub on_complete: Option<Webhook>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Webhook {
    pub url: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub secret: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub events: Vec<String>,
}

// ── Nodes ────────────────────────────────────────────────────────────────

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
    pub options: HashMap<String, OptionValue>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub on_invalid: Option<Recovery>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub on_timeout: Option<Recovery>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InputNode {
    pub prompt: String,
    pub variable: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub validate: Option<Validation>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub on_invalid: Option<Recovery>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub on_timeout: Option<Recovery>,
    pub next: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ActionNode {
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub set: HashMap<String, Value>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub compute: HashMap<String, String>,
    pub next: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EndNode {
    pub text: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub payments: Option<Payments>,
}

// ── Branching / conditions ───────────────────────────────────────────────

/// A menu option value: a single branch or an ordered list of branches
/// (for `when` chains). Normalized to `Vec<Branch>` at parse time.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum OptionValue {
    One(Branch),
    Many(Vec<Branch>),
}

impl OptionValue {
    pub fn branches(&self) -> Vec<&Branch> {
        match self {
            OptionValue::One(b) => vec![b],
            OptionValue::Many(list) => list.iter().collect(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Branch {
    /// `when` accepts a single condition or a list; normalized to `Vec`.
    #[serde(default, deserialize_with = "deserialize_when", skip_serializing_if = "Vec::is_empty")]
    pub when: Vec<Condition>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub set: HashMap<String, Value>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub compute: HashMap<String, String>,
    pub goto: String,
}

fn deserialize_when<'de, D>(deserializer: D) -> Result<Vec<Condition>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum When {
        One(Condition),
        Many(Vec<Condition>),
    }

    let when = Option::<When>::deserialize(deserializer)?;
    Ok(match when {
        Some(When::One(cond)) => vec![cond],
        Some(When::Many(list)) => list,
        None => vec![],
    })
}


#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Condition {
    pub var: String,
    pub op: Op,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value: Option<Value>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Op {
    Eq,
    Neq,
    Gt,
    Gte,
    Lt,
    Lte,
    Contains,
    StartsWith,
    Matches,
    IsSet,
    In,
}

// ── Validation ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Validation {
    #[serde(rename = "type")]
    pub kind: ValidationKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min_length: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_length: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pattern: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub options: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ValidationKind {
    Int,
    Float,
    Text,
    Phone,
    Amount,
    Option,
}

// ── Recovery ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Recovery {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    pub goto: String,
}

// ── Payments (parsed; firing is a later milestone) ───────────────────────

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Payments {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mpesa: Option<MpesaStkPush>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MpesaStkPush {
    pub short_code: String,
    pub amount_expr: String,
    pub phone_expr: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub account_ref: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transaction_desc: Option<String>,
}

// ── Loading ──────────────────────────────────────────────────────────────

/// Load a flow from a file path (`.json` or YAML) or the embedded demo.
/// Parses, checks the DSL identifier, and validates fail-closed.
pub fn load_flow(path: Option<&str>) -> Result<Arc<Flow>, String> {
    let (raw, label) = match path {
        Some(p) => (
            std::fs::read_to_string(p).map_err(|e| format!("cannot read schema file '{p}': {e}"))?,
            p.to_string(),
        ),
        None => (demo::FARMER_ORDER.to_string(), "<embedded demo>".to_string()),
    };

    let use_json = path.is_some_and(|p| p.to_ascii_lowercase().ends_with(".json"));
    let doc: FlowDocument = if use_json {
        serde_json::from_str(&raw)
            .map_err(|e| format!("failed to parse schema {label}: {e}"))?
    } else {
        serde_yaml::from_str(&raw)
            .map_err(|e| format!("failed to parse schema {label}: {e}"))?
    };

    if doc.schema != DSL_IDENTIFIER {
        return Err(format!(
            "unsupported DSL '{0}' in {label} (expected '{1}')",
            doc.schema, DSL_IDENTIFIER
        ));
    }

    validate::validate_flow(&doc.flow)
        .map_err(|errors| format!("schema {label} failed validation: {}", errors.join("; ")))?;

    Ok(Arc::new(doc.flow))
}

/// Value → display string used in interpolation.
pub fn value_display(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                i.to_string()
            } else if let Some(f) = n.as_f64() {
                if f.fract() == 0.0 {
                    format!("{}", f as i64)
                } else {
                    format!("{f}")
                }
            } else {
                n.to_string()
            }
        }
        Value::Bool(b) => b.to_string(),
        _ => String::new(),
    }
}

/// Value → numeric form used in comparisons and expressions.
pub fn value_to_f64(v: &Value) -> f64 {
    match v {
        Value::Number(n) => n.as_f64().unwrap_or(0.0),
        Value::String(s) => s.trim().parse::<f64>().unwrap_or(0.0),
        Value::Bool(b) => {
            if *b {
                1.0
            } else {
                0.0
            }
        }
        _ => 0.0,
    }
}

/// Loose equality ("5" == 5, 5 == 5.0).
pub fn loose_eq(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Number(x), Value::Number(y)) => x.as_f64() == y.as_f64(),
        (Value::String(x), Value::String(y)) => x == y,
        (Value::Number(_), Value::String(s)) => a.as_f64() == s.trim().parse::<f64>().ok(),
        (Value::String(s), Value::Number(_)) => s.trim().parse::<f64>().ok() == b.as_f64(),
        (Value::Bool(x), Value::Bool(y)) => x == y,
        _ => false,
    }
}
