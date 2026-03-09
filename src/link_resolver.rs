use crate::error::WpsError;
use crate::executor;

pub fn parse_share_link_id(url: &str) -> Option<String> {
    let normalized = if url.starts_with("http://") || url.starts_with("https://") {
        url.to_string()
    } else {
        format!("https://{url}")
    };
    let parsed = reqwest::Url::parse(&normalized).ok()?;
    let host = parsed.host_str()?.to_lowercase();
    if !host.ends_with("kdocs.cn") && host != "open.wps.cn" {
        return None;
    }
    let parts = parsed
        .path_segments()
        .map(|s| s.collect::<Vec<_>>())
        .unwrap_or_default();
    for i in 0..parts.len() {
        if parts[i] == "l" && i + 1 < parts.len() && !parts[i + 1].is_empty() {
            return Some(parts[i + 1].to_string());
        }
    }
    None
}

pub async fn resolve_share_link(
    url: &str,
    auth_type: &str,
    dry_run: bool,
    retry: u32,
) -> Result<serde_json::Value, WpsError> {
    let link_id = parse_share_link_id(url).ok_or_else(|| {
        WpsError::Validation(format!(
            "invalid share link `{url}`; expected format like https://365.kdocs.cn/l/<link_id>"
        ))
    })?;
    let resp = executor::execute_raw(
        "GET",
        &format!("/v7/links/{link_id}/meta"),
        std::collections::HashMap::new(),
        std::collections::HashMap::new(),
        None,
        auth_type,
        dry_run,
        retry,
    )
    .await?;
    let data = resp
        .get("data")
        .and_then(|v| v.get("data"))
        .cloned()
        .unwrap_or(serde_json::Value::Null);
    Ok(serde_json::json!({
        "ok": resp.get("ok").and_then(|v| v.as_bool()).unwrap_or(false),
        "status": resp.get("status").cloned().unwrap_or(serde_json::Value::Null),
        "link_id": link_id,
        "input_url": url,
        "file_id": data.get("file_id").cloned().unwrap_or(serde_json::Value::Null),
        "drive_id": data.get("drive_id").cloned().unwrap_or(serde_json::Value::Null),
        "link_status": data.get("status").cloned().unwrap_or(serde_json::Value::Null),
        "data": data,
        "raw": resp
    }))
}
