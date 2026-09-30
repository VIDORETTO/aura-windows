//! Upstream failures rendered as Responses-style HTTP errors so the Codex
//! client maps them to its own categories (AC-011 of 003).

use axum::http::{HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use serde_json::{Value, json};

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("{status} {code}: {message}")]
pub struct UpstreamError {
    pub status: u16,
    pub code: String,
    pub message: String,
    pub retry_after: Option<u64>,
}

impl UpstreamError {
    pub fn new(status: u16, code: &str, message: impl Into<String>) -> Self {
        Self {
            status,
            code: code.into(),
            message: message.into(),
            retry_after: None,
        }
    }

    pub fn unreachable(message: impl Into<String>) -> Self {
        Self::new(502, "upstream_unreachable", message)
    }

    /// Builds an error from an upstream HTTP status and body (any provider).
    pub fn from_http(status: u16, body: &str, retry_after: Option<u64>) -> Self {
        let v: Value = serde_json::from_str(body).unwrap_or(Value::Null);
        let err = if v["error"].is_object() {
            &v["error"]
        } else {
            &v
        };
        let upstream_code = err["code"]
            .as_str()
            .or(err["type"].as_str())
            .or(v["detail"].as_str().filter(|_| false))
            .map(str::to_string);
        let message = err["message"]
            .as_str()
            .or(v["detail"].as_str())
            .map(str::to_string)
            .unwrap_or_else(|| body.chars().take(500).collect());
        let code = match (status, upstream_code.as_deref()) {
            (_, Some(c)) if c.contains("context_length") || c.contains("context_window") => {
                "context_length_exceeded".to_string()
            }
            (_, Some(c)) if c.starts_with("subscription_sharing") || c.starts_with("chatpass") => {
                c.to_string()
            }
            (401, _) | (403, Some("authentication_error")) => "invalid_api_key".to_string(),
            (429, _) => "rate_limit_exceeded".to_string(),
            (529, _) | (503, _) => "server_overloaded".to_string(),
            (s, _) if s >= 500 => "server_error".to_string(),
            (_, Some(c)) => c.to_string(),
            (s, None) => format!("http_{s}"),
        };
        Self {
            status,
            code,
            message,
            retry_after,
        }
    }

    pub fn body(&self) -> Value {
        json!({"error": {"message": self.message, "type": self.code, "code": self.code, "param": null}})
    }
}

impl IntoResponse for UpstreamError {
    fn into_response(self) -> Response {
        let status = StatusCode::from_u16(self.status).unwrap_or(StatusCode::BAD_GATEWAY);
        let mut resp = (status, axum::Json(self.body())).into_response();
        if let Some(secs) = self.retry_after
            && let Ok(v) = HeaderValue::from_str(&secs.to_string())
        {
            resp.headers_mut().insert("retry-after", v);
        }
        resp
    }
}

pub fn retry_after(headers: &reqwest::header::HeaderMap) -> Option<u64> {
    headers
        .get("retry-after")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.trim().parse().ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn categories() {
        assert_eq!(
            UpstreamError::from_http(401, "{}", None).code,
            "invalid_api_key"
        );
        let e = UpstreamError::from_http(429, r#"{"error":{"message":"slow down"}}"#, Some(20));
        assert_eq!(
            (e.code.as_str(), e.retry_after),
            ("rate_limit_exceeded", Some(20))
        );
        assert_eq!(
            UpstreamError::from_http(503, "", None).code,
            "server_overloaded"
        );
        assert_eq!(
            UpstreamError::from_http(
                400,
                r#"{"error":{"code":"context_length_exceeded","message":"too long"}}"#,
                None
            )
            .code,
            "context_length_exceeded"
        );
        assert_eq!(
            UpstreamError::from_http(
                429,
                r#"{"error":{"code":"subscription_sharing_usage_limit_exceeded"}}"#,
                None
            )
            .code,
            "subscription_sharing_usage_limit_exceeded"
        );
        assert_eq!(
            UpstreamError::from_http(403, r#"{"detail":"region"}"#, None).message,
            "region"
        );
    }
}
