use std::collections::HashMap;

use serde_json::Value;

use crate::error::WpsError;
use crate::executor;

fn api_status_ok(v: &Value) -> bool {
    v.get("ok").and_then(|x| x.as_bool()).unwrap_or(false)
}

fn api_code(v: &Value) -> Option<i64> {
    v.get("data").and_then(|x| x.get("code")).and_then(|x| x.as_i64())
}

fn sheet_items_from_resp(resp: &Value) -> Vec<Value> {
    resp.get("data")
        .and_then(|v| v.get("data"))
        .and_then(|v| v.get("sheets"))
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default()
}

fn find_worksheet(items: &[Value], worksheet_id: Option<i64>) -> Option<Value> {
    if let Some(target) = worksheet_id {
        return items
            .iter()
            .find(|s| s.get("sheet_id").and_then(|v| v.as_i64()) == Some(target))
            .cloned();
    }
    items.first().cloned()
}

fn active_area_row_to(sheet: &Value) -> i64 {
    sheet
        .get("active_area")
        .and_then(|v| v.get("row_to"))
        .and_then(|v| v.as_i64())
        .unwrap_or(0)
}

fn active_area_col_from(sheet: &Value) -> i64 {
    sheet
        .get("active_area")
        .and_then(|v| v.get("col_from"))
        .and_then(|v| v.as_i64())
        .unwrap_or(0)
}

fn to_formula_text(v: &Value) -> String {
    match v {
        Value::Null => "".to_string(),
        Value::String(s) => s.clone(),
        Value::Number(n) => n.to_string(),
        Value::Bool(b) => {
            if *b {
                "TRUE".to_string()
            } else {
                "FALSE".to_string()
            }
        }
        _ => serde_json::to_string(v).unwrap_or_default(),
    }
}

fn extract_value_rows(input: Value) -> Result<Vec<Vec<Value>>, WpsError> {
    match input {
        Value::Array(arr) => {
            if arr.is_empty() {
                return Err(WpsError::Validation("xlsx 写入数据不能为空".to_string()));
            }
            if arr.first().and_then(|v| v.as_array()).is_some() {
                let mut rows = Vec::new();
                for r in arr {
                    let row = r
                        .as_array()
                        .cloned()
                        .ok_or_else(|| WpsError::Validation("xlsx values 必须是二维数组".to_string()))?;
                    rows.push(row);
                }
                Ok(rows)
            } else {
                Ok(vec![arr])
            }
        }
        Value::Object(mut obj) => {
            if let Some(v) = obj.remove("values") {
                return extract_value_rows(v);
            }
            Err(WpsError::Validation(
                "xlsx 写入 JSON 需为二维数组，或对象形如 {\"values\": [[...],[...]]}".to_string(),
            ))
        }
        _ => Err(WpsError::Validation(
            "xlsx 写入 JSON 需为二维数组，或对象形如 {\"values\": [[...],[...]]}".to_string(),
        )),
    }
}

async fn get_worksheets(
    file_id: &str,
    auth_type: &str,
    dry_run: bool,
    retry: u32,
) -> Result<Value, WpsError> {
    executor::execute_raw(
        "GET",
        &format!("/v7/sheets/{file_id}/worksheets"),
        HashMap::new(),
        HashMap::new(),
        None,
        auth_type,
        dry_run,
        retry,
    )
    .await
}

pub async fn read_xlsx_via_sheets(
    drive_id: &str,
    file_id: &str,
    auth_type: &str,
    dry_run: bool,
    retry: u32,
    max_sheets: u32,
    max_rows: u32,
    max_cols: u32,
    file_meta_resp: Value,
    read_mode: &str,
    trigger_response: Option<Value>,
) -> Result<Value, WpsError> {
    let sheets_resp = get_worksheets(file_id, auth_type, dry_run, retry).await?;
    if !api_status_ok(&sheets_resp) {
        return Ok(sheets_resp);
    }
    let sheet_items = sheet_items_from_resp(&sheets_resp);

    let mut sheets_out = Vec::new();
    for sheet in sheet_items.into_iter().take(max_sheets as usize) {
        let sheet_id = sheet.get("sheet_id").and_then(|v| v.as_i64());
        let name = sheet
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let area = sheet.get("active_area").cloned().unwrap_or(Value::Null);
        let row_from = area
            .get("row_from")
            .and_then(|v| v.as_i64())
            .unwrap_or(0)
            .max(0);
        let row_to = area
            .get("row_to")
            .and_then(|v| v.as_i64())
            .unwrap_or(row_from)
            .max(row_from);
        let col_from = area
            .get("col_from")
            .and_then(|v| v.as_i64())
            .unwrap_or(0)
            .max(0);
        let col_to = area
            .get("col_to")
            .and_then(|v| v.as_i64())
            .unwrap_or(col_from)
            .max(col_from);
        let capped_row_to = std::cmp::min(row_to, row_from + max_rows.saturating_sub(1) as i64);
        let capped_col_to = std::cmp::min(col_to, col_from + max_cols.saturating_sub(1) as i64);

        let mut query = HashMap::new();
        query.insert("row_from".to_string(), row_from.to_string());
        query.insert("row_to".to_string(), capped_row_to.to_string());
        query.insert("col_from".to_string(), col_from.to_string());
        query.insert("col_to".to_string(), capped_col_to.to_string());

        let range_resp = if let Some(id) = sheet_id {
            executor::execute_raw(
                "GET",
                &format!("/v7/sheets/{file_id}/worksheets/{id}/range_data"),
                query,
                HashMap::new(),
                None,
                auth_type,
                dry_run,
                retry,
            )
            .await?
        } else {
            serde_json::json!({"ok": false, "error": {"message":"sheet_id missing"}})
        };

        sheets_out.push(serde_json::json!({
            "sheet_id": sheet_id,
            "name": name,
            "active_area": area,
            "read_area": {
                "row_from": row_from,
                "row_to": capped_row_to,
                "col_from": col_from,
                "col_to": capped_col_to
            },
            "range_data": range_resp.get("data").and_then(|v| v.get("data")).and_then(|v| v.get("range_data")).cloned().unwrap_or(Value::Array(vec![])),
            "raw": range_resp
        }));
    }

    let mut out = serde_json::json!({
        "ok": true,
        "status": 200,
        "data": {
            "code": 0,
            "msg": "ok",
            "data": {
                "src_format": "xlsx",
                "read_mode": read_mode,
                "file_id": file_id,
                "drive_id": drive_id,
                "file_meta": file_meta_resp.get("data").and_then(|v| v.get("data")).cloned().unwrap_or(Value::Null),
                "sheets": sheets_out
            }
        }
    });
    if let Some(trigger) = trigger_response {
        out["fallback"] = serde_json::json!({
            "reason": "content_extract_failed",
            "trigger_code": api_code(&trigger),
            "trigger_response": trigger
        });
    }
    Ok(out)
}

pub async fn write_xlsx_values(
    file_id: &str,
    worksheet_id: Option<i64>,
    values_input: Value,
    row_from: Option<i64>,
    col_from: Option<i64>,
    write_mode: &str,
    auth_type: &str,
    dry_run: bool,
    retry: u32,
) -> Result<Value, WpsError> {
    let rows = extract_value_rows(values_input)?;
    let sheet_resp = get_worksheets(file_id, auth_type, dry_run, retry).await?;
    if !dry_run && !api_status_ok(&sheet_resp) {
        return Err(WpsError::Network(format!("获取工作表列表失败: {sheet_resp}")));
    }
    let items = sheet_items_from_resp(&sheet_resp);
    let sheet = find_worksheet(&items, worksheet_id);
    if sheet.is_none() && !dry_run {
        return Err(WpsError::Validation(
            "未找到可用 worksheet，请检查 --xlsx-sheet-id 或文件是否含工作表".to_string(),
        ));
    }
    let worksheet_id = sheet
        .as_ref()
        .and_then(|v| v.get("sheet_id"))
        .and_then(|v| v.as_i64())
        .or(worksheet_id)
        .unwrap_or(0);

    let base_row_from = match write_mode {
        "append" => row_from.unwrap_or_else(|| sheet.as_ref().map(active_area_row_to).unwrap_or(0) + 1),
        _ => row_from.unwrap_or(0),
    };
    let base_col_from = col_from.unwrap_or_else(|| sheet.as_ref().map(active_area_col_from).unwrap_or(0));

    let mut range_data = Vec::new();
    for (r_idx, row) in rows.iter().enumerate() {
        for (c_idx, cell) in row.iter().enumerate() {
            let r = base_row_from + r_idx as i64;
            let c = base_col_from + c_idx as i64;
            range_data.push(serde_json::json!({
                "row_from": r,
                "row_to": r,
                "col_from": c,
                "col_to": c,
                "op_type": "cell_operation_type_formula",
                "formula": to_formula_text(cell)
            }));
        }
    }
    let body = serde_json::json!({ "range_data": range_data });

    let resp = executor::execute_raw(
        "POST",
        &format!("/v7/sheets/{file_id}/worksheets/{worksheet_id}/range_data/batch_update"),
        HashMap::new(),
        HashMap::new(),
        Some(body.to_string()),
        auth_type,
        dry_run,
        retry,
    )
    .await?;

    Ok(serde_json::json!({
        "ok": resp.get("ok").cloned().unwrap_or(Value::Bool(dry_run)),
        "status": resp.get("status").cloned().unwrap_or(Value::Null),
        "data": {
            "code": api_code(&resp).unwrap_or(if dry_run {0} else {-1}),
            "msg": resp.get("data").and_then(|v| v.get("msg")).cloned().unwrap_or(Value::String("ok".to_string())),
            "data": {
                "target_format": "xlsx",
                "file_id": file_id,
                "worksheet_id": worksheet_id,
                "write_mode": write_mode,
                "start": {
                    "row_from": base_row_from,
                    "col_from": base_col_from
                },
                "row_count": rows.len(),
                "col_count": rows.first().map(|r| r.len()).unwrap_or(0),
                "result": resp.get("data").cloned().unwrap_or(Value::Null)
            }
        }
    }))
}
