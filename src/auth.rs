use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use base64::Engine;
use chrono::Utc;
use hmac::{Hmac, Mac};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use sha1::Sha1;

use crate::error::WpsError;
use crate::secure_store;

type HmacSha1 = Hmac<Sha1>;

#[derive(Debug, Clone, Copy)]
pub enum AuthType {
    App,
    User,
    Cookie,
}

impl AuthType {
    pub fn parse(v: &str) -> Self {
        if v.eq_ignore_ascii_case("user") {
            Self::User
        } else if v.eq_ignore_ascii_case("cookie") {
            Self::Cookie
        } else {
            Self::App
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TokenCache {
    pub app_access_token: Option<String>,
    pub app_expires_at: Option<u64>,
    pub user_access_token: Option<String>,
    pub user_refresh_token: Option<String>,
    pub user_expires_at: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Credentials {
    pub ak: Option<String>,
    pub sk: Option<String>,
    pub oauth_server: Option<String>,
    pub oauth_redirect_uri: Option<String>,
    pub oauth_scope: Option<String>,
    pub oauth_device_endpoint: Option<String>,
    pub oauth_token_endpoint: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceAuthSession {
    pub device_code: String,
    pub user_code: Option<String>,
    pub verification_uri: Option<String>,
    pub verification_uri_complete: Option<String>,
    pub expires_in: u64,
    pub interval: u64,
    pub raw: serde_json::Value,
}

pub fn config_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("WPS_CLI_CONFIG_DIR") {
        return PathBuf::from(dir);
    }
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".config")
        .join("wps")
}

pub fn credentials_path() -> PathBuf {
    config_dir().join("credentials.enc")
}

pub fn token_cache_path() -> PathBuf {
    config_dir().join("token_cache.enc")
}

pub fn legacy_credentials_path() -> PathBuf {
    config_dir().join("credentials.json")
}

pub fn legacy_token_cache_path() -> PathBuf {
    config_dir().join("token_cache.json")
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct CookiePayload {
    cookie: Option<String>,
    cookie_string: Option<String>,
    wps_sid: Option<String>,
}

fn normalize_cookie_string(raw: &str) -> Option<String> {
    let v = raw.trim();
    if v.is_empty() {
        return None;
    }
    if v.contains('=') {
        return Some(v.to_string());
    }
    Some(format!("wps_sid={v}; csrf={v}"))
}

fn cookie_from_sid(raw: &str) -> Option<String> {
    let sid = raw.trim();
    if sid.is_empty() {
        return None;
    }
    Some(format!("wps_sid={sid}; csrf={sid}"))
}

fn parse_cookie_file_payload(raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }
    if trimmed.starts_with('{') {
        if let Ok(v) = serde_json::from_str::<CookiePayload>(trimmed) {
            if let Some(cookie) = v.cookie_string.as_deref().and_then(normalize_cookie_string) {
                return Some(cookie);
            }
            if let Some(cookie) = v.cookie.as_deref().and_then(normalize_cookie_string) {
                return Some(cookie);
            }
            if let Some(cookie) = v.wps_sid.as_deref().and_then(cookie_from_sid) {
                return Some(cookie);
            }
        }
    }
    normalize_cookie_string(trimmed)
}

fn read_cookie_file(path: &Path) -> Option<String> {
    let raw = std::fs::read_to_string(path).ok()?;
    parse_cookie_file_payload(&raw)
}

pub fn cookie_api_base() -> String {
    std::env::var("WPS_CLI_COOKIE_BASE")
        .ok()
        .filter(|v| !v.trim().is_empty())
        .unwrap_or_else(|| "https://api.wps.cn".to_string())
}

pub fn cookie_origin() -> String {
    std::env::var("WPS_CLI_COOKIE_ORIGIN")
        .ok()
        .filter(|v| !v.trim().is_empty())
        .unwrap_or_else(|| "https://365.kdocs.cn".to_string())
}

pub fn cookie_referer() -> String {
    std::env::var("WPS_CLI_COOKIE_REFERER")
        .ok()
        .filter(|v| !v.trim().is_empty())
        .unwrap_or_else(|| "https://365.kdocs.cn/woa/im/messages".to_string())
}

pub fn get_cookie_header() -> Result<String, WpsError> {
    for k in ["WPS_CLI_COOKIE", "WPS_COOKIE"] {
        if let Ok(v) = std::env::var(k) {
            if let Some(cookie) = normalize_cookie_string(&v) {
                return Ok(cookie);
            }
        }
    }
    for k in ["WPS_CLI_WPS_SID", "WPS_SID", "wps_sid"] {
        if let Ok(v) = std::env::var(k) {
            if let Some(cookie) = cookie_from_sid(&v) {
                return Ok(cookie);
            }
        }
    }
    if let Ok(file_path) = std::env::var("WPS_CLI_COOKIE_FILE") {
        let p = PathBuf::from(file_path);
        if let Some(cookie) = read_cookie_file(&p) {
            return Ok(cookie);
        }
    }

    let mut candidates: Vec<PathBuf> = vec![];
    candidates.push(config_dir().join("cookie.json"));
    candidates.push(config_dir().join("cookie_cache.json"));
    if let Some(home) = dirs::home_dir() {
        candidates.push(home.join(".cursor/skills/wpsv7-skills/.wps_sid_cache.json"));
        candidates.push(home.join(".cursor/skills/wps-sign/cookie.json"));
    }
    for p in candidates {
        if p.exists() {
            if let Some(cookie) = read_cookie_file(&p) {
                return Ok(cookie);
            }
        }
    }
    Err(WpsError::Auth(
        "missing cookie auth credential. Set `WPS_CLI_COOKIE` (full cookie) or `WPS_CLI_WPS_SID`, or provide `WPS_CLI_COOKIE_FILE`.".to_string(),
    ))
}

pub fn save_credentials(creds: &Credentials) -> Result<(), WpsError> {
    let dir = config_dir();
    let payload = serde_json::to_string_pretty(creds)
        .map_err(|e| WpsError::Auth(format!("failed to serialize credentials: {e}")))?;
    secure_store::save_encrypted_json(&dir, &credentials_path(), &payload)?;
    Ok(())
}

pub fn load_credentials() -> Credentials {
    let enc_path = credentials_path();
    if enc_path.exists() {
        if let Ok(raw) = secure_store::load_encrypted_json(&config_dir(), &enc_path) {
            if let Ok(v) = serde_json::from_str::<Credentials>(&raw) {
                return v;
            }
        }
    }
    let plain_path = legacy_credentials_path();
    if plain_path.exists() {
        if let Ok(raw) = std::fs::read_to_string(&plain_path) {
            if let Ok(v) = serde_json::from_str::<Credentials>(&raw) {
                let _ = save_credentials(&v);
                let _ = std::fs::remove_file(&plain_path);
                return v;
            }
        }
    }
    Credentials::default()
}

pub fn load_token_cache() -> TokenCache {
    let enc_path = token_cache_path();
    if enc_path.exists() {
        if let Ok(raw) = secure_store::load_encrypted_json(&config_dir(), &enc_path) {
            if let Ok(v) = serde_json::from_str::<TokenCache>(&raw) {
                return v;
            }
        }
    }
    let plain_path = legacy_token_cache_path();
    if plain_path.exists() {
        if let Ok(raw) = std::fs::read_to_string(&plain_path) {
            if let Ok(v) = serde_json::from_str::<TokenCache>(&raw) {
                let _ = save_token_cache(&v);
                let _ = std::fs::remove_file(&plain_path);
                return v;
            }
        }
    }
    TokenCache::default()
}

pub fn save_token_cache(cache: &TokenCache) -> Result<(), WpsError> {
    let dir = config_dir();
    let payload = serde_json::to_string_pretty(cache)
        .map_err(|e| WpsError::Auth(format!("failed to serialize token cache: {e}")))?;
    secure_store::save_encrypted_json(&dir, &token_cache_path(), &payload)?;
    Ok(())
}

fn unix_ts() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

pub fn rfc1123_now() -> String {
    Utc::now().format("%a, %d %b %Y %H:%M:%S GMT").to_string()
}

pub fn generate_kso1_signature(method: &str, full_url: &str, date_str: &str, sk: &str) -> Result<String, WpsError> {
    let parsed = url::Url::parse(full_url)
        .map_err(|e| WpsError::Auth(format!("invalid url for signature: {e}")))?;
    let mut sign_path = parsed.path().to_string();
    if let Some(q) = parsed.query() {
        sign_path.push('?');
        sign_path.push_str(q);
    }
    let string_to_sign = format!("{}\n{}\n{}", method.to_uppercase(), sign_path, date_str);
    let mut mac = HmacSha1::new_from_slice(sk.as_bytes())
        .map_err(|e| WpsError::Auth(format!("failed to initialize hmac: {e}")))?;
    mac.update(string_to_sign.as_bytes());
    let signature = base64::engine::general_purpose::STANDARD.encode(mac.finalize().into_bytes());
    Ok(format!("KSO-1 {signature}"))
}

async fn fetch_token_from_oauth_server(base: &str, auth_type: AuthType) -> Option<String> {
    let endpoint = match auth_type {
        AuthType::App => "/api/token/app",
        AuthType::User => "/api/token/user",
        AuthType::Cookie => return None,
    };
    let url = format!("{}{}", base.trim_end_matches('/'), endpoint);
    let client = Client::new();
    let resp = client.get(url).send().await.ok()?;
    let value = resp.json::<serde_json::Value>().await.ok()?;
    value
        .get("token")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .or_else(|| value.get("access_token").and_then(|v| v.as_str()).map(|s| s.to_string()))
}

async fn fetch_app_token_with_ak_sk(ak: &str, sk: &str) -> Result<(String, u64), WpsError> {
    let client = Client::new();
    let resp = client
        .post("https://openapi.wps.cn/oauth2/token")
        .form(&[
            ("grant_type", "client_credentials"),
            ("client_id", ak),
            ("client_secret", sk),
        ])
        .send()
        .await
        .map_err(|e| WpsError::Network(format!("failed to request app token: {e}")))?;
    let val = resp
        .json::<serde_json::Value>()
        .await
        .map_err(|e| WpsError::Network(format!("failed to parse app token response: {e}")))?;
    let token = val
        .get("access_token")
        .and_then(|v| v.as_str())
        .ok_or_else(|| WpsError::Auth(format!("missing access_token in response: {val}")))?;
    let expires_in = val.get("expires_in").and_then(|v| v.as_u64()).unwrap_or(7200);
    Ok((token.to_string(), unix_ts() + expires_in.saturating_sub(60)))
}

pub fn build_oauth_authorize_url(
    client_id: &str,
    redirect_uri: &str,
    scope: &str,
    state: &str,
) -> Result<String, WpsError> {
    let mut url = reqwest::Url::parse("https://openapi.wps.cn/oauth2/auth")
        .map_err(|e| WpsError::Auth(format!("invalid authorize endpoint: {e}")))?;
    url.query_pairs_mut()
        .append_pair("response_type", "code")
        .append_pair("client_id", client_id)
        .append_pair("redirect_uri", redirect_uri)
        .append_pair("scope", scope)
        .append_pair("state", state);
    Ok(url.to_string())
}

pub async fn exchange_user_token_with_code(
    client_id: &str,
    client_secret: &str,
    redirect_uri: &str,
    code: &str,
) -> Result<TokenCache, WpsError> {
    let client = Client::new();
    let token_endpoint = oauth_token_endpoint();
    let resp = client
        .post(token_endpoint)
        .form(&[
            ("grant_type", "authorization_code"),
            ("client_id", client_id),
            ("client_secret", client_secret),
            ("code", code),
            ("redirect_uri", redirect_uri),
        ])
        .send()
        .await
        .map_err(|e| WpsError::Network(format!("failed to exchange auth code: {e}")))?;
    let val = resp
        .json::<serde_json::Value>()
        .await
        .map_err(|e| WpsError::Network(format!("failed to parse token response: {e}")))?;

    let access_token = val
        .get("access_token")
        .and_then(|v| v.as_str())
        .ok_or_else(|| {
            let msg = val
                .get("msg")
                .and_then(|v| v.as_str())
                .or_else(|| val.get("error").and_then(|v| v.as_str()))
                .unwrap_or("");
            if msg.eq_ignore_ascii_case("invalid_grant") {
                WpsError::Auth(format!(
                    "授权码无效或已过期，请重新执行登录流程: wpscli auth login --user --mode local. raw={val}"
                ))
            } else {
                WpsError::Auth(format!("missing access_token in response: {val}"))
            }
        })?;
    let refresh_token = val
        .get("refresh_token")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    let expires_in = val.get("expires_in").and_then(|v| v.as_u64()).unwrap_or(7200);
    Ok(TokenCache {
        user_access_token: Some(access_token.to_string()),
        user_refresh_token: refresh_token,
        user_expires_at: Some(unix_ts() + expires_in.saturating_sub(60)),
        ..Default::default()
    })
}

pub async fn refresh_user_token(
    client_id: &str,
    client_secret: &str,
    refresh_token: &str,
) -> Result<TokenCache, WpsError> {
    let client = Client::new();
    let token_endpoint = oauth_token_endpoint();
    let resp = client
        .post(token_endpoint)
        .form(&[
            ("grant_type", "refresh_token"),
            ("client_id", client_id),
            ("client_secret", client_secret),
            ("refresh_token", refresh_token),
        ])
        .send()
        .await
        .map_err(|e| WpsError::Network(format!("failed to refresh user token: {e}")))?;
    let val = resp
        .json::<serde_json::Value>()
        .await
        .map_err(|e| WpsError::Network(format!("failed to parse refresh response: {e}")))?;

    let access_token = val
        .get("access_token")
        .and_then(|v| v.as_str())
        .ok_or_else(|| {
            let msg = val
                .get("msg")
                .and_then(|v| v.as_str())
                .or_else(|| val.get("error").and_then(|v| v.as_str()))
                .unwrap_or("");
            if msg.eq_ignore_ascii_case("invalid_grant") {
                WpsError::Auth(format!(
                    "refresh_token 无效或已过期，请重新登录获取新的 refresh_token: wpscli auth login --user --mode local. raw={val}"
                ))
            } else {
                WpsError::Auth(format!("missing access_token in response: {val}"))
            }
        })?;
    let refresh = val
        .get("refresh_token")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .or_else(|| Some(refresh_token.to_string()));
    let expires_in = val.get("expires_in").and_then(|v| v.as_u64()).unwrap_or(7200);
    Ok(TokenCache {
        user_access_token: Some(access_token.to_string()),
        user_refresh_token: refresh,
        user_expires_at: Some(unix_ts() + expires_in.saturating_sub(60)),
        ..Default::default()
    })
}

pub async fn get_access_token(auth_type: AuthType) -> Result<String, WpsError> {
    fn env_non_empty(name: &str) -> Option<String> {
        std::env::var(name).ok().and_then(|v| {
            if v.trim().is_empty() {
                None
            } else {
                Some(v)
            }
        })
    }

    match auth_type {
        // app mode must not be polluted by generic/user token env
        AuthType::App => {
            if let Some(token) = env_non_empty("WPS_CLI_APP_TOKEN") {
                return Ok(token);
            }
        }
        AuthType::User => {
            if let Some(token) = env_non_empty("WPS_CLI_USER_TOKEN") {
                return Ok(token);
            }
            // backward compatibility: generic token is treated as user token only
            if let Some(token) = env_non_empty("WPS_CLI_TOKEN") {
                return Ok(token);
            }
        }
        AuthType::Cookie => {
            return Err(WpsError::Auth(
                "cookie auth does not use bearer token. Use `auth-type cookie` with cookie sources: `WPS_CLI_COOKIE`, `WPS_CLI_WPS_SID`, or `WPS_CLI_COOKIE_FILE`.".to_string(),
            ));
        }
    }

    let creds = load_credentials();
    if let Some(base) = creds
        .oauth_server
        .clone()
        .or_else(|| std::env::var("WPS_OAUTH_SERVER").ok())
    {
        if let Some(token) = fetch_token_from_oauth_server(&base, auth_type).await {
            return Ok(token);
        }
    }

    let mut cache = load_token_cache();
    let now = unix_ts();
    match auth_type {
        AuthType::App => {
            if let (Some(t), Some(exp)) = (cache.app_access_token.clone(), cache.app_expires_at) {
                if exp > now {
                    return Ok(t);
                }
            }
            let ak = std::env::var("WPS_CLI_AK")
                .ok()
                .or(creds.ak.clone())
                .ok_or_else(|| WpsError::Auth("missing AK (WPS_CLI_AK)".to_string()))?;
            let sk = std::env::var("WPS_CLI_SK")
                .ok()
                .or(creds.sk.clone())
                .ok_or_else(|| WpsError::Auth("missing SK (WPS_CLI_SK)".to_string()))?;
            let (token, exp) = fetch_app_token_with_ak_sk(&ak, &sk).await?;
            cache.app_access_token = Some(token.clone());
            cache.app_expires_at = Some(exp);
            let _ = save_token_cache(&cache);
            Ok(token)
        }
        AuthType::User => {
            if let (Some(t), Some(exp)) = (cache.user_access_token.clone(), cache.user_expires_at) {
                if exp > now {
                    return Ok(t);
                }
            }
            if cache.user_refresh_token.is_some() {
                if let Ok(token) = force_refresh_user_access_token().await {
                    return Ok(token);
                }
            }
            Err(WpsError::Auth(
                "missing user token; run `wpscli auth login --user` or `wpscli auth login --user-token <token>`".to_string(),
            ))
        }
        AuthType::Cookie => Err(WpsError::Auth(
            "cookie auth does not use bearer token. Use `auth-type cookie` with cookie sources: `WPS_CLI_COOKIE`, `WPS_CLI_WPS_SID`, or `WPS_CLI_COOKIE_FILE`.".to_string(),
        )),
    }
}

pub async fn force_refresh_user_access_token() -> Result<String, WpsError> {
    let creds = load_credentials();
    let mut cache = load_token_cache();
    let rt = cache
        .user_refresh_token
        .clone()
        .ok_or_else(|| WpsError::Auth("missing refresh_token; run `wpscli auth login --user` first".to_string()))?;
    let ak = std::env::var("WPS_CLI_AK")
        .ok()
        .or(creds.ak.clone())
        .ok_or_else(|| WpsError::Auth("missing AK (WPS_CLI_AK) for user token refresh".to_string()))?;
    let sk = std::env::var("WPS_CLI_SK")
        .ok()
        .or(creds.sk.clone())
        .ok_or_else(|| WpsError::Auth("missing SK (WPS_CLI_SK) for user token refresh".to_string()))?;

    let refreshed = refresh_user_token(&ak, &sk, &rt).await?;
    let token = refreshed
        .user_access_token
        .clone()
        .ok_or_else(|| WpsError::Auth("refresh succeeded but access_token missing".to_string()))?;
    cache.user_access_token = refreshed.user_access_token;
    cache.user_refresh_token = refreshed.user_refresh_token;
    cache.user_expires_at = refreshed.user_expires_at;
    let _ = save_token_cache(&cache);
    Ok(token)
}

pub fn oauth_token_endpoint() -> String {
    if let Ok(v) = std::env::var("WPS_CLI_OAUTH_TOKEN_ENDPOINT") {
        if !v.trim().is_empty() {
            return v;
        }
    }
    let creds = load_credentials();
    if let Some(v) = creds.oauth_token_endpoint {
        if !v.trim().is_empty() {
            return v;
        }
    }
    "https://openapi.wps.cn/oauth2/token".to_string()
}

pub fn oauth_device_endpoint_candidates() -> Vec<String> {
    if let Ok(v) = std::env::var("WPS_CLI_OAUTH_DEVICE_ENDPOINT") {
        if !v.trim().is_empty() {
            return vec![v];
        }
    }
    let creds = load_credentials();
    if let Some(v) = creds.oauth_device_endpoint {
        if !v.trim().is_empty() {
            return vec![v];
        }
    }
    vec![
        "https://openapi.wps.cn/oauth2/device_authorization".to_string(),
        "https://openapi.wps.cn/oauth2/device/code".to_string(),
        "https://openapi.wps.cn/oauth2/device_authorize".to_string(),
    ]
}

pub async fn start_device_authorization(
    client_id: &str,
    scope: &str,
) -> Result<DeviceAuthSession, WpsError> {
    let client = Client::new();
    let endpoints = oauth_device_endpoint_candidates();
    let mut errs: Vec<String> = Vec::new();
    for endpoint in endpoints {
        let resp = client
            .post(&endpoint)
            .form(&[("client_id", client_id), ("scope", scope)])
            .send()
            .await;
        let Ok(resp) = resp else {
            errs.push(format!("network error for {endpoint}"));
            continue;
        };
        let val = match resp.json::<serde_json::Value>().await {
            Ok(v) => v,
            Err(e) => {
                errs.push(format!("invalid json from {endpoint}: {e}"));
                continue;
            }
        };
        let device_code = val
            .get("device_code")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        if device_code.is_empty() {
            errs.push(format!("missing device_code from {endpoint}: {val}"));
            continue;
        }
        let expires_in = val.get("expires_in").and_then(|v| v.as_u64()).unwrap_or(600);
        let interval = val.get("interval").and_then(|v| v.as_u64()).unwrap_or(5);
        return Ok(DeviceAuthSession {
            device_code,
            user_code: val.get("user_code").and_then(|v| v.as_str()).map(|s| s.to_string()),
            verification_uri: val
                .get("verification_uri")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            verification_uri_complete: val
                .get("verification_uri_complete")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            expires_in,
            interval,
            raw: val,
        });
    }
    Err(WpsError::Auth(format!(
        "failed to start device authorization; tried endpoints={:?}; last_errors={:?}. set endpoint via `wpscli auth setup --oauth-device-endpoint <url>` or env `WPS_CLI_OAUTH_DEVICE_ENDPOINT`",
        oauth_device_endpoint_candidates(),
        errs
    )))
}

pub async fn poll_device_token(
    client_id: &str,
    client_secret: &str,
    device_code: &str,
    poll_interval_secs: u64,
    timeout_secs: u64,
) -> Result<TokenCache, WpsError> {
    let token_endpoint = oauth_token_endpoint();
    let client = Client::new();
    let deadline = unix_ts() + timeout_secs;
    let mut interval = poll_interval_secs.max(1);
    loop {
        if unix_ts() >= deadline {
            return Err(WpsError::Auth(
                "device login timeout waiting for user authorization".to_string(),
            ));
        }
        let payload_standard = [
            ("grant_type", "urn:ietf:params:oauth:grant-type:device_code"),
            ("client_id", client_id),
            ("client_secret", client_secret),
            ("device_code", device_code),
        ];
        let mut val = client
            .post(&token_endpoint)
            .form(&payload_standard)
            .send()
            .await
            .map_err(|e| WpsError::Network(format!("device token poll failed: {e}")))?
            .json::<serde_json::Value>()
            .await
            .map_err(|e| WpsError::Network(format!("invalid device poll response: {e}")))?;

        if val.get("access_token").and_then(|v| v.as_str()).is_none() {
            let payload_compat = [
                ("grant_type", "device_code"),
                ("client_id", client_id),
                ("client_secret", client_secret),
                ("device_code", device_code),
            ];
            val = client
                .post(&token_endpoint)
                .form(&payload_compat)
                .send()
                .await
                .map_err(|e| WpsError::Network(format!("device token poll failed: {e}")))?
                .json::<serde_json::Value>()
                .await
                .map_err(|e| WpsError::Network(format!("invalid device poll response: {e}")))?;
        }

        if let Some(access_token) = val.get("access_token").and_then(|v| v.as_str()) {
            let refresh_token = val
                .get("refresh_token")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            let expires_in = val.get("expires_in").and_then(|v| v.as_u64()).unwrap_or(7200);
            return Ok(TokenCache {
                user_access_token: Some(access_token.to_string()),
                user_refresh_token: refresh_token,
                user_expires_at: Some(unix_ts() + expires_in.saturating_sub(60)),
                ..Default::default()
            });
        }

        let err_code = val
            .get("error")
            .and_then(|v| v.as_str())
            .or_else(|| val.get("msg").and_then(|v| v.as_str()))
            .unwrap_or("");
        if err_code.eq_ignore_ascii_case("authorization_pending")
            || err_code.contains("pending")
            || val.get("code").and_then(|v| v.as_i64()) == Some(400010018)
        {
            tokio::time::sleep(std::time::Duration::from_secs(interval)).await;
            continue;
        }
        if err_code.eq_ignore_ascii_case("slow_down") || err_code.contains("slow") {
            interval = (interval + 2).min(30);
            tokio::time::sleep(std::time::Duration::from_secs(interval)).await;
            continue;
        }
        if err_code.eq_ignore_ascii_case("access_denied") || err_code.contains("denied") {
            return Err(WpsError::Auth("device authorization denied by user".to_string()));
        }
        if err_code.eq_ignore_ascii_case("expired_token") || err_code.contains("expired") {
            return Err(WpsError::Auth("device_code expired; restart login".to_string()));
        }
        return Err(WpsError::Auth(format!(
            "device token exchange failed: {val}"
        )));
    }
}

pub fn try_open_browser(url: &str) -> bool {
    #[cfg(target_os = "macos")]
    {
        return Command::new("open").arg(url).status().map(|s| s.success()).unwrap_or(false);
    }
    #[cfg(target_os = "linux")]
    {
        return Command::new("xdg-open")
            .arg(url)
            .status()
            .map(|s| s.success())
            .unwrap_or(false);
    }
    #[cfg(target_os = "windows")]
    {
        return Command::new("cmd")
            .args(["/C", "start", "", url])
            .status()
            .map(|s| s.success())
            .unwrap_or(false);
    }
    #[allow(unreachable_code)]
    false
}

pub fn wait_for_oauth_code(
    redirect_uri: &str,
    expected_state: Option<&str>,
    timeout_secs: u64,
) -> Result<String, WpsError> {
    let parsed = reqwest::Url::parse(redirect_uri)
        .map_err(|e| WpsError::Auth(format!("invalid redirect uri: {e}")))?;
    let host = parsed.host_str().unwrap_or("127.0.0.1");
    let port = parsed
        .port_or_known_default()
        .ok_or_else(|| WpsError::Auth("redirect uri missing port".to_string()))?;
    let expected_path = parsed.path().to_string();
    let bind_addr = format!("{host}:{port}");
    let listener = TcpListener::bind(&bind_addr)
        .map_err(|e| WpsError::Auth(format!("failed to bind local callback server on {bind_addr}: {e}")))?;
    listener
        .set_nonblocking(true)
        .map_err(|e| WpsError::Auth(format!("failed to set nonblocking listener: {e}")))?;
    let deadline = unix_ts() + timeout_secs;
    loop {
        if unix_ts() >= deadline {
            return Err(WpsError::Auth(
                "oauth login timeout waiting for browser callback".to_string(),
            ));
        }
        match listener.accept() {
            Ok((mut stream, _)) => {
                let mut buf = [0u8; 4096];
                let n = stream.read(&mut buf).unwrap_or(0);
                let req = String::from_utf8_lossy(&buf[..n]);
                let mut path_q = "/";
                if let Some(line) = req.lines().next() {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 2 {
                        path_q = parts[1];
                    }
                }
                let callback_url = format!("http://localhost{path_q}");
                let ok_page = b"HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\n\r\n<html><body><h3>WPS login success</h3><p>You can close this tab and return to CLI.</p></body></html>";
                let err_page = b"HTTP/1.1 400 Bad Request\r\nContent-Type: text/html; charset=utf-8\r\n\r\n<html><body><h3>WPS login failed</h3><p>Missing or invalid code/state.</p></body></html>";
                let parsed_cb = reqwest::Url::parse(&callback_url)
                    .map_err(|e| WpsError::Auth(format!("failed to parse callback url: {e}")))?;
                if parsed_cb.path() != expected_path {
                    let _ = stream.write_all(err_page);
                    continue;
                }
                let code = parsed_cb
                    .query_pairs()
                    .find(|(k, _)| k == "code")
                    .map(|(_, v)| v.to_string());
                let state = parsed_cb
                    .query_pairs()
                    .find(|(k, _)| k == "state")
                    .map(|(_, v)| v.to_string());
                let state_ok = expected_state
                    .map(|s| state.as_deref() == Some(s))
                    .unwrap_or(true);
                if let Some(c) = code {
                    if state_ok {
                        let _ = stream.write_all(ok_page);
                        return Ok(c);
                    }
                }
                let _ = stream.write_all(err_page);
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                std::thread::sleep(std::time::Duration::from_millis(150));
            }
            Err(e) => {
                return Err(WpsError::Auth(format!(
                    "failed while waiting for oauth callback: {e}"
                )));
            }
        }
    }
}

pub fn extract_code_from_callback_url(
    callback_url: &str,
    expected_state: Option<&str>,
) -> Result<String, WpsError> {
    let parsed = reqwest::Url::parse(callback_url)
        .map_err(|e| WpsError::Auth(format!("invalid callback url: {e}")))?;
    let code = parsed
        .query_pairs()
        .find(|(k, _)| k == "code")
        .map(|(_, v)| v.to_string())
        .ok_or_else(|| WpsError::Auth("missing `code` in callback url".to_string()))?;
    if let Some(expected) = expected_state {
        let actual = parsed
            .query_pairs()
            .find(|(k, _)| k == "state")
            .map(|(_, v)| v.to_string());
        if actual.as_deref() != Some(expected) {
            return Err(WpsError::Auth(format!(
                "oauth state mismatch: expected `{expected}`, got `{}`",
                actual.unwrap_or_default()
            )));
        }
    }
    Ok(code)
}

pub fn current_sk() -> Result<String, WpsError> {
    if let Ok(sk) = std::env::var("WPS_CLI_SK") {
        if !sk.is_empty() {
            return Ok(sk);
        }
    }
    let creds = load_credentials();
    creds
        .sk
        .ok_or_else(|| WpsError::Auth("missing SK (WPS_CLI_SK)".to_string()))
}
