//! Maps Codex/upstream failures to [`TurnError`] categories (AC-008, 002).
//!
//! Inputs come in two shapes: the `error` object of the `error` notification
//! or of a failed `turn` (`{message, codexErrorInfo, additionalDetails}`), where
//! `codexErrorInfo` is either a string (`"usageLimitExceeded"`) or an object
//! (`{"httpConnectionFailed": {"httpStatusCode": 429}}`). Upstream Responses
//! error codes (`subscription_sharing_*`) travel inside `message` or
//! `additionalDetails` and take precedence.

use crate::events::TurnError;
use serde_json::Value;

pub fn turn_error_from(error: &Value) -> TurnError {
    let message = error
        .get("message")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();
    let details = error
        .get("additionalDetails")
        .map(stringify)
        .unwrap_or_default();
    let haystack = format!("{message} {details}");

    if haystack.contains("subscription_sharing_usage_limit_exceeded") {
        return TurnError::PlanUsageLimit;
    }
    if haystack.contains("subscription_sharing_user_not_eligible") {
        return TurnError::PlanNotEligible;
    }
    if haystack.contains("subscription_sharing_unsupported_capability") {
        return TurnError::UnsupportedCapability {
            param: extract_param(&haystack),
        };
    }
    if haystack.contains("subscription_sharing_invalid_user") {
        return TurnError::SessionExpired;
    }

    let (variant, status) = match error.get("codexErrorInfo") {
        Some(Value::String(s)) => (s.clone(), None),
        Some(Value::Object(map)) => match map.iter().next() {
            Some((k, v)) => (k.clone(), v.get("httpStatusCode").and_then(Value::as_u64)),
            None => (String::new(), None),
        },
        _ => (String::new(), None),
    };
    let variant_lc = variant.to_ascii_lowercase();
    match variant_lc.as_str() {
        "contextwindowexceeded" => return TurnError::ContextTooLong,
        "usagelimitexceeded" | "ratelimitexceeded" => {
            return TurnError::UsageLimit {
                retry_after_secs: extract_retry_after(&haystack),
            };
        }
        "unauthorized" => return TurnError::SessionExpired,
        "httpconnectionfailed"
        | "responsestreamconnectionfailed"
        | "responsestreamdisconnected" => {
            return match status {
                Some(401) => TurnError::SessionExpired,
                Some(429) => TurnError::UsageLimit {
                    retry_after_secs: extract_retry_after(&haystack),
                },
                Some(s) if (400..500).contains(&s) => TurnError::Other {
                    code: format!("http_{s}"),
                    message,
                },
                _ => TurnError::NoConnection,
            };
        }
        _ => {}
    }
    if haystack.contains("context_length_exceeded") {
        return TurnError::ContextTooLong;
    }
    TurnError::Other {
        code: if variant.is_empty() {
            "unknown".into()
        } else {
            variant
        },
        message,
    }
}

fn stringify(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        other => other.to_string(),
    }
}

fn extract_param(text: &str) -> Option<String> {
    let idx = text.find("\"param\"")?;
    let rest = &text[idx + 7..];
    let start = rest.find('"')? + 1;
    let end = rest[start..].find('"')? + start;
    Some(rest[start..end].to_string()).filter(|s| !s.is_empty())
}

fn extract_retry_after(text: &str) -> Option<u64> {
    let lower = text.to_ascii_lowercase();
    for key in ["retry_after", "retry-after", "retryafter"] {
        if let Some(i) = lower.find(key) {
            let digits: String = lower[i + key.len()..]
                .chars()
                .skip_while(|c| !c.is_ascii_digit())
                .take_while(|c| c.is_ascii_digit())
                .collect();
            if let Ok(n) = digits.parse() {
                return Some(n);
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn plan_usage_limit_from_upstream_body() {
        let e = json!({"message": "unexpected status 429: {\"error\":{\"code\":\"subscription_sharing_usage_limit_exceeded\"}}",
                       "codexErrorInfo": {"httpConnectionFailed": {"httpStatusCode": 429}}});
        assert_eq!(turn_error_from(&e), TurnError::PlanUsageLimit);
    }

    #[test]
    fn not_eligible_and_unsupported_capability() {
        assert_eq!(
            turn_error_from(&json!({"message": "403 subscription_sharing_user_not_eligible"})),
            TurnError::PlanNotEligible
        );
        let e = json!({"message": "400", "additionalDetails": {"error": {"code": "subscription_sharing_unsupported_capability", "param": "tools[3]"}}});
        assert_eq!(
            turn_error_from(&e),
            TurnError::UnsupportedCapability {
                param: Some("tools[3]".into())
            }
        );
    }

    #[test]
    fn codex_error_info_string_and_object_forms() {
        assert_eq!(
            turn_error_from(&json!({"codexErrorInfo": "contextWindowExceeded"})),
            TurnError::ContextTooLong
        );
        assert_eq!(
            turn_error_from(&json!({"codexErrorInfo": "unauthorized"})),
            TurnError::SessionExpired
        );
        assert_eq!(
            turn_error_from(
                &json!({"codexErrorInfo": {"httpConnectionFailed": {"httpStatusCode": null}}})
            ),
            TurnError::NoConnection
        );
        assert_eq!(
            turn_error_from(
                &json!({"message": "retry_after: 20", "codexErrorInfo": "usageLimitExceeded"})
            ),
            TurnError::UsageLimit {
                retry_after_secs: Some(20)
            }
        );
    }

    #[test]
    fn unknown_errors_keep_code_and_message() {
        assert_eq!(
            turn_error_from(&json!({"message": "boom", "codexErrorInfo": "internalServerError"})),
            TurnError::Other {
                code: "internalServerError".into(),
                message: "boom".into()
            }
        );
    }
}
