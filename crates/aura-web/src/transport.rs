//! The variable boundary is DNS/HTTP, not parsing, budgeting or provenance.
use crate::WebError;
use futures::{Stream, StreamExt, future::BoxFuture, stream::BoxStream};
use std::{
    collections::BTreeMap,
    net::{IpAddr, SocketAddr},
    time::Duration,
};
use url::Url;

#[derive(Debug, Clone)]
pub struct HttpRequest {
    pub url: Url,
    pub method: String,
    pub headers: BTreeMap<String, String>,
    pub body: Vec<u8>,
    pub addresses: Vec<IpAddr>,
}
pub struct HttpResponse {
    pub status: u16,
    pub headers: BTreeMap<String, String>,
    pub body: HttpBody,
}
pub struct HttpBody(BoxStream<'static, Result<Vec<u8>, WebError>>);
impl HttpBody {
    pub fn new(stream: impl Stream<Item = Result<Vec<u8>, WebError>> + Send + 'static) -> Self {
        Self(stream.boxed())
    }
    pub fn into_stream(self) -> BoxStream<'static, Result<Vec<u8>, WebError>> {
        self.0
    }
}
impl From<Vec<u8>> for HttpBody {
    fn from(bytes: Vec<u8>) -> Self {
        Self::new(futures::stream::once(async move { Ok(bytes) }))
    }
}
pub trait Transport: Send + Sync {
    fn resolve<'a>(&'a self, host: &'a str) -> BoxFuture<'a, Result<Vec<IpAddr>, WebError>>;
    fn request(&self, request: HttpRequest) -> BoxFuture<'_, Result<HttpResponse, WebError>>;
}

/// One isolated client per destination: no ambient proxy, cookies, redirect,
/// retry or credentials. DNS-approved addresses keep the original TLS/SNI host.
pub struct ReqwestTransport;
impl Transport for ReqwestTransport {
    fn resolve<'a>(&'a self, host: &'a str) -> BoxFuture<'a, Result<Vec<IpAddr>, WebError>> {
        Box::pin(async move {
            tokio::net::lookup_host((host, 443))
                .await
                .map(|addresses| addresses.map(|address| address.ip()).collect())
                .map_err(|_| WebError::Unavailable)
        })
    }
    fn request(&self, request: HttpRequest) -> BoxFuture<'_, Result<HttpResponse, WebError>> {
        Box::pin(async move {
            crate::search::normalize_url(request.url.as_str())?;
            let host = request.url.host_str().ok_or(WebError::UnsafeUrl)?;
            let port = request
                .url
                .port_or_known_default()
                .ok_or(WebError::UnsafeUrl)?;
            let addresses: Vec<_> = request
                .addresses
                .iter()
                .map(|ip| SocketAddr::new(*ip, port))
                .collect();
            if addresses.is_empty()
                || request
                    .addresses
                    .iter()
                    .any(|ip| !crate::search::public_ip(*ip))
            {
                return Err(WebError::UnsafeUrl);
            }
            let client = reqwest::Client::builder()
                .no_proxy()
                .redirect(reqwest::redirect::Policy::none())
                .retry(reqwest::retry::never())
                .referer(false)
                .tls_sslkeylogfile(false)
                .gzip(true)
                .brotli(true)
                .deflate(true)
                .user_agent(concat!("Aura/", env!("CARGO_PKG_VERSION")))
                .connect_timeout(Duration::from_secs(5))
                .timeout(Duration::from_secs(30))
                .resolve_to_addrs(host, &addresses)
                .build()
                .map_err(|_| WebError::Unavailable)?;
            let method = reqwest::Method::from_bytes(request.method.as_bytes())
                .map_err(|_| WebError::InvalidInput)?;
            let mut builder = client.request(method, request.url);
            for (name, value) in request.headers {
                builder = builder.header(name, value);
            }
            let response = builder
                .body(request.body)
                .send()
                .await
                .map_err(network_error)?;
            if response
                .remote_addr()
                .is_none_or(|peer| !addresses.contains(&peer))
            {
                return Err(WebError::UnsafeUrl);
            }
            let status = response.status().as_u16();
            let headers: BTreeMap<_, _> = response
                .headers()
                .iter()
                .filter_map(|(name, value)| {
                    value
                        .to_str()
                        .ok()
                        .map(|value| (name.as_str().into(), value.into()))
                })
                .collect();
            if response
                .content_length()
                .is_some_and(|len| len > 2 * 1024 * 1024)
            {
                return Err(WebError::TooLarge);
            }
            if headers
                .get("content-encoding")
                .is_some_and(|encoding: &String| !encoding.eq_ignore_ascii_case("identity"))
            {
                return Err(WebError::UnsupportedContent);
            }
            let body = HttpBody::new(
                response
                    .bytes_stream()
                    .map(|chunk| chunk.map(|chunk| chunk.to_vec()).map_err(network_error)),
            );
            Ok(HttpResponse {
                status,
                headers,
                body,
            })
        })
    }
}
fn network_error(error: reqwest::Error) -> WebError {
    if error.is_timeout() {
        WebError::Timeout
    } else {
        WebError::Unavailable
    }
}
