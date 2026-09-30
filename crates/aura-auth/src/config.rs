//! Endpoints and constants of the SIWC flow. Tests point them at a local mock.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SiwcConfig {
    pub authorize_url: String,
    pub token_url: String,
    pub revocation_url: String,
    pub jwks_url: String,
    pub issuer: String,
    pub resource: String,
    pub scopes: Vec<String>,
    /// Display name suggested at first registration (`agent_name_hint`).
    pub agent_name: String,
    /// Preferred loopback port; any free port is used when busy.
    pub preferred_port: u16,
    /// Seconds before expiry when the access token is refreshed.
    pub refresh_margin_secs: i64,
}

impl Default for SiwcConfig {
    fn default() -> Self {
        Self {
            authorize_url: "https://auth.openai.com/api/accounts/authorize".into(),
            token_url: "https://auth.openai.com/api/accounts/oauth/token".into(),
            // From https://auth.openai.com/.well-known/openid-configuration;
            // re-read the discovery document if OpenAI changes it.
            revocation_url: "https://auth.openai.com/oauth/revoke".into(),
            jwks_url: "https://auth.openai.com/.well-known/jwks.json".into(),
            issuer: "https://auth.openai.com".into(),
            resource: "https://api.openai.com/v1".into(),
            scopes: [
                "openid",
                "profile",
                "email",
                "offline_access",
                "resource.invoke",
                crate::PLAN_SCOPE,
            ]
            .into_iter()
            .map(String::from)
            .collect(),
            agent_name: "Aura".into(),
            preferred_port: 1455,
            refresh_margin_secs: 300,
        }
    }
}

impl SiwcConfig {
    /// Refreshes endpoints from the OpenID discovery document.
    pub async fn discover(mut self, http: &reqwest::Client) -> Self {
        let url = format!(
            "{}/.well-known/openid-configuration",
            self.issuer.trim_end_matches('/')
        );
        if let Ok(resp) = http.get(url).send().await
            && let Ok(doc) = resp.json::<serde_json::Value>().await
        {
            if let Some(v) = doc["revocation_endpoint"].as_str() {
                self.revocation_url = v.to_string();
            }
            if let Some(v) = doc["jwks_uri"].as_str() {
                self.jwks_url = v.to_string();
            }
        }
        self
    }
}
