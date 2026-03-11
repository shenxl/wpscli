use std::collections::HashMap;
use std::sync::OnceLock;

use base64::engine::general_purpose::{URL_SAFE, URL_SAFE_NO_PAD};
use base64::Engine;
use reqwest::{Client, Method};
use serde_json::Value;

use crate::auth::{self, AuthType};
use crate::descriptor;
use crate::descriptor::EndpointDescriptor;
use crate::error::WpsError;
use crate::scope_catalog::{self, ScopeType};
use crate::validate::encode_path_segment;

#[derive(Debug, Clone, Default)]
pub struct ExecOptions {
    pub path_params: HashMap<String, String>,
    pub query_params: HashMap<String, String>,
    pub headers: HashMap<String, String>,
    pub body: Option<String>,
    pub auth_type: String,
    pub dry_run: bool,
    pub retry: u32,
    pub paginate: bool,
}

#[derive(Debug, Clone)]
struct EndpointRoute {
    method: String,
    path_template: String,
    scopes: Vec<String>,
    endpoint_id: String,
}

static ENDPOINT_ROUTES: OnceLock<Vec<EndpointRoute>> = OnceLock::new();

fn build_url(
    endpoint: &EndpointDescriptor,
    path_params: &HashMap<String, String>,
    base_url: &str,
) -> Result<String, WpsError> {
    let mut path = endpoint.path.clone();
    for p in &endpoint.params.path {
        if let Some(v) = path_params.get(&p.name) {
            path = path.replace(&format!("{{{}}}", p.name), &encode_path_segment(v));
        } else if p.required {
            return Err(WpsError::Validation(format!("missing required path param {}", p.name)));
        }
    }
    Ok(format!("{}{}", base_url.trim_end_matches('/'), path))
}

fn normalize_path(path_or_url: &str) -> String {
    let mut path = if path_or_url.starts_with("http://") || path_or_url.starts_with("https://") {
        reqwest::Url::parse(path_or_url)
            .ok()
            .map(|u| u.path().to_string())
            .unwrap_or_else(|| path_or_url.to_string())
    } else {
        path_or_url.to_string()
    };
    if let Some((p, _)) = path.split_once('?') {
        path = p.to_string();
    }
    if !path.starts_with('/') {
        path = format!("/{path}");
    }
    if path.len() > 1 && path.ends_with('/') {
        path.pop();
    }
    path
}

fn path_match_score(template: &str, actual: &str) -> Option<usize> {
    let t = template.trim_matches('/').split('/').collect::<Vec<_>>();
    let a = actual.trim_matches('/').split('/').collect::<Vec<_>>();
    if t.len() != a.len() {
        return None;
    }
    let mut score = 0usize;
    for (ts, asg) in t.iter().zip(a.iter()) {
        let is_param = ts.starts_with('{') && ts.ends_with('}');
        if is_param {
            continue;
        }
        if ts != asg {
            return None;
        }
        score += 1;
    }
    Some(score)
}

fn load_endpoint_routes() -> Vec<EndpointRoute> {
    let Ok(manifest) = descriptor::load_manifest() else {
        return vec![];
    };
    let mut routes = Vec::new();
    for svc in manifest.services {
        let Some(service) = svc.get("service").and_then(|v| v.as_str()) else {
            continue;
        };
        let Ok(desc) = descriptor::load_service_descriptor(service) else {
            continue;
        };
        for ep in desc.endpoints {
            routes.push(EndpointRoute {
                method: ep.http_method.to_uppercase(),
                path_template: normalize_path(&ep.path),
                scopes: ep.scopes,
                endpoint_id: ep.id,
            });
        }
    }
    routes
}

fn endpoint_routes() -> &'static Vec<EndpointRoute> {
    ENDPOINT_ROUTES.get_or_init(load_endpoint_routes)
}

fn infer_endpoint_route(method: &str, path_or_url: &str) -> Option<EndpointRoute> {
    let method_up = method.to_uppercase();
    let actual = normalize_path(path_or_url);
    let mut best: Option<(usize, &EndpointRoute)> = None;
    for r in endpoint_routes().iter() {
        if r.method != method_up {
            continue;
        }
        if let Some(score) = path_match_score(&r.path_template, &actual) {
            match best {
                None => best = Some((score, r)),
                Some((prev, _)) if score > prev => best = Some((score, r)),
                _ => {}
            }
        }
    }
    best.map(|(_, r)| r.clone())
}

fn dedup_scopes(scopes: &[String]) -> Vec<String> {
    let mut seen = std::collections::HashSet::new();
    let mut out = Vec::new();
    for s in scopes {
        if s.trim().is_empty() {
            continue;
        }
        if seen.insert(s.clone()) {
            out.push(s.clone());
        }
    }
    out
}

fn extract_token_scopes(access_token: &str) -> Vec<String> {
    let mut parts = access_token.split('.');
    let _ = parts.next();
    let payload = parts.next().unwrap_or_default();
    if payload.is_empty() {
        return vec![];
    }
    let bytes = URL_SAFE_NO_PAD
        .decode(payload)
        .or_else(|_| URL_SAFE.decode(payload))
        .unwrap_or_default();
    if bytes.is_empty() {
        return vec![];
    }
    let value: Value = serde_json::from_slice(&bytes).unwrap_or(Value::Null);
    let mut set = std::collections::BTreeSet::new();
    if let Some(s) = value.get("scope").and_then(|v| v.as_str()) {
        for p in s.replace(',', " ").split_whitespace() {
            set.insert(p.to_string());
        }
    }
    if let Some(arr) = value.get("scopes").and_then(|v| v.as_array()) {
        for v in arr {
            if let Some(s) = v.as_str() {
                set.insert(s.to_string());
            }
        }
    }
    set.into_iter().collect()
}

fn scope_prelight(endpoint: &EndpointDescriptor, auth_type: AuthType, token: &str) -> Result<(), WpsError> {
    if matches!(auth_type, AuthType::Cookie) {
        return Ok(());
    }
    let endpoint_scopes = dedup_scopes(&endpoint.scopes);
    if endpoint_scopes.is_empty() {
        return Ok(());
    }
    let catalog_source = scope_catalog::source_label();
    let unknown_scopes = scope_catalog::filter_unknown(&endpoint_scopes);
    match auth_type {
        AuthType::User => {
            let delegated = scope_catalog::filter_supported(&endpoint_scopes, ScopeType::Delegated);
            if delegated.is_empty() && unknown_scopes.is_empty() {
                return Err(WpsError::Auth(format!(
                    "接口 `{}` 仅声明 app_role scope，当前 user token 不适配。请改用 app token（--auth-type app）。scopes={:?} catalog={}",
                    endpoint.id, endpoint_scopes, catalog_source
                )));
            }
            let token_scopes = extract_token_scopes(token);
            if token_scopes.is_empty() {
                return Ok(());
            }
            let got = token_scopes.iter().cloned().collect::<std::collections::HashSet<_>>();
            let satisfied = delegated.iter().any(|s| got.contains(s));
            if !satisfied && !delegated.is_empty() {
                let reauth_arg = delegated.join(",");
                return Err(WpsError::Auth(format!(
                    "user token scope 不匹配接口 `{}`。接口可用 delegated scopes={:?}，当前 token scopes={:?}。请执行：wpscli auth login --user --mode local --scope {}。catalog={}",
                    endpoint.id, delegated, token_scopes, reauth_arg, catalog_source
                )));
            }
        }
        AuthType::App => {
            let app_role = scope_catalog::filter_supported(&endpoint_scopes, ScopeType::AppRole);
            if app_role.is_empty() && unknown_scopes.is_empty() {
                return Err(WpsError::Auth(format!(
                    "接口 `{}` 未声明 app_role scope，当前 app token 可能不适配。请改用 user token（--user-token）。scopes={:?} catalog={}",
                    endpoint.id, endpoint_scopes, catalog_source
                )));
            }
        }
        AuthType::Cookie => {}
    }
    Ok(())
}

pub async fn execute_endpoint(endpoint: &EndpointDescriptor, opts: ExecOptions) -> Result<Value, WpsError> {
    let auth_type = AuthType::parse(&opts.auth_type);
    let base_url = if matches!(auth_type, AuthType::Cookie) {
        auth::cookie_api_base()
    } else {
        "https://openapi.wps.cn".to_string()
    };
    let mut url = build_url(endpoint, &opts.path_params, &base_url)?;
    if !opts.query_params.is_empty() {
        let parsed = reqwest::Url::parse_with_params(&url, opts.query_params.iter())
            .map_err(|e| WpsError::Validation(format!("invalid query params: {e}")))?;
        url = parsed.to_string();
    }

    if opts.dry_run {
        let mut request_headers = opts.headers.clone();
        request_headers
            .entry("Content-Type".to_string())
            .or_insert_with(|| "application/json".to_string());
        return Ok(serde_json::json!({
            "ok": true,
            "dry_run": true,
            "auth_skipped": true,
            "request": {
                "method": endpoint.http_method,
                "url": url,
                "headers": request_headers,
                "body": opts.body,
            }
        }));
    }

    let method = Method::from_bytes(endpoint.http_method.as_bytes())
        .map_err(|e| WpsError::Validation(format!("invalid http method: {e}")))?;
    let mut token = String::new();
    let mut sk: Option<String> = None;
    let mut cookie_header: Option<String> = None;
    if matches!(auth_type, AuthType::Cookie) {
        cookie_header = Some(auth::get_cookie_header()?);
    } else {
        token = auth::get_access_token(auth_type).await?;
        scope_prelight(endpoint, auth_type, &token)?;
        sk = Some(auth::current_sk()?);
    }

    let client = Client::new();
    let attempts = std::cmp::max(1, opts.retry + 1);
    let mut last_err = None;
    let mut refreshed_once = false;
    for attempt in 1..=attempts {
        let mut request_headers = opts.headers.clone();
        request_headers
            .entry("Content-Type".to_string())
            .or_insert_with(|| "application/json".to_string());
        if matches!(auth_type, AuthType::Cookie) {
            request_headers
                .entry("Origin".to_string())
                .or_insert_with(auth::cookie_origin);
            request_headers
                .entry("Referer".to_string())
                .or_insert_with(auth::cookie_referer);
            request_headers.insert(
                "Cookie".to_string(),
                cookie_header.clone().unwrap_or_default(),
            );
        } else {
            let date = auth::rfc1123_now();
            let signature = auth::generate_kso1_signature(
                endpoint.http_method.as_str(),
                &url,
                &date,
                sk.as_deref().unwrap_or_default(),
            )?;
            request_headers.insert("Authorization".to_string(), format!("Bearer {token}"));
            request_headers.insert("X-Kso-Date".to_string(), date);
            request_headers.insert("X-Kso-Authorization".to_string(), signature);
        }

        let mut req = client.request(method.clone(), &url);
        for (k, v) in &request_headers {
            req = req.header(k, v);
        }
        if let Some(body) = &opts.body {
            req = req.body(body.clone());
        }
        match req.send().await {
            Ok(resp) => {
                let status = resp.status().as_u16();
                let text = resp
                    .text()
                    .await
                    .map_err(|e| WpsError::Network(format!("failed to read response text: {e}")))?;
                let parsed_json = serde_json::from_str::<Value>(&text).unwrap_or_else(|_| {
                    serde_json::json!({
                        "raw_text": text
                    })
                });
                if matches!(auth_type, AuthType::User)
                    && !refreshed_once
                    && looks_like_user_token_expired(status, &text, &parsed_json)
                {
                    if let Ok(new_token) = auth::force_refresh_user_access_token().await {
                        token = new_token;
                        refreshed_once = true;
                        continue;
                    }
                }
                if opts.paginate {
                    return Ok(serde_json::json!({
                        "ok": status < 400,
                        "status": status,
                        "data": parsed_json,
                        "pagination": {"enabled": true, "note": "basic pagination mode currently returns first page only"}
                    }));
                }
                return Ok(serde_json::json!({
                    "ok": status < 400,
                    "status": status,
                    "data": parsed_json
                }));
            }
            Err(e) => {
                last_err = Some(e.to_string());
                if attempt < attempts {
                    tokio::time::sleep(std::time::Duration::from_millis(500 * attempt as u64)).await;
                }
            }
        }
    }
    Err(WpsError::Network(format!(
        "request failed after retries: {}",
        last_err.unwrap_or_else(|| "unknown error".to_string())
    )))
}

fn looks_like_user_token_expired(status: u16, raw_text: &str, json: &Value) -> bool {
    if status == 401 {
        return true;
    }
    if status == 403 {
        let lower = raw_text.to_ascii_lowercase();
        if lower.contains("token") && (lower.contains("expired") || lower.contains("invalid")) {
            return true;
        }
    }
    let lower = raw_text.to_ascii_lowercase();
    if lower.contains("invalid_token") || lower.contains("token expired") {
        return true;
    }

    let code = json
        .get("data")
        .and_then(|v| v.get("code"))
        .and_then(|v| v.as_i64())
        .unwrap_or_default();
    if code == 401 || code == 400001001 || code == 400001005 || code == 400001006 {
        return true;
    }
    let msg = json
        .get("data")
        .and_then(|v| v.get("msg"))
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    msg.contains("token") && (msg.contains("invalid") || msg.contains("expired"))
}

pub async fn execute_raw(
    method: &str,
    path_or_url: &str,
    query: HashMap<String, String>,
    headers: HashMap<String, String>,
    body: Option<String>,
    auth_type: &str,
    dry_run: bool,
    retry: u32,
) -> Result<Value, WpsError> {
    let auth_kind = AuthType::parse(auth_type);
    let base_url = if matches!(auth_kind, AuthType::Cookie) {
        auth::cookie_api_base()
    } else {
        "https://openapi.wps.cn".to_string()
    };
    let path = if path_or_url.starts_with("http://") || path_or_url.starts_with("https://") {
        path_or_url.to_string()
    } else {
        format!(
            "{}{}",
            base_url.trim_end_matches('/'),
            if path_or_url.starts_with('/') {
                path_or_url.to_string()
            } else {
                format!("/{path_or_url}")
            }
        )
    };
    let route = infer_endpoint_route(method, &path);
    let endpoint = EndpointDescriptor {
        id: "raw".to_string(),
        doc_id: None,
        name: route
            .as_ref()
            .map(|r| r.endpoint_id.clone())
            .unwrap_or_else(|| "raw".to_string()),
        summary: "".to_string(),
        http_method: method.to_uppercase(),
        path: reqwest::Url::parse(&path)
            .ok()
            .map(|u| u.path().to_string())
            .unwrap_or_else(|| path.clone()),
        signature: "KSO-1".to_string(),
        scopes: route
            .as_ref()
            .map(|r| r.scopes.clone())
            .unwrap_or_default(),
        params: Default::default(),
    };

    execute_endpoint(
        &endpoint,
        ExecOptions {
            path_params: HashMap::new(),
            query_params: query,
            headers,
            body,
            auth_type: auth_type.to_string(),
            dry_run,
            retry,
            paginate: false,
        },
    )
    .await
}
