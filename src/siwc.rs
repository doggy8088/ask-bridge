//! Sign in with ChatGPT (SIWC) backend — issue #40.
//!
//! An opt-in alternative transport for the ChatGPT provider that talks to
//! the public Responses API with an OAuth access token obtained through
//! OpenAI's official dynamic-registration flow for open-source tools, so
//! no API key is required. Usage draws on the account's ChatGPT plan
//! (Work/Codex usage or credits); it is not free or unlimited.
//!
//! The Browser/CDP path remains the default and is untouched. This module
//! owns: backend selection values, the credential store (one record per
//! issued client ID, kept in `~/.config/ask-bridge/siwc-credentials.json`
//! with owner-only permissions), the OAuth sign-in flow (loopback
//! callback, PKCE, state/nonce, JWKS-validated ID tokens), token refresh,
//! and the account model catalog. Inference lives in the same module
//! (added with the Responses transport).

use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use clap::ValueEnum;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::error::Error;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use super::config_file_path;

pub const AUTHORIZE_URL: &str = "https://auth.openai.com/api/accounts/authorize";
pub const TOKEN_URL: &str = "https://auth.openai.com/api/accounts/oauth/token";
pub const DISCOVERY_URL: &str = "https://auth.openai.com/.well-known/openid-configuration";
pub const API_BASE: &str = "https://api.openai.com/v1";
pub const RESOURCE: &str = "https://api.openai.com/v1";
pub const ISSUER: &str = "https://auth.openai.com";
pub const AGENT_NAME_HINT: &str = "ask-bridge";
pub const DYNAMIC_CLIENT_ID: &str = "dynamic_agent_client";
pub const PLAN_SCOPE: &str = "chatgpt.tokens.use.direct";
pub const SCOPES: &str =
    "openid profile email offline_access resource.invoke chatgpt.tokens.use.direct";
/// Preferred loopback callback port (the port used in OpenAI's examples);
/// an ephemeral port is used when it is unavailable.
pub const CALLBACK_PORT: u16 = 1455;
pub const CALLBACK_PATH: &str = "/auth/callback";
const LOGIN_TIMEOUT_SECS: u64 = 300;
/// Refresh the access token when it expires within this window.
const REFRESH_SKEW_SECS: i64 = 60;

// ---------------------------------------------------------------------------
// Backend selection
// ---------------------------------------------------------------------------

/// Execution backend for a provider. `Browser` (Chrome + CDP + MCP) is the
/// historical default; `Siwc` is the opt-in Sign in with ChatGPT API
/// backend, available for the ChatGPT provider only.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, ValueEnum)]
pub enum Backend {
    #[default]
    Browser,
    Siwc,
}

impl Backend {
    pub fn as_config_value(&self) -> &'static str {
        match self {
            Backend::Browser => "browser",
            Backend::Siwc => "siwc",
        }
    }

    pub fn from_config_value(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "browser" => Some(Backend::Browser),
            "siwc" => Some(Backend::Siwc),
            _ => None,
        }
    }
}

impl std::fmt::Display for Backend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_config_value())
    }
}

// ---------------------------------------------------------------------------
// Random values, PKCE, host identity
// ---------------------------------------------------------------------------

fn random_bytes(count: usize) -> Vec<u8> {
    let mut buf = vec![0u8; count];
    rand::thread_rng().fill_bytes(&mut buf);
    buf
}

fn base64url(bytes: &[u8]) -> String {
    URL_SAFE_NO_PAD.encode(bytes)
}

/// A fresh opaque token for OAuth `state` / `nonce` values.
pub fn random_token() -> String {
    base64url(&random_bytes(32))
}

/// PKCE S256 challenge for a verifier (RFC 7636).
pub fn pkce_challenge(verifier: &str) -> String {
    base64url(&Sha256::digest(verifier.as_bytes()))
}

pub fn new_pkce_pair() -> (String, String) {
    let verifier = base64url(&random_bytes(32));
    let challenge = pkce_challenge(&verifier);
    (verifier, challenge)
}

/// A stable per-host identifier (`urn:uuid:...`) generated once and then
/// persisted with the credential store, as the SIWC flow requires the same
/// `ext_agent_host_id` for every sign-in from this host.
pub fn generate_host_id() -> String {
    let mut bytes = random_bytes(16);
    bytes[6] = (bytes[6] & 0x0f) | 0x40; // version 4
    bytes[8] = (bytes[8] & 0x3f) | 0x80; // variant 1
    format!(
        "urn:uuid:{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
        bytes[0],
        bytes[1],
        bytes[2],
        bytes[3],
        bytes[4],
        bytes[5],
        bytes[6],
        bytes[7],
        bytes[8],
        bytes[9],
        bytes[10],
        bytes[11],
        bytes[12],
        bytes[13],
        bytes[14],
        bytes[15]
    )
}

// ---------------------------------------------------------------------------
// Credential store
// ---------------------------------------------------------------------------

/// One signed-in ChatGPT account, keyed by its issued client ID. Mirrors
/// the credential record shape in OpenAI's SIWC documentation.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SiwcAccount {
    pub client_id: String,
    pub email: Option<String>,
    pub subject: String,
    pub issuer: String,
    pub id_token: String,
    pub access_token: String,
    pub refresh_token: String,
    /// Unix seconds when the access token expires.
    pub expires_at: i64,
    pub scopes: Vec<String>,
    pub saved_at: String,
}

impl SiwcAccount {
    pub fn has_plan_scope(&self) -> bool {
        self.scopes.iter().any(|s| s == PLAN_SCOPE)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SiwcStore {
    pub host_id: String,
    pub active_client_id: Option<String>,
    #[serde(default)]
    pub accounts: Vec<SiwcAccount>,
}

impl Default for SiwcStore {
    fn default() -> Self {
        Self {
            host_id: generate_host_id(),
            active_client_id: None,
            accounts: Vec::new(),
        }
    }
}

impl SiwcStore {
    pub fn active_account(&self) -> Option<&SiwcAccount> {
        let active = self.active_client_id.as_deref()?;
        self.accounts.iter().find(|a| a.client_id == active)
    }

    pub fn active_account_mut(&mut self) -> Option<&mut SiwcAccount> {
        let active = self.active_client_id.clone()?;
        self.accounts.iter_mut().find(|a| a.client_id == active)
    }

    pub fn upsert_account(&mut self, account: SiwcAccount) {
        if let Some(existing) = self
            .accounts
            .iter_mut()
            .find(|a| a.client_id == account.client_id)
        {
            *existing = account.clone();
        } else {
            self.accounts.push(account.clone());
        }
        self.active_client_id = Some(account.client_id);
    }
}

pub fn credentials_path() -> Result<PathBuf, String> {
    let config_path = config_file_path()?;
    let dir = config_path
        .parent()
        .ok_or_else(|| "could not locate the ask-bridge config directory".to_string())?;
    Ok(dir.join("siwc-credentials.json"))
}

pub fn load_store() -> Result<SiwcStore, String> {
    let path = credentials_path()?;
    if !path.exists() {
        return Ok(SiwcStore::default());
    }
    let content = std::fs::read_to_string(&path)
        .map_err(|e| format!("failed to read {}: {}", path.to_string_lossy(), e))?;
    serde_json::from_str(&content)
        .map_err(|e| format!("failed to parse {}: {}", path.to_string_lossy(), e))
}

/// Write with owner-only permissions (0600 on Unix) via a temporary file
/// plus rename, so a crash never leaves a torn credential file behind and
/// tokens are never world-readable.
fn write_owner_only_atomic(path: &std::path::Path, content: &[u8]) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("failed to create {}: {}", parent.to_string_lossy(), e))?;
    }
    let tmp_path = path.with_extension(format!("json.tmp-{}", std::process::id()));
    {
        let mut options = std::fs::OpenOptions::new();
        options.write(true).create(true).truncate(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        let mut file = options
            .open(&tmp_path)
            .map_err(|e| format!("failed to write {}: {}", tmp_path.to_string_lossy(), e))?;
        use std::io::Write;
        file.write_all(content)
            .map_err(|e| format!("failed to write {}: {}", tmp_path.to_string_lossy(), e))?;
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(&tmp_path, std::fs::Permissions::from_mode(0o600));
    }
    std::fs::rename(&tmp_path, path).map_err(|e| {
        let _ = std::fs::remove_file(&tmp_path);
        format!("failed to store {}: {}", path.to_string_lossy(), e)
    })
}

pub fn save_store(store: &SiwcStore) -> Result<(), String> {
    let path = credentials_path()?;
    let content = serde_json::to_string_pretty(store)
        .map_err(|e| format!("failed to serialize credentials: {}", e))?;
    write_owner_only_atomic(&path, content.as_bytes())
}

/// Remove all stored SIWC accounts (sign out) while preserving the
/// stable `ext_agent_host_id`: the SIWC flow requires the same host ID
/// for every sign-in from this host, including after a sign-out.
pub fn clear_store() -> Result<bool, String> {
    let path = credentials_path()?;
    if !path.exists() {
        return Ok(false);
    }
    let mut store = load_store()?;
    let had_accounts = !store.accounts.is_empty() || store.active_client_id.is_some();
    store.accounts.clear();
    store.active_client_id = None;
    save_store(&store)?;
    Ok(had_accounts)
}

fn now_unix() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

fn now_rfc3339() -> String {
    // Sufficient for a saved-at record; not used for validation.
    format!("unix:{}", now_unix())
}

// ---------------------------------------------------------------------------
// Authorization request
// ---------------------------------------------------------------------------

pub struct AuthorizeRequest {
    pub client_id: String,
    /// True when this attempt performs first-time dynamic registration.
    pub dynamic_registration: bool,
    pub host_id: String,
    pub redirect_uri: String,
    pub state: String,
    pub nonce: String,
    pub code_challenge: String,
    pub id_token_hint: Option<String>,
    pub login_hint: Option<String>,
}

/// Build the authorize URL exactly as the SIWC flow specifies: the same
/// redirect URI must be reused verbatim for the code exchange.
pub fn build_authorize_url(req: &AuthorizeRequest) -> String {
    let mut params: Vec<(&str, String)> = vec![
        ("client_id", req.client_id.clone()),
        ("response_type", "code".to_string()),
        ("redirect_uri", req.redirect_uri.clone()),
        ("scope", SCOPES.to_string()),
        ("resource", RESOURCE.to_string()),
        ("state", req.state.clone()),
        ("nonce", req.nonce.clone()),
        ("code_challenge_method", "S256".to_string()),
        ("code_challenge", req.code_challenge.clone()),
        ("ext_agent_host_id", req.host_id.clone()),
    ];
    if req.dynamic_registration {
        params.push(("agent_name_hint", AGENT_NAME_HINT.to_string()));
    }
    if let Some(hint) = &req.id_token_hint {
        params.push(("id_token_hint", hint.clone()));
    }
    if let Some(hint) = &req.login_hint {
        params.push(("login_hint", hint.clone()));
    }
    let query = params
        .iter()
        .map(|(k, v)| {
            format!(
                "{}={}",
                k,
                url::form_urlencoded::byte_serialize(v.as_bytes()).collect::<String>()
            )
        })
        .collect::<Vec<_>>()
        .join("&");
    format!("{}?{}", AUTHORIZE_URL, query)
}

/// The authorize URL as printed for manual opening: identical to the
/// real URL except that `id_token_hint` (a full ID token) is removed, so
/// tokens never reach terminal scrollback or captured logs. The browser
/// still receives the complete URL.
pub fn redacted_authorize_url(url: &str) -> String {
    let Some((base, query)) = url.split_once('?') else {
        return url.to_string();
    };
    let kept: Vec<&str> = query
        .split('&')
        .filter(|pair| !pair.starts_with("id_token_hint="))
        .collect();
    format!("{}?{}", base, kept.join("&"))
}

/// Parsed loopback callback parameters.
#[derive(Debug, Default)]
pub struct CallbackParams {
    pub code: Option<String>,
    pub state: Option<String>,
    pub client_id: Option<String>,
    pub error: Option<String>,
}

pub fn parse_callback_query(query: &str) -> CallbackParams {
    let mut params = CallbackParams::default();
    for (key, value) in url::form_urlencoded::parse(query.as_bytes()) {
        match key.as_ref() {
            "code" => params.code = Some(value.into_owned()),
            "state" => params.state = Some(value.into_owned()),
            "client_id" => params.client_id = Some(value.into_owned()),
            "error" => params.error = Some(value.into_owned()),
            _ => {}
        }
    }
    params
}

// ---------------------------------------------------------------------------
// Token endpoint
// ---------------------------------------------------------------------------

fn format_http_error(context: &str, error: reqwest::Error) -> String {
    // OAuth URLs may contain sensitive query parameters.
    let error = error.without_url();
    let mut message = format!("{}: {}", context, error);
    let mut source = error.source();
    while let Some(cause) = source {
        message.push_str(": ");
        message.push_str(&cause.to_string());
        source = cause.source();
    }
    message
}

#[derive(Debug, Deserialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub id_token: Option<String>,
    pub expires_in: Option<i64>,
    pub scope: Option<String>,
}

impl TokenResponse {
    pub fn granted_scopes(&self) -> Vec<String> {
        self.scope
            .as_deref()
            .map(|s| s.split_whitespace().map(|p| p.to_string()).collect())
            .unwrap_or_default()
    }
}

async fn post_token_form(
    http: &reqwest::Client,
    fields: &[(&str, &str)],
) -> Result<TokenResponse, String> {
    let response = http
        .post(TOKEN_URL)
        .form(fields)
        .send()
        .await
        .map_err(|e| format_http_error("token request failed", e))?;
    let status = response.status();
    if !status.is_success() {
        let body = response.text().await.unwrap_or_default();
        let code = serde_json::from_str::<serde_json::Value>(&body)
            .ok()
            .and_then(|v| {
                v.get("error")
                    .and_then(|e| e.as_str())
                    .map(|s| s.to_string())
            })
            .unwrap_or_else(|| status.to_string());
        return Err(format!("token endpoint returned {}: {}", status, code));
    }
    response
        .json::<TokenResponse>()
        .await
        .map_err(|e| format!("invalid token response: {}", e))
}

pub fn token_needs_refresh(expires_at: i64, now: i64) -> bool {
    expires_at - now <= REFRESH_SKEW_SECS
}

// ---------------------------------------------------------------------------
// ID token validation (JWKS)
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct IdTokenClaims {
    pub sub: String,
    pub email: Option<String>,
    pub nonce: Option<String>,
    #[allow(dead_code)]
    pub exp: usize,
}

#[derive(Deserialize)]
struct Discovery {
    jwks_uri: String,
}

#[derive(Deserialize)]
struct Jwks {
    keys: Vec<Jwk>,
}

#[derive(Deserialize)]
struct Jwk {
    kid: Option<String>,
    n: Option<String>,
    e: Option<String>,
}

/// Validate an ID token's signature against OpenAI's published JWKS and
/// check issuer, audience, expiry (via the JWT validator) and nonce.
pub async fn validate_id_token(
    http: &reqwest::Client,
    token: &str,
    expected_client_id: &str,
    expected_nonce: &str,
) -> Result<IdTokenClaims, String> {
    let header = jsonwebtoken::decode_header(token)
        .map_err(|e| format!("invalid ID token header: {}", e))?;
    let kid = header
        .kid
        .ok_or_else(|| "ID token has no key id".to_string())?;
    let discovery: Discovery = http
        .get(DISCOVERY_URL)
        .send()
        .await
        .map_err(|e| format_http_error("failed to fetch OpenID configuration", e))?
        .json()
        .await
        .map_err(|e| format!("invalid OpenID configuration: {}", e))?;
    let jwks: Jwks = http
        .get(&discovery.jwks_uri)
        .send()
        .await
        .map_err(|e| format_http_error("failed to fetch JWKS", e))?
        .json()
        .await
        .map_err(|e| format!("invalid JWKS: {}", e))?;
    let key = jwks
        .keys
        .iter()
        .find(|k| k.kid.as_deref() == Some(kid.as_str()))
        .ok_or_else(|| format!("no JWKS key matches ID token kid {}", kid))?;
    let modulus = key
        .n
        .as_deref()
        .ok_or_else(|| "JWKS key is missing its modulus".to_string())?;
    let exponent = key
        .e
        .as_deref()
        .ok_or_else(|| "JWKS key is missing its exponent".to_string())?;
    let decoding_key = jsonwebtoken::DecodingKey::from_rsa_components(modulus, exponent)
        .map_err(|e| format!("invalid JWKS RSA key: {}", e))?;
    let mut validation = jsonwebtoken::Validation::new(jsonwebtoken::Algorithm::RS256);
    validation.set_issuer(&[ISSUER]);
    validation.set_audience(&[expected_client_id]);
    let data = jsonwebtoken::decode::<IdTokenClaims>(token, &decoding_key, &validation)
        .map_err(|e| format!("ID token validation failed: {}", e))?;
    if data.claims.nonce.as_deref() != Some(expected_nonce) {
        return Err("ID token nonce does not match this sign-in attempt".to_string());
    }
    Ok(data.claims)
}

// ---------------------------------------------------------------------------
// Model catalog
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModelInfo {
    pub slug: String,
    pub display_name: String,
}

/// Parse the account model catalog (`{models: [{slug, display_name,
/// visibility}]}`); entries intended for display (`visibility: "list"`,
/// or no visibility field) are kept in server order.
pub fn parse_models_catalog(value: &serde_json::Value) -> Vec<ModelInfo> {
    let Some(models) = value.get("models").and_then(|m| m.as_array()) else {
        return Vec::new();
    };
    models
        .iter()
        .filter_map(|entry| {
            let slug = entry.get("slug").and_then(|s| s.as_str())?;
            let visibility = entry.get("visibility").and_then(|v| v.as_str());
            if visibility.is_some_and(|v| v != "list") {
                return None;
            }
            let display_name = entry
                .get("display_name")
                .and_then(|d| d.as_str())
                .unwrap_or(slug)
                .to_string();
            Some(ModelInfo {
                slug: slug.to_string(),
                display_name,
            })
        })
        .collect()
}

pub async fn fetch_models(
    http: &reqwest::Client,
    access_token: &str,
) -> Result<Vec<ModelInfo>, String> {
    let response = http
        .get(format!("{}/models", API_BASE))
        .bearer_auth(access_token)
        .send()
        .await
        .map_err(|e| format_http_error("model catalog request failed", e))?;
    let status = response.status();
    if !status.is_success() {
        return Err(format!("model catalog request returned {}", status));
    }
    let value: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("invalid model catalog response: {}", e))?;
    Ok(parse_models_catalog(&value))
}

// ---------------------------------------------------------------------------
// Token refresh
// ---------------------------------------------------------------------------

/// Return a usable access token for the active account, refreshing (and
/// persisting the rotated tokens) when the stored one is near expiry.
pub async fn fresh_access_token(
    http: &reqwest::Client,
    store: &mut SiwcStore,
) -> Result<String, String> {
    let account = store
        .active_account()
        .ok_or_else(|| {
            "not signed in with ChatGPT; run `ask-bridge --provider chatgpt --backend siwc login` first"
                .to_string()
        })?
        .clone();
    if !token_needs_refresh(account.expires_at, now_unix()) {
        return Ok(account.access_token);
    }
    let token = post_token_form(
        http,
        &[
            ("grant_type", "refresh_token"),
            ("refresh_token", account.refresh_token.as_str()),
            ("client_id", account.client_id.as_str()),
            ("resource", RESOURCE),
        ],
    )
    .await
    .map_err(|e| {
        if e.contains("invalid_grant") {
            "SIWC session expired or was revoked; run `ask-bridge --provider chatgpt --backend siwc login` again"
                .to_string()
        } else {
            e
        }
    })?;
    let expires_at = now_unix() + token.expires_in.unwrap_or(3600);
    let granted = token.granted_scopes();
    if let Some(stored) = store.active_account_mut() {
        stored.access_token = token.access_token.clone();
        if let Some(refresh) = token.refresh_token {
            stored.refresh_token = refresh;
        }
        if let Some(id_token) = token.id_token {
            stored.id_token = id_token;
        }
        if !granted.is_empty() {
            stored.scopes = granted;
        }
        stored.expires_at = expires_at;
        stored.saved_at = now_rfc3339();
    }
    save_store(store)?;
    Ok(token.access_token)
}

// ---------------------------------------------------------------------------
// Sign-in flow
// ---------------------------------------------------------------------------

pub fn http_client() -> Result<reqwest::Client, String> {
    reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| format_http_error("failed to build HTTP client", e))
}

/// Wait for one loopback callback request and return its query string.
/// Unrelated requests (favicon etc.) get a 404 and the wait continues.
async fn wait_for_callback(
    listener: &tokio::net::TcpListener,
    timeout_secs: u64,
) -> Result<String, String> {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(timeout_secs);
    loop {
        let remaining = deadline.saturating_duration_since(tokio::time::Instant::now());
        if remaining.is_zero() {
            return Err(format!(
                "timed out after {} seconds waiting for the browser sign-in to complete",
                timeout_secs
            ));
        }
        let (mut stream, _) = tokio::time::timeout(remaining, listener.accept())
            .await
            .map_err(|_| "timed out waiting for the browser sign-in to complete".to_string())?
            .map_err(|e| format!("loopback listener failed: {}", e))?;
        // A request line may arrive split across several TCP segments, so
        // accumulate until its CRLF terminator (bounded and timed out)
        // instead of trusting a single read to hold the whole line.
        let mut received: Vec<u8> = Vec::new();
        let mut chunk = [0u8; 2048];
        let request_line = loop {
            if let Some(end) = received
                .windows(2)
                .position(|w| w == b"\r\n")
                .or_else(|| received.iter().position(|b| *b == b'\n'))
            {
                break String::from_utf8_lossy(&received[..end]).to_string();
            }
            if received.len() > 16_384 {
                break String::from_utf8_lossy(&received).to_string();
            }
            // Each read is capped by both the per-read limit and the
            // overall login deadline, so a slow-drip connection cannot
            // hold the listener past the deadline.
            let read_cap = deadline
                .saturating_duration_since(tokio::time::Instant::now())
                .min(std::time::Duration::from_secs(10));
            if read_cap.is_zero() {
                return Err(format!(
                    "timed out after {} seconds waiting for the browser sign-in to complete",
                    timeout_secs
                ));
            }
            match tokio::time::timeout(read_cap, stream.read(&mut chunk)).await {
                Ok(Ok(0)) => break String::from_utf8_lossy(&received).to_string(),
                Ok(Ok(size)) => received.extend_from_slice(&chunk[..size]),
                _ => break String::new(),
            }
        };
        if request_line.is_empty() {
            continue;
        }
        let mut parts = request_line.split_whitespace();
        let _method = parts.next();
        let target = parts.next().unwrap_or_default().to_string();
        let (path, query) = match target.split_once('?') {
            Some((p, q)) => (p.to_string(), q.to_string()),
            None => (target.clone(), String::new()),
        };
        if path != CALLBACK_PATH {
            let _ = stream
                .write_all(b"HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\n\r\n")
                .await;
            continue;
        }
        let body = "<html><body><h1>Signed in</h1><p>You can close this tab and return to ask-bridge.</p></body></html>";
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\n\r\n{}",
            body.len(),
            body
        );
        let _ = stream.write_all(response.as_bytes()).await;
        return Ok(query);
    }
}

/// Run the interactive SIWC sign-in: dynamic registration on first use,
/// issued-client reauthorization afterwards.
pub async fn login() -> Result<(), String> {
    let mut store = load_store()?;
    // Persist a freshly generated host ID immediately so the same value is
    // reused even if this attempt is abandoned.
    if !credentials_path()?.exists() {
        save_store(&store)?;
    }
    let existing = store.active_account().cloned();
    let (client_id, dynamic_registration, id_token_hint, login_hint) = match &existing {
        Some(account) => (
            account.client_id.clone(),
            false,
            Some(account.id_token.clone()),
            account.email.clone(),
        ),
        None => (DYNAMIC_CLIENT_ID.to_string(), true, None, None),
    };

    let listener = match tokio::net::TcpListener::bind(("127.0.0.1", CALLBACK_PORT)).await {
        Ok(l) => l,
        Err(_) => tokio::net::TcpListener::bind(("127.0.0.1", 0))
            .await
            .map_err(|e| format!("failed to start the loopback callback listener: {}", e))?,
    };
    let port = listener
        .local_addr()
        .map_err(|e| format!("failed to read the loopback listener address: {}", e))?
        .port();
    let redirect_uri = format!("http://127.0.0.1:{}{}", port, CALLBACK_PATH);

    let state = random_token();
    let nonce = random_token();
    let (verifier, challenge) = new_pkce_pair();
    let authorize_url = build_authorize_url(&AuthorizeRequest {
        client_id: client_id.clone(),
        dynamic_registration,
        host_id: store.host_id.clone(),
        redirect_uri: redirect_uri.clone(),
        state: state.clone(),
        nonce: nonce.clone(),
        code_challenge: challenge,
        id_token_hint,
        login_hint,
    });

    println!("Opening your browser to sign in with ChatGPT...");
    println!("If the browser does not open, visit this URL manually:");
    println!("{}", redacted_authorize_url(&authorize_url));
    let _ = open::that(&authorize_url);

    let query = wait_for_callback(&listener, LOGIN_TIMEOUT_SECS).await?;
    let callback = parse_callback_query(&query);
    if let Some(error) = &callback.error {
        if error == "access_denied" {
            return Err(
                "sign-in was declined in the browser; no credentials were stored. If ChatGPT plan usage is not enabled for your account, authorize it later and run login again"
                    .to_string(),
            );
        }
        return Err(format!("sign-in failed in the browser: {}", error));
    }
    if callback.state.as_deref() != Some(state.as_str()) {
        return Err("sign-in callback state does not match this attempt; aborting".to_string());
    }
    let code = callback
        .code
        .ok_or_else(|| "sign-in callback did not include an authorization code".to_string())?;

    // The issued client ID comes from the registration callback; on
    // reauthorization it may be omitted, but a different one is a mismatch.
    let issued_client_id = match (&callback.client_id, dynamic_registration) {
        (Some(id), _) if *id != client_id && !dynamic_registration => {
            return Err(
                "sign-in callback returned a different client ID than the selected account; aborting"
                    .to_string(),
            );
        }
        (Some(id), _) => id.clone(),
        (None, true) => {
            return Err(
                "registration callback did not include an issued client ID; registration is incomplete"
                    .to_string(),
            );
        }
        (None, false) => client_id.clone(),
    };

    let http = http_client()?;
    let token = post_token_form(
        &http,
        &[
            ("grant_type", "authorization_code"),
            ("code", code.as_str()),
            ("code_verifier", verifier.as_str()),
            ("client_id", issued_client_id.as_str()),
            ("redirect_uri", redirect_uri.as_str()),
            ("resource", RESOURCE),
        ],
    )
    .await?;
    let id_token = token
        .id_token
        .clone()
        .ok_or_else(|| "token response did not include an ID token".to_string())?;
    let claims = validate_id_token(&http, &id_token, &issued_client_id, &nonce).await?;

    if let Some(previous) = store
        .accounts
        .iter()
        .find(|a| a.client_id == issued_client_id)
    {
        if previous.subject != claims.sub {
            return Err(
                "the signed-in ChatGPT account does not match the account registered for this client; credentials were not replaced"
                    .to_string(),
            );
        }
    }

    let granted = token.granted_scopes();
    let scopes = if granted.is_empty() {
        SCOPES.split_whitespace().map(|s| s.to_string()).collect()
    } else {
        granted
    };
    let account = SiwcAccount {
        client_id: issued_client_id,
        email: claims.email.clone(),
        subject: claims.sub.clone(),
        issuer: ISSUER.to_string(),
        id_token,
        access_token: token.access_token.clone(),
        refresh_token: token
            .refresh_token
            .clone()
            .ok_or_else(|| "token response did not include a refresh token".to_string())?,
        expires_at: now_unix() + token.expires_in.unwrap_or(3600),
        scopes,
        saved_at: now_rfc3339(),
    };
    if !account.has_plan_scope() {
        return Err(
            "ChatGPT plan usage was not granted (missing scope chatgpt.tokens.use.direct). Enable plan usage for your account, then run login again; no credentials were stored"
                .to_string(),
        );
    }
    let email = account.email.clone().unwrap_or_else(|| claims.sub.clone());
    store.upsert_account(account);
    save_store(&store)?;

    println!("Signed in with ChatGPT as {}.", email);
    println!(
        "Responses through this backend use your ChatGPT plan's Work/Codex usage or credits; they are not free or unlimited."
    );
    match fetch_models(&http, &token.access_token).await {
        Ok(models) => {
            println!("{} models are available to this account:", models.len());
            for model in models.iter().take(10) {
                println!("  - {} ({})", model.display_name, model.slug);
            }
            if models.len() > 10 {
                println!("  ... and {} more", models.len() - 10);
            }
        }
        Err(e) => {
            println!(
                "Signed in, but the model catalog could not be loaded: {}",
                e
            );
        }
    }
    Ok(())
}

/// Sign out: delete the stored SIWC credentials.
pub fn logout() -> Result<(), String> {
    if clear_store()? {
        println!("Signed out of ChatGPT (stored SIWC account credentials removed).");
    } else {
        println!("No SIWC credentials were stored.");
    }
    Ok(())
}

/// A compact view of the currently stored sign-in state (never includes
/// token values).
pub fn describe_store(store: &SiwcStore) -> BTreeMap<&'static str, String> {
    let mut map = BTreeMap::new();
    match store.active_account() {
        Some(account) => {
            map.insert(
                "account",
                account
                    .email
                    .clone()
                    .unwrap_or_else(|| account.subject.clone()),
            );
            map.insert("plan_scope", account.has_plan_scope().to_string());
        }
        None => {
            map.insert("account", "not signed in".to_string());
        }
    }
    map
}

// ---------------------------------------------------------------------------
// Inference (Responses API over SSE)
// ---------------------------------------------------------------------------

/// Build the `/v1/responses` request body. Only fields the SIWC route
/// accepts are sent: `store: false` and `stream: true` are mandatory, and
/// fields such as `temperature`, `max_output_tokens`, `metadata` or `user`
/// must be omitted entirely.
pub fn build_responses_body(model: &str, prompt: &str) -> serde_json::Value {
    serde_json::json!({
        "model": model,
        "input": [{ "role": "user", "content": prompt }],
        "store": false,
        "stream": true,
    })
}

/// Resolve a user-supplied `--model` value against the account catalog:
/// exact slug, then case-insensitive slug or display name (mirroring the
/// web flow's forgiving menu-label matching). Without a value, the first
/// catalog entry (the account's server-ordered default) is used.
pub fn resolve_model(catalog: &[ModelInfo], requested: Option<&str>) -> Result<String, String> {
    if catalog.is_empty() {
        return Err("the account model catalog is empty; no SIWC models are available".to_string());
    }
    let Some(requested) = requested.map(str::trim).filter(|s| !s.is_empty()) else {
        return Ok(catalog[0].slug.clone());
    };
    if let Some(found) = catalog.iter().find(|m| m.slug == requested) {
        return Ok(found.slug.clone());
    }
    let lowered = requested.to_ascii_lowercase();
    if let Some(found) = catalog.iter().find(|m| {
        m.slug.to_ascii_lowercase() == lowered || m.display_name.to_ascii_lowercase() == lowered
    }) {
        return Ok(found.slug.clone());
    }
    let available = catalog
        .iter()
        .map(|m| m.slug.as_str())
        .collect::<Vec<_>>()
        .join(", ");
    Err(format!(
        "model \"{}\" is not available to this ChatGPT account; available models: {}",
        requested, available
    ))
}

/// Accumulates one SSE stream into its final outcome. Only
/// `response.completed` counts as success; failures that arrive mid-stream
/// (including plan usage limits) are surfaced with their error code.
#[derive(Debug, Default)]
pub struct StreamAccumulator {
    pub text: String,
    pub completed: bool,
    pub failure: Option<String>,
    pub incomplete: Option<String>,
}

impl StreamAccumulator {
    /// Apply one SSE `data:` payload; returns any output text delta so the
    /// caller can print it live.
    pub fn apply(&mut self, data: &str) -> Option<String> {
        let data = data.trim();
        if data.is_empty() || data == "[DONE]" {
            return None;
        }
        let event: serde_json::Value = serde_json::from_str(data).ok()?;
        match event.get("type").and_then(|t| t.as_str()) {
            Some("response.output_text.delta") | Some("response.refusal.delta") => {
                let delta = event
                    .get("delta")
                    .and_then(|d| d.as_str())
                    .unwrap_or_default()
                    .to_string();
                self.text.push_str(&delta);
                Some(delta)
            }
            Some("response.completed") => {
                self.completed = true;
                None
            }
            Some("response.failed") => {
                let error = event
                    .get("response")
                    .and_then(|r| r.get("error"))
                    .cloned()
                    .unwrap_or(serde_json::Value::Null);
                let code = error
                    .get("code")
                    .and_then(|c| c.as_str())
                    .unwrap_or("unknown_error");
                let message = error
                    .get("message")
                    .and_then(|m| m.as_str())
                    .unwrap_or_default();
                self.failure = Some(
                    format!("{}: {}", code, message)
                        .trim_end_matches(": ")
                        .to_string(),
                );
                None
            }
            Some("response.incomplete") => {
                let reason = event
                    .get("response")
                    .and_then(|r| r.get("incomplete_details"))
                    .and_then(|d| d.get("reason"))
                    .and_then(|r| r.as_str())
                    .unwrap_or("unknown reason");
                self.incomplete = Some(reason.to_string());
                None
            }
            _ => None,
        }
    }

    /// True once a terminal event (completed / failed / incomplete) has
    /// been applied; the caller should stop reading the stream there.
    pub fn is_terminal(&self) -> bool {
        self.completed || self.failure.is_some() || self.incomplete.is_some()
    }

    pub fn into_result(self) -> Result<String, String> {
        if let Some(failure) = &self.failure {
            if failure.contains("subscription_sharing_usage_limit_exceeded")
                || failure.contains("subscription_sharing_usage_unavailable")
            {
                return Err(format!(
                    "ChatGPT plan usage limit reached ({}). The SIWC backend draws on your plan's Work/Codex usage or credits; wait for the limit to reset or switch back to --backend browser. No fallback was attempted",
                    failure
                ));
            }
            return Err(format!("the response failed: {}", failure));
        }
        if let Some(reason) = &self.incomplete {
            return Err(format!("the response ended incomplete: {}", reason));
        }
        if !self.completed {
            return Err(
                "the stream ended without response.completed; the response is not usable"
                    .to_string(),
            );
        }
        Ok(self.text)
    }
}

/// Incremental SSE framing over raw bytes: chunk boundaries may split
/// multi-byte UTF-8 characters, so bytes are buffered and only complete
/// event blocks (separated by a blank line, LF or CRLF) are decoded.
#[derive(Default)]
pub struct SseParser {
    buffer: Vec<u8>,
}

fn find_subslice(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}

impl SseParser {
    pub fn push(&mut self, chunk: &[u8]) -> Vec<String> {
        self.buffer.extend_from_slice(chunk);
        let mut payloads = Vec::new();
        loop {
            let lf = find_subslice(&self.buffer, b"\n\n").map(|i| (i, 2));
            let crlf = find_subslice(&self.buffer, b"\r\n\r\n").map(|i| (i, 4));
            let Some((idx, len)) = [lf, crlf].into_iter().flatten().min_by_key(|(i, _)| *i) else {
                break;
            };
            let block: Vec<u8> = self.buffer.drain(..idx + len).collect();
            if let Some(payload) = decode_sse_block(&block) {
                payloads.push(payload);
            }
        }
        payloads
    }

    /// Flush a trailing event block that arrived without its final blank
    /// line before the stream closed.
    pub fn finish(&mut self) -> Vec<String> {
        if self.buffer.is_empty() {
            return Vec::new();
        }
        let block = std::mem::take(&mut self.buffer);
        decode_sse_block(&block).into_iter().collect()
    }
}

fn decode_sse_block(block: &[u8]) -> Option<String> {
    let text = String::from_utf8_lossy(block);
    let data: Vec<&str> = text
        .lines()
        .filter_map(|line| line.strip_prefix("data:"))
        .map(|d| d.trim_start())
        .collect();
    if data.is_empty() {
        None
    } else {
        Some(data.join("\n"))
    }
}

fn classify_http_error(status: reqwest::StatusCode, body: &str) -> String {
    let parsed: serde_json::Value = serde_json::from_str(body).unwrap_or_default();
    let code = parsed
        .get("error")
        .and_then(|e| e.get("code"))
        .and_then(|c| c.as_str())
        .unwrap_or_default();
    let message = parsed
        .get("error")
        .and_then(|e| e.get("message"))
        .and_then(|m| m.as_str())
        .unwrap_or_default();
    match status.as_u16() {
        401 => "the stored ChatGPT sign-in was rejected (401); run `ask-bridge --provider chatgpt --backend siwc login` again".to_string(),
        429 => format!(
            "ChatGPT plan usage or rate limit reached (429): {} {}",
            code, message
        )
        .trim()
        .to_string(),
        _ => format!("Responses API returned {}: {} {}", status, code, message)
            .trim()
            .to_string(),
    }
}

/// Run one single-turn prompt through the SIWC backend, streaming output
/// text to stdout as it arrives. Returns the full response text on
/// success (the caller handles `-o` persistence).
pub async fn ask(
    prompt: &str,
    requested_model: Option<&str>,
    timeout_secs: u64,
) -> Result<String, String> {
    let mut store = load_store()?;
    if store.active_account().is_none() {
        return Err(
            "not signed in with ChatGPT; run `ask-bridge --provider chatgpt --backend siwc login` first"
                .to_string(),
        );
    }
    // The overall deadline covers everything: token refresh, the model
    // catalog and the streamed exchange. The pre-requests additionally use
    // the standard 30-second-timeout client; only the streaming call uses
    // a client without a total timeout (the deadline governs it).
    let deadline = std::time::Duration::from_secs(timeout_secs.max(1));
    let work = async {
        let http = http_client()?;
        let access_token = fresh_access_token(&http, &mut store).await?;
        let catalog = fetch_models(&http, &access_token).await?;
        let model = resolve_model(&catalog, requested_model)?;

        let stream_http = reqwest::Client::builder()
            .build()
            .map_err(|e| format_http_error("failed to build HTTP client", e))?;
        let response = stream_http
            .post(format!("{}/responses", API_BASE))
            .bearer_auth(&access_token)
            .json(&build_responses_body(&model, prompt))
            .send()
            .await
            .map_err(|e| format_http_error("Responses request failed", e))?;
        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(classify_http_error(status, &body));
        }
        let mut response = response;
        let mut parser = SseParser::default();
        let mut accumulator = StreamAccumulator::default();
        use std::io::Write;
        'stream: loop {
            match response.chunk().await {
                Ok(Some(chunk)) => {
                    for payload in parser.push(&chunk) {
                        if let Some(delta) = accumulator.apply(&payload) {
                            print!("{}", delta);
                            let _ = std::io::stdout().flush();
                        }
                        if accumulator.is_terminal() {
                            // A terminal event settles the outcome; do not
                            // let a lingering connection replace it with a
                            // timeout or interruption error.
                            break 'stream;
                        }
                    }
                }
                Ok(None) => {
                    for payload in parser.finish() {
                        if let Some(delta) = accumulator.apply(&payload) {
                            print!("{}", delta);
                            let _ = std::io::stdout().flush();
                        }
                    }
                    break;
                }
                Err(e) => {
                    if accumulator.is_terminal() {
                        break;
                    }
                    return Err(format_http_error("the response stream was interrupted", e));
                }
            }
        }
        accumulator.into_result()
    };
    let text = tokio::time::timeout(deadline, work)
        .await
        .map_err(|_| format!("SIWC request timed out after {} seconds", timeout_secs))??;
    if !text.ends_with('\n') {
        println!();
    }
    Ok(text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn http_errors_include_transport_causes_without_request_secrets() {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let http = reqwest::Client::builder()
            .no_proxy()
            .timeout(std::time::Duration::from_millis(100))
            .build()
            .unwrap();
        let error = http
            .post(format!(
                "http://{}/?code=secret-code",
                listener.local_addr().unwrap()
            ))
            .bearer_auth("secret-access-token")
            .form(&[("refresh_token", "secret-refresh-token")])
            .send()
            .await
            .unwrap_err();
        assert!(error.is_timeout());

        let message = format_http_error("token request failed", error);
        assert!(message.starts_with("token request failed:"));
        assert!(message.contains("operation timed out"), "{}", message);
        assert!(!message.contains("secret-"), "{}", message);
        assert!(!message.contains("127.0.0.1"), "{}", message);
    }

    #[test]
    fn pkce_matches_rfc7636_vector() {
        // RFC 7636 appendix B test vector.
        assert_eq!(
            pkce_challenge("dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk"),
            "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM"
        );
    }

    #[test]
    fn authorize_url_for_dynamic_registration_has_all_parameters() {
        let url = build_authorize_url(&AuthorizeRequest {
            client_id: DYNAMIC_CLIENT_ID.to_string(),
            dynamic_registration: true,
            host_id: "urn:uuid:host".to_string(),
            redirect_uri: "http://127.0.0.1:1455/auth/callback".to_string(),
            state: "state-1".to_string(),
            nonce: "nonce-1".to_string(),
            code_challenge: "challenge-1".to_string(),
            id_token_hint: None,
            login_hint: None,
        });
        assert!(url.starts_with("https://auth.openai.com/api/accounts/authorize?"));
        for needle in [
            "client_id=dynamic_agent_client",
            "agent_name_hint=ask-bridge",
            "ext_agent_host_id=urn%3Auuid%3Ahost",
            "response_type=code",
            "code_challenge_method=S256",
            "code_challenge=challenge-1",
            "state=state-1",
            "nonce=nonce-1",
            "resource=https%3A%2F%2Fapi.openai.com%2Fv1",
        ] {
            assert!(url.contains(needle), "missing {} in {}", needle, url);
        }
    }

    #[test]
    fn authorize_url_for_reauthorization_uses_hints_without_agent_name() {
        let url = build_authorize_url(&AuthorizeRequest {
            client_id: "oaiapp_123".to_string(),
            dynamic_registration: false,
            host_id: "urn:uuid:host".to_string(),
            redirect_uri: "http://127.0.0.1:54321/auth/callback".to_string(),
            state: "s".to_string(),
            nonce: "n".to_string(),
            code_challenge: "c".to_string(),
            id_token_hint: Some("hint-token".to_string()),
            login_hint: Some("user@example.com".to_string()),
        });
        assert!(url.contains("client_id=oaiapp_123"));
        assert!(url.contains("id_token_hint=hint-token"));
        assert!(url.contains("login_hint=user%40example.com"));
        assert!(!url.contains("agent_name_hint"));
    }

    #[test]
    fn callback_query_parsing_reads_all_fields() {
        let params = parse_callback_query("code=abc&state=xyz&client_id=oaiapp_1&scope=openid");
        assert_eq!(params.code.as_deref(), Some("abc"));
        assert_eq!(params.state.as_deref(), Some("xyz"));
        assert_eq!(params.client_id.as_deref(), Some("oaiapp_1"));
        assert!(params.error.is_none());
        let denied = parse_callback_query("error=access_denied&state=xyz");
        assert_eq!(denied.error.as_deref(), Some("access_denied"));
    }

    #[test]
    fn models_catalog_filters_display_visibility() {
        let value = serde_json::json!({
            "models": [
                {"slug": "gpt-a", "display_name": "GPT A", "visibility": "list"},
                {"slug": "gpt-hidden", "display_name": "Hidden", "visibility": "hide"},
                {"slug": "gpt-b", "display_name": "GPT B"}
            ]
        });
        let models = parse_models_catalog(&value);
        assert_eq!(models.len(), 2);
        assert_eq!(models[0].slug, "gpt-a");
        assert_eq!(models[1].display_name, "GPT B");
        assert!(parse_models_catalog(&serde_json::json!({})).is_empty());
    }

    #[test]
    fn store_roundtrips_through_json_and_tracks_active_account() {
        let mut store = SiwcStore {
            host_id: "urn:uuid:stable".to_string(),
            active_client_id: None,
            accounts: Vec::new(),
        };
        store.upsert_account(SiwcAccount {
            client_id: "oaiapp_1".to_string(),
            email: Some("user@example.com".to_string()),
            subject: "sub-1".to_string(),
            issuer: ISSUER.to_string(),
            id_token: "id".to_string(),
            access_token: "access".to_string(),
            refresh_token: "refresh".to_string(),
            expires_at: 123,
            scopes: vec![PLAN_SCOPE.to_string()],
            saved_at: "unix:1".to_string(),
        });
        let json = serde_json::to_string(&store).unwrap();
        let restored: SiwcStore = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.host_id, "urn:uuid:stable");
        let active = restored.active_account().unwrap();
        assert_eq!(active.email.as_deref(), Some("user@example.com"));
        assert!(active.has_plan_scope());
    }

    #[test]
    fn responses_body_uses_only_allowed_fields() {
        let body = build_responses_body("gpt-x", "hello");
        assert_eq!(body["model"], "gpt-x");
        assert_eq!(body["store"], false);
        assert_eq!(body["stream"], true);
        assert_eq!(body["input"][0]["role"], "user");
        assert_eq!(body["input"][0]["content"], "hello");
        for forbidden in [
            "temperature",
            "max_output_tokens",
            "metadata",
            "user",
            "previous_response_id",
        ] {
            assert!(
                body.get(forbidden).is_none(),
                "field {} must be omitted",
                forbidden
            );
        }
    }

    #[test]
    fn stream_accumulator_handles_delta_completed_failed_and_incomplete() {
        let mut acc = StreamAccumulator::default();
        let delta = acc.apply(r#"{"type":"response.output_text.delta","delta":"Hello"}"#);
        assert_eq!(delta.as_deref(), Some("Hello"));
        acc.apply(r#"{"type":"response.output_text.delta","delta":" world"}"#);
        acc.apply(r#"{"type":"response.completed","response":{}}"#);
        assert_eq!(acc.into_result().unwrap(), "Hello world");

        let mut acc = StreamAccumulator::default();
        acc.apply(r#"{"type":"response.failed","response":{"error":{"code":"subscription_sharing_usage_limit_exceeded","message":"limit"}}}"#);
        assert!(acc.into_result().unwrap_err().contains("plan usage limit"));

        let mut acc = StreamAccumulator::default();
        acc.apply(r#"{"type":"response.incomplete","response":{"incomplete_details":{"reason":"max_output_tokens"}}}"#);
        assert!(acc.into_result().unwrap_err().contains("incomplete"));

        let mut acc = StreamAccumulator::default();
        acc.apply(r#"{"type":"response.output_text.delta","delta":"partial"}"#);
        assert!(
            acc.into_result()
                .unwrap_err()
                .contains("response.completed")
        );
    }

    #[test]
    fn sse_parser_preserves_utf8_split_across_chunks_and_crlf_frames() {
        // Split the stream inside a multi-byte character and use CRLF
        // event separators: the decoded payload must be intact.
        let raw_text = "data: {\"type\":\"response.output_text.delta\",\"delta\":\"中文\"}\r\n\r\n";
        let raw = raw_text.as_bytes();
        // Split inside the multi-byte characters themselves: the first cut
        // lands after the first byte of 中, the second after the first
        // byte of 文. Per-chunk lossy decoding would corrupt both.
        let first = raw_text.find("中文").unwrap() + 1;
        let second = raw_text.find("文").unwrap() + 1;
        let mut parser = SseParser::default();
        assert!(parser.push(&raw[..first]).is_empty());
        assert!(parser.push(&raw[first..second]).is_empty());
        let payloads = parser.push(&raw[second..]);
        assert_eq!(payloads.len(), 1);
        let mut acc = StreamAccumulator::default();
        let delta = acc.apply(&payloads[0]);
        assert_eq!(delta.as_deref(), Some("中文"));

        // A trailing event without its final blank line is flushed.
        let mut parser = SseParser::default();
        assert!(
            parser
                .push(b"data: {\"type\":\"response.completed\"}")
                .is_empty()
        );
        assert_eq!(parser.finish().len(), 1);
    }

    #[test]
    fn stream_accumulator_collects_refusals_and_reports_terminal() {
        let mut acc = StreamAccumulator::default();
        let delta =
            acc.apply(r#"{"type":"response.refusal.delta","delta":"I cannot help with that"}"#);
        assert_eq!(delta.as_deref(), Some("I cannot help with that"));
        assert!(!acc.is_terminal());
        acc.apply(r#"{"type":"response.completed","response":{}}"#);
        assert!(acc.is_terminal());
        assert_eq!(acc.into_result().unwrap(), "I cannot help with that");
    }

    #[test]
    fn sse_parser_frames_events_across_chunks() {
        let mut parser = SseParser::default();
        let first = parser.push(b"data: {\"type\":\"one\"}\n\ndata: {\"type\":\"tw");
        assert_eq!(first, vec!["{\"type\":\"one\"}"],);
        let second = parser.push(b"o\"}\n\n");
        assert_eq!(second, vec!["{\"type\":\"two\"}"],);
    }

    #[test]
    fn model_resolution_matches_slug_then_display_name() {
        let catalog = vec![
            ModelInfo {
                slug: "gpt-a".to_string(),
                display_name: "GPT A".to_string(),
            },
            ModelInfo {
                slug: "gpt-b".to_string(),
                display_name: "GPT B".to_string(),
            },
        ];
        assert_eq!(resolve_model(&catalog, None).unwrap(), "gpt-a");
        assert_eq!(resolve_model(&catalog, Some("gpt-b")).unwrap(), "gpt-b");
        assert_eq!(resolve_model(&catalog, Some("GPT A")).unwrap(), "gpt-a");
        let err = resolve_model(&catalog, Some("gpt-z")).unwrap_err();
        assert!(err.contains("gpt-a, gpt-b"), "unexpected: {}", err);
        assert!(resolve_model(&[], None).is_err());
    }

    #[test]
    fn printed_authorize_url_never_contains_the_id_token_hint() {
        let url = build_authorize_url(&AuthorizeRequest {
            client_id: "oaiapp_123".to_string(),
            dynamic_registration: false,
            host_id: "urn:uuid:host".to_string(),
            redirect_uri: "http://127.0.0.1:1455/auth/callback".to_string(),
            state: "s".to_string(),
            nonce: "n".to_string(),
            code_challenge: "c".to_string(),
            id_token_hint: Some("SECRET-ID-TOKEN".to_string()),
            login_hint: Some("user@example.com".to_string()),
        });
        assert!(
            url.contains("SECRET-ID-TOKEN"),
            "browser URL keeps the hint"
        );
        let printed = redacted_authorize_url(&url);
        assert!(!printed.contains("SECRET-ID-TOKEN"));
        assert!(!printed.contains("id_token_hint"));
        assert!(printed.contains("login_hint=user%40example.com"));
    }

    #[test]
    fn refresh_window_is_sixty_seconds() {
        assert!(token_needs_refresh(1000, 941));
        assert!(token_needs_refresh(1000, 1000));
        assert!(!token_needs_refresh(1000, 900));
    }

    #[test]
    fn backend_config_values_roundtrip() {
        assert_eq!(Backend::from_config_value("siwc"), Some(Backend::Siwc));
        assert_eq!(
            Backend::from_config_value(" Browser "),
            Some(Backend::Browser)
        );
        assert_eq!(Backend::from_config_value("api"), None);
        assert_eq!(Backend::Siwc.as_config_value(), "siwc");
    }

    #[test]
    fn host_ids_are_uuid_shaped_and_unique() {
        let a = generate_host_id();
        let b = generate_host_id();
        assert!(a.starts_with("urn:uuid:"));
        assert_eq!(a.len(), "urn:uuid:".len() + 36);
        assert_ne!(a, b);
    }
}
