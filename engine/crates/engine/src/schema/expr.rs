//! Tiny expression engine per spec §9, plus interpolation (§8) and
//! condition evaluation (§6).
//!
//! Grammar: `expr := term (("+"|"-") term)*`, `term := factor (("*"|"/") factor)*`,
//! `factor := number | var | "(" expr ")"`. Division truncates toward zero.
//! Unknown variables evaluate as 0.0 (documented).

use std::collections::HashMap;

use serde_json::Value;

use crate::schema::{loose_eq, value_display, value_to_f64, Condition, Op};

/// Session-scoped evaluation context. System variables (`$...`) are resolved
/// here; user variables come from `vars`.
pub struct Context<'a> {
    pub vars: &'a HashMap<String, Value>,
    pub phone: &'a str,
    pub session_id: &'a str,
    pub service_code: &'a str,
    pub flow_id: &'a str,
    pub flow_version: u32,
    pub last_input: &'a str,
}

impl<'a> Context<'a> {
    pub fn new(
        vars: &'a HashMap<String, Value>,
        phone: &'a str,
        session_id: &'a str,
        service_code: &'a str,
        flow_id: &'a str,
        flow_version: u32,
        last_input: &'a str,
    ) -> Self {
        Self {
            vars,
            phone,
            session_id,
            service_code,
            flow_id,
            flow_version,
            last_input,
        }
    }

    pub fn get(&self, name: &str) -> Option<Value> {
        match name {
            "$phone" => Some(Value::String(self.phone.to_string())),
            "$sessionId" => Some(Value::String(self.session_id.to_string())),
            "$serviceCode" => Some(Value::String(self.service_code.to_string())),
            "$flowId" => Some(Value::String(self.flow_id.to_string())),
            "$flowVersion" => Some(Value::Number((self.flow_version as i64).into())),
            "$lastInput" => Some(Value::String(self.last_input.to_string())),
            _ => self.vars.get(name).cloned(),
        }
    }

}

// ── Expressions ──────────────────────────────────────────────────────────

/// Evaluate an expression. Unknown variables and division by zero evaluate to
/// 0.0; a syntax error also yields 0.0 (deploy-time validation catches syntax).
pub fn eval_expr(expr: &str, ctx: &Context) -> f64 {
    match Parser::new(expr, ctx).parse() {
        Ok(v) => v,
        Err(_) => {
            tracing::warn!(expr, "expression evaluation failed");
            0.0
        }
    }
}

/// Syntax-only check used by the deploy-time validator.
pub fn expr_syntax_ok(expr: &str) -> bool {
    let empty = HashMap::new();
    let ctx = Context::new(&empty, "", "", "", "", 0, "");
    Parser::new(expr, &ctx).parse().is_ok()
}

struct Parser<'a, 'c> {
    chars: std::iter::Peekable<std::str::Chars<'a>>,
    ctx: &'c Context<'a>,
}

impl<'a, 'c> Parser<'a, 'c> {
    fn new(expr: &'a str, ctx: &'c Context<'a>) -> Self {
        Self {
            chars: expr.chars().peekable(),
            ctx,
        }
    }

    fn parse(&mut self) -> Result<f64, String> {
        let value = self.parse_add()?;
        self.skip_ws();
        if self.chars.peek().is_some() {
            return Err("trailing characters".into());
        }
        Ok(value)
    }

    fn parse_add(&mut self) -> Result<f64, String> {
        let mut value = self.parse_mul()?;
        loop {
            self.skip_ws();
            match self.peek() {
                Some('+') => {
                    self.next();
                    value += self.parse_mul()?;
                }
                Some('-') => {
                    self.next();
                    value -= self.parse_mul()?;
                }
                _ => return Ok(value),
            }
        }
    }

    fn parse_mul(&mut self) -> Result<f64, String> {
        let mut value = self.parse_factor()?;
        loop {
            self.skip_ws();
            match self.peek() {
                Some('*') => {
                    self.next();
                    value *= self.parse_factor()?;
                }
                Some('/') => {
                    self.next();
                    let divisor = self.parse_factor()?;
                    // Integer semantics: truncate toward zero; guard div-by-zero.
                    value = if divisor == 0.0 { 0.0 } else { (value / divisor).trunc() };
                }
                _ => return Ok(value),
            }
        }
    }

    fn parse_factor(&mut self) -> Result<f64, String> {
        self.skip_ws();
        match self.peek() {
            Some('(') => {
                self.next();
                let value = self.parse_add()?;
                self.skip_ws();
                if self.peek() == Some(')') {
                    self.next();
                    Ok(value)
                } else {
                    Err("expected ')'".into())
                }
            }
            Some(c) if c.is_ascii_digit() || (c == '-' && self.peek_n(1).is_some_and(|n| n.is_ascii_digit())) => {
                self.parse_number()
            }
            Some(c) if c.is_ascii_alphabetic() || c == '$' || c == '_' => {
                let name = self.parse_ident();
                Ok(value_to_f64(&self.ctx.get(&name).unwrap_or(Value::Null)))
            }
            Some(c) => Err(format!("unexpected character '{c}'")),
            None => Err("unexpected end of expression".into()),
        }
    }

    fn parse_number(&mut self) -> Result<f64, String> {
        let mut raw = String::new();
        if self.peek() == Some('-') {
            raw.push('-');
            self.next();
        }
        while self.peek().is_some_and(|c| c.is_ascii_digit() || c == '.') {
            raw.push(self.next().unwrap());
        }
        raw.parse::<f64>()
            .map_err(|_| format!("invalid number '{raw}'"))
    }

    fn parse_ident(&mut self) -> String {
        let mut name = String::new();
        while self
            .peek()
            .is_some_and(|c| c.is_ascii_alphanumeric() || c == '_' || c == '$')
        {
            name.push(self.next().unwrap());
        }
        name
    }

    fn skip_ws(&mut self) {
        while self.peek().is_some_and(|c| c.is_whitespace()) {
            self.next();
        }
    }

    fn peek(&mut self) -> Option<char> {
        self.chars.peek().copied()
    }

    fn peek_n(&mut self, n: usize) -> Option<char> {
        self.chars.clone().nth(n)
    }

    fn next(&mut self) -> Option<char> {
        self.chars.next()
    }
}

// ── Interpolation ────────────────────────────────────────────────────────

/// Replace `{var}` in a template. `{{` escapes to a literal `{`. Missing
/// variables render as an empty string with a warning.
pub fn interpolate(template: &str, ctx: &Context) -> String {
    let mut out = String::new();
    let mut chars = template.chars().peekable();

    while let Some(c) = chars.next() {
        if c != '{' {
            out.push(c);
            continue;
        }
        // Escaped literal brace.
        if chars.peek() == Some(&'{') {
            chars.next();
            out.push('{');
            continue;
        }
        let mut name = String::new();
        let mut closed = false;
        while let Some(&next) = chars.peek() {
            if next == '}' {
                chars.next();
                closed = true;
                break;
            }
            name.push(next);
            chars.next();
        }
        if closed {
            match ctx.get(name.trim()) {
                Some(v) => out.push_str(&value_display(&v)),
                None => {
                    tracing::warn!(var = %name.trim(), "interpolation: variable not found");
                }
            }
        } else {
            // Unterminated brace: keep it literally.
            out.push('{');
            out.push_str(&name);
        }
    }
    out
}

/// Resolve a `set` literal: a bare `$var` reference resolves to the variable
/// value; anything else is interpolated as a template.
pub fn resolve_set_value(value: &Value, ctx: &Context) -> Value {
    match value {
        Value::String(s) => {
            let trimmed = s.trim();
            if trimmed.starts_with('$')
                && !trimmed.contains(' ')
                && ctx.get(trimmed).is_some()
            {
                ctx.get(trimmed).unwrap_or_else(|| Value::String(s.clone()))
            } else {
                Value::String(interpolate(s, ctx))
            }
        }
        other => other.clone(),
    }
}

// ── Conditions ───────────────────────────────────────────────────────────

/// Evaluate one condition against the context (spec §6).
pub fn eval_condition(cond: &Condition, ctx: &Context) -> bool {
    let value = cond.value.clone().unwrap_or(Value::Null);
    match cond.op {
        Op::IsSet => ctx
            .get(&cond.var)
            .is_some_and(|v| !v.is_null() && !(v.is_string() && v.as_str().unwrap().is_empty())),
        Op::Eq => loose_eq(&ctx.get(&cond.var).unwrap_or(Value::Null), &value),
        Op::Neq => !loose_eq(&ctx.get(&cond.var).unwrap_or(Value::Null), &value),
        Op::Gt => {
            value_to_f64(&ctx.get(&cond.var).unwrap_or(Value::Null)) > value_to_f64(&value)
        }
        Op::Gte => {
            value_to_f64(&ctx.get(&cond.var).unwrap_or(Value::Null)) >= value_to_f64(&value)
        }
        Op::Lt => {
            value_to_f64(&ctx.get(&cond.var).unwrap_or(Value::Null)) < value_to_f64(&value)
        }
        Op::Lte => {
            value_to_f64(&ctx.get(&cond.var).unwrap_or(Value::Null)) <= value_to_f64(&value)
        }
        Op::Contains => {
            let hay = value_display(&ctx.get(&cond.var).unwrap_or(Value::Null));
            hay.contains(&value_display(&value))
        }
        Op::StartsWith => {
            let hay = value_display(&ctx.get(&cond.var).unwrap_or(Value::Null));
            hay.starts_with(&value_display(&value))
        }
        Op::Matches => match regex::Regex::new(&value_display(&value)) {
            Ok(re) => re.is_match(&value_display(&ctx.get(&cond.var).unwrap_or(Value::Null))),
            Err(_) => false,
        },
        Op::In => {
            let list = value.as_array().cloned().unwrap_or_default();
            let hay = ctx.get(&cond.var).unwrap_or(Value::Null);
            list.iter().any(|item| loose_eq(&hay, item))
        }
    }
}

/// Evaluate an ordered `when` list: conditions within one branch are ANDed;
/// branches are ORed in order (first match wins).
pub fn eval_conditions(when: &[Condition], ctx: &Context) -> bool {
    when.iter().all(|c| eval_condition(c, ctx))
}

// ── Tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn ctx(vars: &HashMap<String, Value>) -> Context<'_> {
        Context::new(vars, "254712345678", "s1", "*483*42#", "farmer-order", 4, "5")
    }

    fn vars(pairs: &[(&str, Value)]) -> HashMap<String, Value> {
        pairs.iter().map(|(k, v)| (k.to_string(), v.clone())).collect()
    }

    #[test]
    fn arithmetic_and_precedence() {
        let v = vars(&[("unitPrice", json!(3500)), ("qty", json!(2))]);
        let c = ctx(&v);
        assert_eq!(eval_expr("unitPrice * qty", &c), 7000.0);
        assert_eq!(eval_expr("1 + 2 * 3", &c), 7.0);
        assert_eq!(eval_expr("(1 + 2) * 3", &c), 9.0);
        assert_eq!(eval_expr("10 - 2 - 3", &c), 5.0);
    }

    #[test]
    fn division_truncates_toward_zero() {
        let v = HashMap::new();
        let c = ctx(&v);
        assert_eq!(eval_expr("7 / 2", &c), 3.0);
        assert_eq!(eval_expr("-7 / 2", &c), -3.0);
        assert_eq!(eval_expr("5 / 0", &c), 0.0); // guarded
    }

    #[test]
    fn unknown_variables_evaluate_to_zero() {
        let v = HashMap::new();
        let c = ctx(&v);
        assert_eq!(eval_expr("missing * 5", &c), 0.0);
    }

    #[test]
    fn interpolation_basics() {
        let v = vars(&[("qty", json!(2)), ("product", json!("maize-seed")), ("total", json!(7000))]);
        let c = ctx(&v);
        assert_eq!(
            interpolate("Order: {qty} x {product} = KES {total}", &c),
            "Order: 2 x maize-seed = KES 7000"
        );
        assert_eq!(interpolate("{{literal}} {missing}", &c), "{literal} ");
        assert_eq!(interpolate("phone {phone}", &c), "phone "); // user var, not system
        assert_eq!(interpolate("caller {$phone}", &c), "caller 254712345678");
    }

    #[test]
    fn set_value_resolution() {
        let v = HashMap::new();
        let c = ctx(&v);
        assert_eq!(
            resolve_set_value(&json!("$phone"), &c),
            json!("254712345678")
        );
        assert_eq!(
            resolve_set_value(&json!("ref-{qty}"), &ctx(&vars(&[("qty", json!(3))]))),
            json!("ref-3")
        );
        assert_eq!(resolve_set_value(&json!(3500), &c), json!(3500));
    }

    #[test]
    fn conditions() {
        let v = vars(&[("total", json!(17500))]);
        let c = ctx(&v);
        let cond = |var: &str, op: Op, val: Value| Condition {
            var: var.to_string(),
            op,
            value: Some(val),
        };
        assert!(eval_condition(&cond("total", Op::Gte, json!(10000)), &c));
        assert!(!eval_condition(&cond("total", Op::Lt, json!(10000)), &c));
        // loose equality: string "17500" == number 17500
        let v2 = vars(&[("total", json!("17500"))]);
        assert!(eval_condition(&cond("total", Op::Eq, json!(17500)), &ctx(&v2)));
        // isSet
        let vars3 = vars(&[("a", json!("x")), ("b", json!(""))]);
        let c3 = ctx(&vars3);
        let set = |var: &str| Condition { var: var.to_string(), op: Op::IsSet, value: None };
        assert!(eval_condition(&set("a"), &c3));
        assert!(!eval_condition(&set("b"), &c3));
        assert!(!eval_condition(&set("nope"), &c3));
        // in
        assert!(eval_condition(
            &Condition { var: "product".into(), op: Op::In, value: Some(json!(["maize-seed", "fertilizer"])) },
            &ctx(&vars(&[("product", json!("fertilizer"))]))
        ));
        // matches
        assert!(eval_condition(
            &Condition { var: "phone".into(), op: Op::Matches, value: Some(json!("^254\\d{9}$")) },
            &ctx(&vars(&[("phone", json!("254712345678"))]))
        ));
    }

    #[test]
    fn syntax_check() {
        assert!(expr_syntax_ok("unitPrice * qty"));
        assert!(expr_syntax_ok("(a + b) / 2"));
        assert!(!expr_syntax_ok("(a + b"));
        assert!(!expr_syntax_ok("a @ b"));
    }
}
