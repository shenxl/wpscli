use std::collections::HashMap;

use reqwest::{Client, Method};
use serde_json::Value;

use crate::auth::{self, AuthType};
use crate::descriptor::EndpointDescriptor;
use crate::error::WpsError;
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

fn build_url(endpoint: &EndpointDescriptor, path_params: &HashMap<String, String>) -> Result<String, WpsError> {
    let mut path = endpoint.path.clone();
    for p in &endpoint.params.path {
        if let Some(v) = path_params.get(&p.name) {
            path = path.replace(&format!("{{{}}}", p.name), &encode_path_segment(v));
        } else if p.required {
            return Err(WpsError::Validation(format!("missing required path param {}", p.name)));
        }
    }
    Ok(format!("https://openapi.wps.cn{path}"))
}

pub async fn execute_endpoint(endpoint: &EndpointDescriptor, opts: ExecOptions) -> Result<Value, WpsError> {
    let mut url = build_url(endpoint, &opts.path_params)?;
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
    let auth_type = AuthType::parse(&opts.auth_type);
    let mut token = auth::get_access_token(auth_type).await?;
    let sk = auth::current_sk()?;

    let client = Client::new();
    let attempts = std::cmp::max(1, opts.retry + 1);
    let mut last_err = None;
    let mut refreshed_once = false;
    for attempt in 1..=attempts {
        let date = auth::rfc1123_now();
        let signature = auth::generate_kso1_signature(endpoint.http_method.as_str(), &url, &date, &sk)?;
        let mut request_headers = opts.headers.clone();
        request_headers.insert("Authorization".to_string(), format!("Bearer {token}"));
        request_headers.insert("X-Kso-Date".to_string(), date);
        request_headers.insert("X-Kso-Authorization".to_string(), signature);
        request_headers
            .entry("Content-Type".to_string())
            .or_insert_with(|| "application/json".to_string());

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
    let path = if path_or_url.starts_with("http://") || path_or_url.starts_with("https://") {
        path_or_url.to_string()
    } else {
        format!(
            "https://openapi.wps.cn{}",
            if path_or_url.starts_with('/') {
                path_or_url.to_string()
            } else {
                format!("/{path_or_url}")
            }
        )
    };
    let endpoint = EndpointDescriptor {
        id: "raw".to_string(),
        doc_id: None,
        name: "raw".to_string(),
        summary: "".to_string(),
        http_method: method.to_uppercase(),
        path: path.trim_start_matches("https://openapi.wps.cn").to_string(),
        signature: "KSO-1".to_string(),
        scopes: vec![],
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
