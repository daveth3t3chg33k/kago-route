//! Input validation per spec §7, with normalization (phone → E.164,
//! numeric kinds → JSON numbers).

use serde_json::{Number, Value};

use crate::schema::{Validation, ValidationKind};

/// Validate one raw segment against a `Validation` block.
///
/// Returns the normalized value on success, or `Err(message)` on failure
/// where `message` is the best available message (`message` override or the
/// kind's default).
pub fn validate_input(v: &Validation, raw: &str) -> Result<Value, String> {
    let fallback = |default: &str| v.message.clone().unwrap_or_else(|| default.to_string());

    match v.kind {
        ValidationKind::Int => {
            let trimmed = raw.trim();
            let Ok(n) = trimmed.parse::<i64>() else {
                return Err(fallback("Enter a whole number."));
            };
            if let Some(min) = v.min {
                if (n as f64) < min {
                    return Err(fallback("Enter a whole number."));
                }
            }
            if let Some(max) = v.max {
                if (n as f64) > max {
                    return Err(fallback("Enter a whole number."));
                }
            }
            Ok(Value::Number(Number::from(n)))
        }
        ValidationKind::Float => {
            let trimmed = raw.trim();
            let Ok(n) = trimmed.parse::<f64>() else {
                return Err(fallback("Enter a number."));
            };
            check_bounds(v, n, &fallback("Enter a number."))?;
            Ok(Value::Number(Number::from_f64(n).unwrap_or(Number::from(0))))
        }
        ValidationKind::Amount => {
            let trimmed = raw.trim();
            let Ok(n) = trimmed.parse::<f64>() else {
                return Err(fallback("Enter a valid amount."));
            };
            if n < 0.0 {
                return Err(fallback("Enter a valid amount."));
            }
            // No more than 2 decimal places.
            if (n * 100.0 - (n * 100.0).round()).abs() > 1e-6 {
                return Err(fallback("Enter a valid amount."));
            }
            check_bounds(v, n, &fallback("Enter a valid amount."))?;
            Ok(Value::Number(Number::from_f64(n).unwrap_or(Number::from(0))))
        }
        ValidationKind::Phone => match normalize_phone(raw) {
            Some(normalized) => Ok(Value::String(normalized)),
            None => Err(fallback("Enter a valid phone number.")),
        },
        ValidationKind::Text => {
            let trimmed = raw.trim().to_string();
            if let Some(min) = v.min_length {
                if trimmed.chars().count() < min {
                    return Err(fallback("Invalid input."));
                }
            }
            if let Some(max) = v.max_length {
                if trimmed.chars().count() > max {
                    return Err(fallback("Invalid input."));
                }
            }
            if let Some(pattern) = &v.pattern {
                let re = regex::Regex::new(pattern).map_err(|e| {
                    tracing::error!(pattern, "invalid validation regex: {e}");
                    fallback("Invalid input.")
                })?;
                if !re.is_match(&trimmed) {
                    return Err(fallback("Invalid input."));
                }
            }
            Ok(Value::String(trimmed))
        }
        ValidationKind::Option => {
            let trimmed = raw.trim().to_string();
            if v.options.iter().any(|o| o == &trimmed) {
                Ok(Value::String(trimmed))
            } else {
                Err(fallback("Choose a valid option."))
            }
        }
    }
}

fn check_bounds(v: &Validation, n: f64, msg: &str) -> Result<(), String> {
    if let Some(min) = v.min {
        if n < min {
            return Err(msg.to_string());
        }
    }
    if let Some(max) = v.max {
        if n > max {
            return Err(msg.to_string());
        }
    }
    Ok(())
}

/// Normalize a phone number to E.164-style digits.
/// - `+2547...` / `2547...` → kept as-is
/// - `07...` (10-digit Kenyan) → `2547...`
/// - otherwise: accepted as digits if 9..=15 long
pub fn normalize_phone(raw: &str) -> Option<String> {
    let trimmed = raw.trim().trim_start_matches('+');
    if trimmed.is_empty() || !trimmed.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    let len = trimmed.len();
    if !(9..=15).contains(&len) {
        return None;
    }
    if trimmed.starts_with('0') && len == 10 {
        return Some(format!("254{}", &trimmed[1..]));
    }
    Some(trimmed.to_string())
}

// ── Tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn v(kind: ValidationKind) -> Validation {
        Validation {
            kind,
            min: None,
            max: None,
            min_length: None,
            max_length: None,
            pattern: None,
            options: vec![],
            message: None,
        }
    }

    #[test]
    fn int_validation_and_bounds() {
        let mut val = v(ValidationKind::Int);
        assert_eq!(validate_input(&val, "42").unwrap(), json!(42));
        assert!(validate_input(&val, "4.2").is_err());
        assert!(validate_input(&val, "abc").is_err());
        val.min = Some(1.0);
        val.max = Some(50.0);
        assert!(validate_input(&val, "0").is_err());
        assert!(validate_input(&val, "51").is_err());
        assert!(validate_input(&val, "50").is_ok());
    }

    #[test]
    fn amount_two_decimals() {
        let val = v(ValidationKind::Amount);
        assert_eq!(validate_input(&val, "500").unwrap(), json!(500.0));
        assert_eq!(validate_input(&val, "499.99").unwrap(), json!(499.99));
        assert!(validate_input(&val, "1.234").is_err());
        assert!(validate_input(&val, "-5").is_err());
    }

    #[test]
    fn phone_normalization() {
        let val = v(ValidationKind::Phone);
        assert_eq!(validate_input(&val, "0712345678").unwrap(), json!("254712345678"));
        assert_eq!(validate_input(&val, "+254712345678").unwrap(), json!("254712345678"));
        assert_eq!(validate_input(&val, "254712345678").unwrap(), json!("254712345678"));
        assert!(validate_input(&val, "123").is_err());
        assert!(validate_input(&val, "2547abc").is_err());
    }

    #[test]
    fn text_lengths_and_pattern() {
        let mut val = v(ValidationKind::Text);
        val.min_length = Some(3);
        assert!(validate_input(&val, "ab").is_err());
        assert!(validate_input(&val, "abc").is_ok());
        val.pattern = Some("^[A-Z]{3}$".to_string());
        assert!(validate_input(&val, "abc").is_err());
        assert!(validate_input(&val, "ABC").is_ok());
    }

    #[test]
    fn option_membership() {
        let mut val = v(ValidationKind::Option);
        val.options = vec!["a".to_string(), "b".to_string()];
        assert!(validate_input(&val, "a").is_ok());
        assert!(validate_input(&val, "c").is_err());
    }

    #[test]
    fn custom_message_wins() {
        let mut val = v(ValidationKind::Int);
        val.message = Some("Custom error.".to_string());
        assert_eq!(
            validate_input(&val, "x").unwrap_err(),
            "Custom error."
        );
    }
}
