use std::collections::HashMap;

use base64::Engine;
use clap::{Arg, Command};
use serde_json::Value;

use crate::error::WpsError;
use crate::executor;

pub fn command() -> Command {
    Command::new("airpage")
        .about("智能文档（Airpage）助手命令")
        .after_help("示例：\n  wpscli airpage query <file_id>")
        .subcommand(
            Command::new("query")
                .about("查询文件元信息（用于确认 Airpage 文件）")
                .arg(Arg::new("file-id").required(true).help("文件 ID")),
        )
}

pub async fn handle(args: &[String]) -> Result<serde_json::Value, WpsError> {
    let mut argv = vec!["airpage".to_string()];
    argv.extend_from_slice(args);
    let m = command()
        .try_get_matches_from(argv)
        .map_err(|e| WpsError::Validation(e.to_string()))?;
    match m.subcommand() {
        Some(("query", s)) => {
            let file_id = s.get_one::<String>("file-id").expect("required");
            executor::execute_raw(
                "GET",
                &format!("/v7/files/{file_id}/meta"),
                HashMap::new(),
                HashMap::new(),
                None,
                "user",
                false,
                1,
            )
            .await
        }
        _ => Err(WpsError::Validation("unknown airpage subcommand".to_string())),
    }
}

fn api_status_ok(v: &Value) -> bool {
    v.get("ok").and_then(|x| x.as_bool()).unwrap_or(false)
}

fn api_code(v: &Value) -> Option<i64> {
    v.get("data").and_then(|x| x.get("code")).and_then(|x| x.as_i64())
}

fn api_business_ok(v: &Value) -> bool {
    api_status_ok(v) && api_code(v).unwrap_or(0) == 0
}

fn decode_airpage_field(value: &Value) -> Value {
    if let Some(s) = value.as_str() {
        if let Ok(decoded) = base64::engine::general_purpose::STANDARD.decode(s) {
            if let Ok(parsed) = serde_json::from_slice::<Value>(&decoded) {
                return parsed;
            }
        }
        if let Ok(parsed) = serde_json::from_str::<Value>(s) {
            return parsed;
        }
    }
    value.clone()
}

fn extract_airpage_result(resp: &Value, key: &str) -> Value {
    let v = resp
        .get("data")
        .and_then(|x| x.get("data"))
        .and_then(|x| x.get(key))
        .cloned()
        .unwrap_or(Value::Null);
    decode_airpage_field(&v)
}

async fn post_blocks(file_id: &str, suffix: &str, arg: Value, auth_type: &str, dry_run: bool, retry: u32) -> Result<Value, WpsError> {
    let encoded = base64::engine::general_purpose::STANDARD.encode(arg.to_string().as_bytes());
    executor::execute_raw(
        "POST",
        &format!("/v7/airpage/{file_id}/blocks/{suffix}"),
        HashMap::new(),
        HashMap::new(),
        Some(serde_json::json!({ "arg": encoded }).to_string()),
        auth_type,
        dry_run,
        retry,
    )
    .await
}

async fn query_doc_blocks(file_id: &str, auth_type: &str, dry_run: bool, retry: u32) -> Result<Value, WpsError> {
    let encoded = base64::engine::general_purpose::STANDARD.encode(
        serde_json::json!({"blockId":"doc"}).to_string().as_bytes(),
    );
    executor::execute_raw(
        "POST",
        &format!("/v7/airpage/{file_id}/blocks"),
        HashMap::new(),
        HashMap::new(),
        Some(serde_json::json!({ "arg": encoded }).to_string()),
        auth_type,
        dry_run,
        retry,
    )
    .await
}

fn doc_children_from_query(resp: &Value) -> Vec<Value> {
    let result = extract_airpage_result(resp, "result");
    let blocks = result
        .get("blocks")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    for b in blocks {
        if b.get("id").and_then(|v| v.as_str()) == Some("doc")
            || b.get("type").and_then(|v| v.as_str()) == Some("doc")
        {
            return b
                .get("content")
                .and_then(|v| v.as_array())
                .cloned()
                .unwrap_or_default();
        }
    }
    Vec::new()
}

pub async fn write_markdown(
    file_id: &str,
    markdown: &str,
    write_mode: &str,
    auth_type: &str,
    dry_run: bool,
    retry: u32,
) -> Result<Value, WpsError> {
    if write_mode == "replace" {
        let q = query_doc_blocks(file_id, auth_type, dry_run, retry).await?;
        if !dry_run && !api_business_ok(&q) {
            return Err(WpsError::Network(format!("airpage query failed: {q}")));
        }
        let children = doc_children_from_query(&q);
        let count = children.len();
        if count > 1 {
            let del = post_blocks(
                file_id,
                "delete",
                serde_json::json!({"blockId":"doc","startIndex":1,"endIndex":count}),
                auth_type,
                dry_run,
                retry,
            )
            .await?;
            if !dry_run && !api_business_ok(&del) {
                return Err(WpsError::Network(format!("airpage delete failed: {del}")));
            }
        }
    }

    let conv = post_blocks(
        file_id,
        "convert",
        serde_json::json!({"format":"markdown","content": markdown}),
        auth_type,
        dry_run,
        retry,
    )
    .await?;
    if !dry_run && !api_business_ok(&conv) {
        return Err(WpsError::Network(format!("airpage convert failed: {conv}")));
    }
    let convert_result = extract_airpage_result(&conv, "result");
    let blocks = convert_result
        .get("blocks")
        .cloned()
        .unwrap_or(Value::Array(vec![]));
    let index = if write_mode == "append" { 999_999 } else { 1 };
    let create = post_blocks(
        file_id,
        "create",
        serde_json::json!({
            "blockId":"doc",
            "index": index,
            "content": blocks
        }),
        auth_type,
        dry_run,
        retry,
    )
    .await?;
    if !dry_run && !api_business_ok(&create) {
        return Err(WpsError::Network(format!("airpage create failed: {create}")));
    }

    Ok(serde_json::json!({
        "ok": true,
        "status": 200,
        "data": {
            "code": 0,
            "msg": "ok",
            "data": {
                "write_mode": write_mode,
                "target_format": "otl",
                "file_id": file_id,
                "result": create.get("data").cloned().unwrap_or(Value::Null)
            }
        }
    }))
}
