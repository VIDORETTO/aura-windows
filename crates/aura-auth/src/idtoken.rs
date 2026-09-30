//! ID token validation against the published JWKS.

use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode, decode_header};
use serde::Deserialize;
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct IdClaims {
    pub sub: String,
    pub email: Option<String>,
    pub nonce: Option<String>,
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum IdTokenError {
    #[error("unknown signing key")]
    UnknownKey,
    #[error("invalid token: {0}")]
    Invalid(String),
    #[error("nonce mismatch")]
    Nonce,
}

pub fn validate(
    id_token: &str,
    jwks: &Value,
    issuer: &str,
    client_id: &str,
    expected_nonce: &str,
) -> Result<IdClaims, IdTokenError> {
    let header = decode_header(id_token).map_err(|e| IdTokenError::Invalid(e.to_string()))?;
    let keys = jwks["keys"].as_array().ok_or(IdTokenError::UnknownKey)?;
    let key = keys
        .iter()
        .find(|k| {
            header
                .kid
                .as_deref()
                .is_none_or(|kid| k["kid"].as_str() == Some(kid))
        })
        .ok_or(IdTokenError::UnknownKey)?;
    let (n, e) = (key["n"].as_str(), key["e"].as_str());
    let decoding = match (n, e) {
        (Some(n), Some(e)) => DecodingKey::from_rsa_components(n, e)
            .map_err(|e| IdTokenError::Invalid(e.to_string()))?,
        _ => return Err(IdTokenError::UnknownKey),
    };
    let alg = match header.alg {
        Algorithm::RS256 | Algorithm::RS384 | Algorithm::RS512 | Algorithm::PS256 => header.alg,
        other => return Err(IdTokenError::Invalid(format!("unsupported alg {other:?}"))),
    };
    let mut validation = Validation::new(alg);
    validation.set_issuer(&[issuer]);
    validation.set_audience(&[client_id]);
    validation.leeway = 60;
    let data = decode::<IdClaims>(id_token, &decoding, &validation)
        .map_err(|e| IdTokenError::Invalid(e.to_string()))?;
    if data.claims.nonce.as_deref() != Some(expected_nonce) {
        return Err(IdTokenError::Nonce);
    }
    Ok(data.claims)
}
