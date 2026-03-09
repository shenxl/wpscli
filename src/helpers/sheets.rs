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

fn excel_col_label(mut col_zero_based: i64) -> String {
    if col_zero_based < 0 {
        return "".to_string();
    }
    let mut s = String::new();
    loop {
        let rem = (col_zero_based % 26) as u8;
        s.insert(0, (b'A' + rem) as char);
        col_zero_based = (col_zero_based / 26) - 1;
        if col_zero_based < 0 {
            break;
        }
    }
    s
}

fn cell_value_for_table(cell: &Value) -> Value {
    if let Some(v) = cell.get("original_cell_value") {
        if !v.is_null() {
            return v.clone();
        }
    }
    if let Some(v) = cell.get("cell_text") {
        return v.clone();
    }
    Value::Null
}

fn value_to_non_empty_header(v: &Value) -> Option<String> {
    match v {
        Value::String(s) => {
            let x = s.trim();
            if x.is_empty() {
                None
            } else {
                Some(x.to_string())
            }
        }
        Value::Number(n) => Some(n.to_string()),
        Value::Bool(b) => Some(if *b { "true".to_string() } else { "false".to_string() }),
        _ => None,
    }
}

fn build_table_from_range_data(
    range_data: &[Value],
    row_from: i64,
    row_to: i64,
    col_from: i64,
    col_to: i64,
) -> Value {
    if row_to < row_from || col_to < col_from {
        return serde_json::json!({
            "row_count": 0,
            "col_count": 0,
            "rows": [],
            "records": []
        });
    }

    let row_count = (row_to - row_from + 1) as usize;
    let col_count = (col_to - col_from + 1) as usize;
    let mut rows = vec![vec![Value::Null; col_count]; row_count];
    let mut non_empty_cells = 0usize;

    for cell in range_data {
        let r1 = cell.get("row_from").and_then(|v| v.as_i64()).unwrap_or(row_from);
        let r2 = cell.get("row_to").and_then(|v| v.as_i64()).unwrap_or(r1);
        let c1 = cell.get("col_from").and_then(|v| v.as_i64()).unwrap_or(col_from);
        let c2 = cell.get("col_to").and_then(|v| v.as_i64()).unwrap_or(c1);
        let value = cell_value_for_table(cell);
        for r in r1..=r2 {
            for c in c1..=c2 {
                if r < row_from || r > row_to || c < col_from || c > col_to {
                    continue;
                }
                let rr = (r - row_from) as usize;
                let cc = (c - col_from) as usize;
                if rows[rr][cc].is_null() && !value.is_null() {
                    non_empty_cells += 1;
                }
                rows[rr][cc] = value.clone();
            }
        }
    }

    let mut columns = Vec::with_capacity(col_count);
    for c in col_from..=col_to {
        columns.push(serde_json::json!({
            "col_index": c,
            "col_key": format!("col_{c}"),
            "col_label": excel_col_label(c)
        }));
    }

    let mut records = Vec::with_capacity(row_count);
    for row in &rows {
        let mut obj = serde_json::Map::new();
        for (idx, v) in row.iter().enumerate() {
            obj.insert(format!("col_{}", col_from + idx as i64), v.clone());
        }
        records.push(Value::Object(obj));
    }

    let header_row_candidate = rows.first().cloned().unwrap_or_default();
    let header_non_empty = header_row_candidate
        .iter()
        .filter_map(value_to_non_empty_header)
        .count();
    let header_inferred = header_non_empty >= 2;

    let mut header_keys = Vec::new();
    let mut seen = HashMap::<String, usize>::new();
    for (idx, col) in header_row_candidate.iter().enumerate() {
        let fallback = format!("col_{}", col_from + idx as i64);
        let base = value_to_non_empty_header(col).unwrap_or(fallback);
        let count = seen.entry(base.clone()).or_insert(0);
        *count += 1;
        let key = if *count == 1 {
            base
        } else {
            format!("{}_{}", base, count)
        };
        header_keys.push(key);
    }

    let mut records_with_header = Vec::new();
    if header_inferred && rows.len() > 1 {
        for row in rows.iter().skip(1) {
            let mut obj = serde_json::Map::new();
            for (idx, v) in row.iter().enumerate() {
                let k = header_keys
                    .get(idx)
                    .cloned()
                    .unwrap_or_else(|| format!("col_{}", col_from + idx as i64));
                obj.insert(k, v.clone());
            }
            records_with_header.push(Value::Object(obj));
        }
    }

    serde_json::json!({
        "row_from": row_from,
        "row_to": row_to,
        "col_from": col_from,
        "col_to": col_to,
        "row_count": row_count,
        "col_count": col_count,
        "non_empty_cells": non_empty_cells,
        "columns": columns,
        "rows": rows,
        "records": records,
        "header_row_candidate": header_row_candidate,
        "header_inferred": header_inferred,
        "header_keys": header_keys,
        "records_with_header": records_with_header
    })
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
    sheet_offset: u32,
    sheet_head: u32,
    row_offset: u32,
    row_head: u32,
    col_offset: u32,
    col_head: u32,
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
    for sheet in sheet_items
        .into_iter()
        .skip(sheet_offset as usize)
        .take(sheet_head as usize)
    {
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
        let read_row_from = std::cmp::min(row_to, row_from + row_offset as i64);
        let read_col_from = std::cmp::min(col_to, col_from + col_offset as i64);
        let read_row_to = if row_head == 0 || read_row_from > row_to {
            read_row_from - 1
        } else {
            std::cmp::min(row_to, read_row_from + row_head.saturating_sub(1) as i64)
        };
        let read_col_to = if col_head == 0 || read_col_from > col_to {
            read_col_from - 1
        } else {
            std::cmp::min(col_to, read_col_from + col_head.saturating_sub(1) as i64)
        };

        let range_resp = if read_row_to < read_row_from || read_col_to < read_col_from {
            serde_json::json!({"ok": true, "status": 200, "data": {"code": 0, "msg": "", "data": {"range_data": []}}})
        } else if let Some(id) = sheet_id {
            let mut query = HashMap::new();
            query.insert("row_from".to_string(), read_row_from.to_string());
            query.insert("row_to".to_string(), read_row_to.to_string());
            query.insert("col_from".to_string(), read_col_from.to_string());
            query.insert("col_to".to_string(), read_col_to.to_string());
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

        let range_data = range_resp
            .get("data")
            .and_then(|v| v.get("data"))
            .and_then(|v| v.get("range_data"))
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();
        let table = build_table_from_range_data(
            &range_data,
            read_row_from,
            read_row_to,
            read_col_from,
            read_col_to,
        );

        sheets_out.push(serde_json::json!({
            "sheet_id": sheet_id,
            "name": name,
            "active_area": area,
            "read_area": {
                "row_from": read_row_from,
                "row_to": read_row_to,
                "col_from": read_col_from,
                "col_to": read_col_to
            },
            "table": table,
            "range_data": range_data,
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
                "paging": {
                    "sheet_offset": sheet_offset,
                    "sheet_head": sheet_head,
                    "row_offset": row_offset,
                    "row_head": row_head,
                    "col_offset": col_offset,
                    "col_head": col_head
                },
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
