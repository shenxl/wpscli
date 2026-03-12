use std::collections::{BTreeMap, HashMap};

use clap::{Arg, ArgAction, ArgMatches, Command};
use serde::Deserialize;
use serde_json::{Map, Value};

use crate::error::WpsError;
use crate::executor;
use crate::link_resolver;
use crate::skill_runtime;

const DEFAULT_FIELD_NAMES: [&str; 4] = ["名称", "数量", "日期", "状态"];
const DB_READ_SCOPES: [&str; 1] = ["kso.dbsheet.read"];
const DB_RW_SCOPES: [&str; 2] = ["kso.dbsheet.read", "kso.dbsheet.readwrite"];

#[derive(Debug, Deserialize)]
struct DbSchemaDoc {
    sheets: BTreeMap<String, DbSchemaSheet>,
}

#[derive(Debug, Deserialize)]
struct DbSchemaSheet {
    title: Option<String>,
    auto_clean: Option<bool>,
    fields: Vec<DbSchemaField>,
    views: Option<Vec<DbSchemaView>>,
}

#[derive(Debug, Deserialize)]
struct DbSchemaField {
    name: String,
    #[serde(rename = "type")]
    field_type: String,
    data: Option<Value>,
}

#[derive(Debug, Deserialize)]
struct DbSchemaView {
    name: Option<String>,
    #[serde(rename = "type")]
    view_type: Option<String>,
}

#[derive(Debug, Clone, Copy)]
enum LogicOp {
    And,
    Or,
}

#[derive(Debug, Clone)]
enum FilterExpr {
    Group { mode: LogicOp, items: Vec<FilterExpr> },
    Cond(Condition),
}

#[derive(Debug, Clone)]
struct Condition {
    field: String,
    op: ConditionOp,
    values: Vec<String>,
}

#[derive(Debug, Clone, Copy)]
enum ConditionOp {
    Eq,
    Ne,
    Gt,
    Lt,
    Ge,
    Le,
    In,
    Contains,
    StartsWith,
    EndsWith,
}

#[derive(Debug, Clone, Copy)]
pub enum DocDbAction {
    Create,
    Update,
    Delete,
}

impl DocDbAction {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Create => "create",
            Self::Update => "update",
            Self::Delete => "delete",
        }
    }
}

pub fn command() -> Command {
    Command::new("dbsheet")
        .about("多维表 SQL-like 助手（schema/init/select/insert/update/delete）")
        .after_help(
            "示例：\n  \
             wpscli dbsheet schema --url \"https://365.kdocs.cn/l/xxxx\" --user-token\n  \
             wpscli dbsheet select --url \"https://365.kdocs.cn/l/xxxx\" --sheet-id 2 --where \"状态 = '进行中'\" --fields \"状态,负责人\" --user-token\n  \
             wpscli dbsheet insert --url \"https://365.kdocs.cn/l/xxxx\" --sheet-id 2 --data-json '[{\"标题\":\"A\"}]' --user-token",
        )
        .subcommand(with_common_opts(
            with_file_args(Command::new("schema").about("获取多维表 schema（支持 --url 或 --file-id）"),
                false),
        ))
        .subcommand(with_common_opts(
            with_file_args(Command::new("list-sheets").about("列出所有工作表"), false),
        ))
        .subcommand(with_common_opts(
            with_file_args(
                Command::new("init")
                    .about("基于 schema 文件初始化多维表结构")
                    .arg(Arg::new("schema").long("schema").required(true).num_args(1).help("YAML schema 文件路径"))
                    .arg(Arg::new("sheet-key").long("sheet-key").num_args(1).help("仅初始化指定 sheet key"))
                    .arg(Arg::new("force-recreate").long("force-recreate").action(ArgAction::SetTrue).help("强制重建（危险操作）")),
                false,
            ),
        ))
        .subcommand(with_common_opts(
            with_file_args(
                Command::new("select")
                    .about("SQL-like 查询记录")
                    .arg(Arg::new("sheet-id").long("sheet-id").required(true).num_args(1).help("工作表 ID"))
                    .arg(Arg::new("where").long("where").num_args(1).help("过滤条件，如: 状态 = '进行中'"))
                    .arg(Arg::new("fields").long("fields").num_args(1).help("逗号分隔字段列表"))
                    .arg(
                        Arg::new("limit")
                            .long("limit")
                            .default_value("100")
                            .value_parser(clap::value_parser!(usize))
                            .help("最多返回条数"),
                    )
                    .arg(
                        Arg::new("offset")
                            .long("offset")
                            .default_value("0")
                            .value_parser(clap::value_parser!(usize))
                            .help("结果偏移量"),
                    )
                    .arg(
                        Arg::new("page-size")
                            .long("page-size")
                            .default_value("100")
                            .value_parser(clap::value_parser!(usize))
                            .help("分页抓取时的每页大小"),
                    ),
                false,
            ),
        ))
        .subcommand(with_common_opts(
            with_file_args(
                Command::new("insert")
                    .about("SQL-like 插入记录")
                    .arg(Arg::new("sheet-id").long("sheet-id").required(true).num_args(1).help("工作表 ID"))
                    .arg(Arg::new("data-json").long("data-json").num_args(1).help("插入数据 JSON"))
                    .arg(Arg::new("data-file").long("data-file").num_args(1).help("从文件读取插入数据 JSON"))
                    .arg(
                        Arg::new("batch-size")
                            .long("batch-size")
                            .default_value("100")
                            .value_parser(clap::value_parser!(usize))
                            .help("批量写入大小"),
                    )
                    .arg(Arg::new("prefer-id").long("prefer-id").action(ArgAction::SetTrue).help("优先使用 id 字段映射")),
                false,
            ),
        ))
        .subcommand(with_common_opts(
            with_file_args(
                Command::new("update")
                    .about("SQL-like 更新记录")
                    .arg(Arg::new("sheet-id").long("sheet-id").required(true).num_args(1).help("工作表 ID"))
                    .arg(Arg::new("record-id").long("record-id").num_args(1).help("单条记录 ID"))
                    .arg(Arg::new("data-json").long("data-json").num_args(1).help("更新数据 JSON"))
                    .arg(Arg::new("data-file").long("data-file").num_args(1).help("从文件读取更新数据 JSON"))
                    .arg(
                        Arg::new("batch-size")
                            .long("batch-size")
                            .default_value("100")
                            .value_parser(clap::value_parser!(usize))
                            .help("批量更新大小"),
                    )
                    .arg(Arg::new("prefer-id").long("prefer-id").action(ArgAction::SetTrue).help("优先使用 id 字段映射")),
                false,
            ),
        ))
        .subcommand(with_common_opts(
            with_file_args(
                Command::new("delete")
                    .about("SQL-like 删除记录（record-id/record-ids/where 三选一）")
                    .arg(Arg::new("sheet-id").long("sheet-id").required(true).num_args(1).help("工作表 ID"))
                    .arg(Arg::new("record-id").long("record-id").num_args(1).help("单条记录 ID"))
                    .arg(Arg::new("record-ids").long("record-ids").num_args(1).help("多条记录 ID，逗号分隔"))
                    .arg(Arg::new("where").long("where").num_args(1).help("按条件删除"))
                    .arg(
                        Arg::new("limit")
                            .long("limit")
                            .default_value("100")
                            .value_parser(clap::value_parser!(usize))
                            .help("where 删除时最多选中条数"),
                    )
                    .arg(
                        Arg::new("batch-size")
                            .long("batch-size")
                            .default_value("100")
                            .value_parser(clap::value_parser!(usize))
                            .help("批量删除大小"),
                    ),
                false,
            ),
        ))
        .subcommand(with_common_opts(
            with_file_args(
                Command::new("view-list")
                    .about("列出指定工作表视图（内化 dbsheet 视图 API）")
                    .arg(Arg::new("sheet-id").long("sheet-id").required(true).num_args(1).help("工作表 ID")),
                false,
            ),
        ))
        .subcommand(with_common_opts(
            with_file_args(
                Command::new("view-get")
                    .about("获取单个视图详情（内化 dbsheet 视图 API）")
                    .arg(Arg::new("sheet-id").long("sheet-id").required(true).num_args(1).help("工作表 ID"))
                    .arg(Arg::new("view-id").long("view-id").required(true).num_args(1).help("视图 ID")),
                false,
            ),
        ))
        .subcommand(with_common_opts(
            with_file_args(
                Command::new("view-create")
                    .about("创建视图（通过结构化入参，不暴露底层路径）")
                    .arg(Arg::new("sheet-id").long("sheet-id").required(true).num_args(1).help("工作表 ID"))
                    .arg(Arg::new("data-json").long("data-json").num_args(1).help("视图创建 payload JSON"))
                    .arg(Arg::new("data-file").long("data-file").num_args(1).help("从文件读取视图创建 payload JSON")),
                false,
            ),
        ))
        .subcommand(with_common_opts(
            with_file_args(
                Command::new("view-update")
                    .about("更新视图（通过结构化入参，不暴露底层路径）")
                    .arg(Arg::new("sheet-id").long("sheet-id").required(true).num_args(1).help("工作表 ID"))
                    .arg(Arg::new("view-id").long("view-id").required(true).num_args(1).help("视图 ID"))
                    .arg(Arg::new("data-json").long("data-json").num_args(1).help("视图更新 payload JSON"))
                    .arg(Arg::new("data-file").long("data-file").num_args(1).help("从文件读取视图更新 payload JSON")),
                false,
            ),
        ))
        .subcommand(with_common_opts(
            with_file_args(
                Command::new("view-delete")
                    .about("删除视图（通过结构化入参，不暴露底层路径）")
                    .arg(Arg::new("sheet-id").long("sheet-id").required(true).num_args(1).help("工作表 ID"))
                    .arg(Arg::new("view-id").long("view-id").required(true).num_args(1).help("视图 ID")),
                false,
            ),
        ))
        .subcommand(with_common_opts(
            with_file_args(
                Command::new("webhook-list")
                    .about("列出 webhook（内化 dbsheet webhook API）")
                    .arg(Arg::new("with-detail").long("with-detail").action(ArgAction::SetTrue).help("是否返回规则详情")),
                false,
            ),
        ))
        .subcommand(with_common_opts(
            with_file_args(
                Command::new("webhook-create")
                    .about("创建 webhook（通过结构化入参，不暴露底层路径）")
                    .arg(Arg::new("data-json").long("data-json").num_args(1).help("webhook 创建 payload JSON"))
                    .arg(Arg::new("data-file").long("data-file").num_args(1).help("从文件读取 webhook 创建 payload JSON")),
                false,
            ),
        ))
        .subcommand(with_common_opts(
            with_file_args(
                Command::new("webhook-delete")
                    .about("删除 webhook（通过结构化入参，不暴露底层路径）")
                    .arg(Arg::new("hook-id").long("hook-id").required(true).num_args(1).help("hook ID")),
                false,
            ),
        ))
        .subcommand(with_common_opts(
            with_file_args(
                Command::new("clean")
                    .about("清理默认字段和默认空行")
                    .arg(Arg::new("sheet-id").long("sheet-id").required(true).num_args(1).help("工作表 ID")),
                false,
            ),
        ))
}

fn with_file_args(cmd: Command, _need_drive: bool) -> Command {
    cmd.arg(Arg::new("url").long("url").num_args(1).help("多维表分享链接"))
        .arg(Arg::new("file-id").long("file-id").num_args(1).help("多维表 file_id"))
}

fn with_common_opts(cmd: Command) -> Command {
    cmd.arg(
        Arg::new("auth-type")
            .long("auth-type")
            .value_parser(["app", "user", "cookie"])
            .default_value("user")
            .help("鉴权类型：app / user / cookie"),
    )
    .arg(Arg::new("user-token").long("user-token").action(ArgAction::SetTrue).help("快捷方式：等价于 --auth-type user"))
    .arg(Arg::new("dry-run").long("dry-run").action(ArgAction::SetTrue))
    .arg(
        Arg::new("retry")
            .long("retry")
            .default_value("1")
            .value_parser(clap::value_parser!(u32))
            .help("网络错误重试次数"),
    )
}

pub async fn handle(args: &[String]) -> Result<Value, WpsError> {
    let mut argv = vec!["dbsheet".to_string()];
    argv.extend_from_slice(args);
    let m = command()
        .try_get_matches_from(argv)
        .map_err(|e| WpsError::Validation(e.to_string()))?;
    match m.subcommand() {
        Some(("schema", s)) => get_schema_cmd(s).await,
        Some(("list-sheets", s)) => list_sheets_cmd(s).await,
        Some(("init", s)) => init_cmd(s).await,
        Some(("select", s)) => select_cmd(s).await,
        Some(("insert", s)) => insert_cmd(s).await,
        Some(("update", s)) => update_cmd(s).await,
        Some(("delete", s)) => delete_cmd(s).await,
        Some(("view-list", s)) => view_list_cmd(s).await,
        Some(("view-get", s)) => view_get_cmd(s).await,
        Some(("view-create", s)) => view_create_cmd(s).await,
        Some(("view-update", s)) => view_update_cmd(s).await,
        Some(("view-delete", s)) => view_delete_cmd(s).await,
        Some(("webhook-list", s)) => webhook_list_cmd(s).await,
        Some(("webhook-create", s)) => webhook_create_cmd(s).await,
        Some(("webhook-delete", s)) => webhook_delete_cmd(s).await,
        Some(("clean", s)) => clean_cmd(s).await,
        _ => Err(WpsError::Validation("unknown dbsheet subcommand".to_string())),
    }
}

fn read_json_input(raw: &str, label: &str) -> Result<Value, WpsError> {
    let input = raw.trim();
    if let Some(path) = input.strip_prefix('@') {
        let data = std::fs::read_to_string(path)
            .map_err(|e| WpsError::Validation(format!("读取 {label} 文件失败: {e}")))?;
        return serde_json::from_str(&data)
            .map_err(|e| WpsError::Validation(format!("{label} JSON 解析失败: {e}")));
    }
    serde_json::from_str(input).map_err(|e| WpsError::Validation(format!("{label} JSON 解析失败: {e}")))
}

fn read_payload_arg(s: &ArgMatches, inline_key: &str, file_key: &str) -> Result<Value, WpsError> {
    if let Some(v) = s.get_one::<String>(inline_key) {
        return read_json_input(v, inline_key);
    }
    if let Some(path) = s.get_one::<String>(file_key) {
        let raw = std::fs::read_to_string(path)
            .map_err(|e| WpsError::Validation(format!("读取 {file_key} 失败: {e}")))?;
        return serde_json::from_str::<Value>(&raw)
            .map_err(|e| WpsError::Validation(format!("{file_key} JSON 解析失败: {e}")));
    }
    Err(WpsError::Validation(format!(
        "缺少 --{inline_key} 或 --{file_key}"
    )))
}

fn effective_auth_type(s: &ArgMatches) -> String {
    if s.get_flag("user-token") {
        "user".to_string()
    } else {
        s.get_one::<String>("auth-type")
            .cloned()
            .unwrap_or_else(|| "user".to_string())
    }
}

async fn ensure_scope(auth: &str, dry: bool, write: bool) -> Result<Value, WpsError> {
    let required = if write { &DB_RW_SCOPES[..] } else { &DB_READ_SCOPES[..] };
    let pf = skill_runtime::ensure_user_scope(auth, required, dry).await?;
    Ok(pf.to_json())
}

async fn resolve_file_id(s: &ArgMatches, auth: &str, dry: bool, retry: u32) -> Result<String, WpsError> {
    if let Some(fid) = s.get_one::<String>("file-id") {
        if !fid.trim().is_empty() {
            return Ok(fid.to_string());
        }
    }
    let url = s
        .get_one::<String>("url")
        .ok_or_else(|| WpsError::Validation("缺少 --file-id 或 --url".to_string()))?;
    let resolved = link_resolver::resolve_share_link(url, auth, dry, retry).await?;
    resolved
        .get("file_id")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| WpsError::Validation("链接解析结果缺少 file_id".to_string()))
}

async fn execute(
    method: &str,
    path: &str,
    body: Option<Value>,
    auth: &str,
    dry: bool,
    retry: u32,
) -> Result<Value, WpsError> {
    executor::execute_raw(
        method,
        path,
        HashMap::new(),
        HashMap::new(),
        body.map(|b| b.to_string()),
        auth,
        dry,
        retry,
    )
    .await
}

fn api_ok(v: &Value) -> bool {
    if !v.get("ok").and_then(|x| x.as_bool()).unwrap_or(false) {
        return false;
    }
    let data = v.get("data").cloned().unwrap_or(Value::Null);
    if let Some(code) = data.get("code").and_then(|x| x.as_i64()) {
        return code == 0;
    }
    if let Some(ok) = data.get("ok").and_then(|x| x.as_bool()) {
        return ok;
    }
    true
}

fn payload(v: &Value) -> Value {
    v.get("data")
        .and_then(|x| x.get("data"))
        .cloned()
        .unwrap_or(Value::Null)
}

fn parse_sheet_id(s: &ArgMatches) -> Result<String, WpsError> {
    s.get_one::<String>("sheet-id")
        .cloned()
        .ok_or_else(|| WpsError::Validation("缺少 --sheet-id".to_string()))
}

async fn get_schema_cmd(s: &ArgMatches) -> Result<Value, WpsError> {
    let auth = effective_auth_type(s);
    let dry = s.get_flag("dry-run");
    let retry = *s.get_one::<u32>("retry").unwrap_or(&1);
    let scope = ensure_scope(&auth, dry, false).await?;
    let file_id = resolve_file_id(s, &auth, dry, retry).await?;
    let resp = execute(
        "GET",
        &format!("/v7/coop/dbsheet/{file_id}/schema"),
        None,
        &auth,
        dry,
        retry,
    )
    .await?;
    Ok(serde_json::json!({
        "ok": api_ok(&resp),
        "status": resp.get("status").cloned().unwrap_or(Value::Null),
        "data": {
            "code": if api_ok(&resp) { 0 } else { -1 },
            "msg": resp.get("data").and_then(|v| v.get("msg")).cloned().unwrap_or(Value::Null),
            "data": {
                "file_id": file_id,
                "scope_preflight": scope,
                "schema": payload(&resp)
            }
        }
    }))
}

async fn list_sheets_cmd(s: &ArgMatches) -> Result<Value, WpsError> {
    let mut out = get_schema_cmd(s).await?;
    let sheets = out
        .get("data")
        .and_then(|v| v.get("data"))
        .and_then(|v| v.get("schema"))
        .and_then(|v| v.get("sheets"))
        .cloned()
        .unwrap_or(Value::Array(vec![]));
    out["data"]["data"] = serde_json::json!({
        "file_id": out["data"]["data"]["file_id"].clone(),
        "scope_preflight": out["data"]["data"]["scope_preflight"].clone(),
        "sheets": sheets
    });
    Ok(out)
}

async fn view_list_cmd(s: &ArgMatches) -> Result<Value, WpsError> {
    let auth = effective_auth_type(s);
    let dry = s.get_flag("dry-run");
    let retry = *s.get_one::<u32>("retry").unwrap_or(&1);
    let scope = ensure_scope(&auth, dry, false).await?;
    let file_id = resolve_file_id(s, &auth, dry, retry).await?;
    let sheet_id = parse_sheet_id(s)?;
    let resp = execute(
        "GET",
        &format!("/v7/dbsheet/{file_id}/sheets/{sheet_id}/views"),
        None,
        &auth,
        dry,
        retry,
    )
    .await?;
    if !dry && !api_ok(&resp) {
        return Err(WpsError::Network(format!("列出视图失败: {resp}")));
    }
    Ok(serde_json::json!({
        "ok": true,
        "msg": "ok",
        "data": {
            "scope_preflight": scope,
            "action": "view-list",
            "file_id": file_id,
            "sheet_id": sheet_id,
            "views": payload(&resp),
        }
    }))
}

async fn view_get_cmd(s: &ArgMatches) -> Result<Value, WpsError> {
    let auth = effective_auth_type(s);
    let dry = s.get_flag("dry-run");
    let retry = *s.get_one::<u32>("retry").unwrap_or(&1);
    let scope = ensure_scope(&auth, dry, false).await?;
    let file_id = resolve_file_id(s, &auth, dry, retry).await?;
    let sheet_id = parse_sheet_id(s)?;
    let view_id = s
        .get_one::<String>("view-id")
        .cloned()
        .ok_or_else(|| WpsError::Validation("缺少 --view-id".to_string()))?;
    let resp = execute(
        "GET",
        &format!("/v7/dbsheet/{file_id}/sheets/{sheet_id}/views/{view_id}"),
        None,
        &auth,
        dry,
        retry,
    )
    .await?;
    if !dry && !api_ok(&resp) {
        return Err(WpsError::Network(format!("读取视图失败: {resp}")));
    }
    Ok(serde_json::json!({
        "ok": true,
        "msg": "ok",
        "data": {
            "scope_preflight": scope,
            "action": "view-get",
            "file_id": file_id,
            "sheet_id": sheet_id,
            "view_id": view_id,
            "view": payload(&resp),
        }
    }))
}

async fn view_create_cmd(s: &ArgMatches) -> Result<Value, WpsError> {
    let auth = effective_auth_type(s);
    let dry = s.get_flag("dry-run");
    let retry = *s.get_one::<u32>("retry").unwrap_or(&1);
    let scope = ensure_scope(&auth, dry, true).await?;
    let file_id = resolve_file_id(s, &auth, dry, retry).await?;
    let sheet_id = parse_sheet_id(s)?;
    let body = read_payload_arg(s, "data-json", "data-file")?;
    let resp = execute(
        "POST",
        &format!("/v7/coop/dbsheet/{file_id}/sheets/{sheet_id}/views"),
        Some(body),
        &auth,
        dry,
        retry,
    )
    .await?;
    if !dry && !api_ok(&resp) {
        return Err(WpsError::Network(format!("创建视图失败: {resp}")));
    }
    Ok(serde_json::json!({
        "ok": true,
        "msg": "ok",
        "data": {
            "scope_preflight": scope,
            "action": "view-create",
            "file_id": file_id,
            "sheet_id": sheet_id,
            "result": payload(&resp),
        }
    }))
}

async fn view_update_cmd(s: &ArgMatches) -> Result<Value, WpsError> {
    let auth = effective_auth_type(s);
    let dry = s.get_flag("dry-run");
    let retry = *s.get_one::<u32>("retry").unwrap_or(&1);
    let scope = ensure_scope(&auth, dry, true).await?;
    let file_id = resolve_file_id(s, &auth, dry, retry).await?;
    let sheet_id = parse_sheet_id(s)?;
    let view_id = s
        .get_one::<String>("view-id")
        .cloned()
        .ok_or_else(|| WpsError::Validation("缺少 --view-id".to_string()))?;
    let body = read_payload_arg(s, "data-json", "data-file")?;
    let resp = execute(
        "POST",
        &format!("/v7/coop/dbsheet/{file_id}/sheets/{sheet_id}/views/{view_id}/update"),
        Some(body),
        &auth,
        dry,
        retry,
    )
    .await?;
    if !dry && !api_ok(&resp) {
        return Err(WpsError::Network(format!("更新视图失败: {resp}")));
    }
    Ok(serde_json::json!({
        "ok": true,
        "msg": "ok",
        "data": {
            "scope_preflight": scope,
            "action": "view-update",
            "file_id": file_id,
            "sheet_id": sheet_id,
            "view_id": view_id,
            "result": payload(&resp),
        }
    }))
}

async fn view_delete_cmd(s: &ArgMatches) -> Result<Value, WpsError> {
    let auth = effective_auth_type(s);
    let dry = s.get_flag("dry-run");
    let retry = *s.get_one::<u32>("retry").unwrap_or(&1);
    let scope = ensure_scope(&auth, dry, true).await?;
    let file_id = resolve_file_id(s, &auth, dry, retry).await?;
    let sheet_id = parse_sheet_id(s)?;
    let view_id = s
        .get_one::<String>("view-id")
        .cloned()
        .ok_or_else(|| WpsError::Validation("缺少 --view-id".to_string()))?;
    let resp = execute(
        "POST",
        &format!("/v7/coop/dbsheet/{file_id}/sheets/{sheet_id}/views/{view_id}/delete"),
        Some(serde_json::json!({})),
        &auth,
        dry,
        retry,
    )
    .await?;
    if !dry && !api_ok(&resp) {
        return Err(WpsError::Network(format!("删除视图失败: {resp}")));
    }
    Ok(serde_json::json!({
        "ok": true,
        "msg": "ok",
        "data": {
            "scope_preflight": scope,
            "action": "view-delete",
            "file_id": file_id,
            "sheet_id": sheet_id,
            "view_id": view_id,
            "result": payload(&resp),
        }
    }))
}

async fn webhook_list_cmd(s: &ArgMatches) -> Result<Value, WpsError> {
    let auth = effective_auth_type(s);
    let dry = s.get_flag("dry-run");
    let retry = *s.get_one::<u32>("retry").unwrap_or(&1);
    let scope = ensure_scope(&auth, dry, true).await?;
    let file_id = resolve_file_id(s, &auth, dry, retry).await?;
    let mut query = HashMap::new();
    if s.get_flag("with-detail") {
        query.insert("with_detail".to_string(), "true".to_string());
    }
    let resp = executor::execute_raw(
        "GET",
        &format!("/v7/coop/dbsheet/{file_id}/hooks"),
        query,
        HashMap::new(),
        None,
        &auth,
        dry,
        retry,
    )
    .await?;
    if !dry && !api_ok(&resp) {
        return Err(WpsError::Network(format!("列出 webhook 失败: {resp}")));
    }
    Ok(serde_json::json!({
        "ok": true,
        "msg": "ok",
        "data": {
            "scope_preflight": scope,
            "action": "webhook-list",
            "file_id": file_id,
            "hooks": payload(&resp),
        }
    }))
}

async fn webhook_create_cmd(s: &ArgMatches) -> Result<Value, WpsError> {
    let auth = effective_auth_type(s);
    let dry = s.get_flag("dry-run");
    let retry = *s.get_one::<u32>("retry").unwrap_or(&1);
    let scope = ensure_scope(&auth, dry, true).await?;
    let file_id = resolve_file_id(s, &auth, dry, retry).await?;
    let body = read_payload_arg(s, "data-json", "data-file")?;
    let resp = execute(
        "POST",
        &format!("/v7/coop/dbsheet/{file_id}/hooks/create"),
        Some(body),
        &auth,
        dry,
        retry,
    )
    .await?;
    if !dry && !api_ok(&resp) {
        return Err(WpsError::Network(format!("创建 webhook 失败: {resp}")));
    }
    Ok(serde_json::json!({
        "ok": true,
        "msg": "ok",
        "data": {
            "scope_preflight": scope,
            "action": "webhook-create",
            "file_id": file_id,
            "result": payload(&resp),
        }
    }))
}

async fn webhook_delete_cmd(s: &ArgMatches) -> Result<Value, WpsError> {
    let auth = effective_auth_type(s);
    let dry = s.get_flag("dry-run");
    let retry = *s.get_one::<u32>("retry").unwrap_or(&1);
    let scope = ensure_scope(&auth, dry, true).await?;
    let file_id = resolve_file_id(s, &auth, dry, retry).await?;
    let hook_id = s
        .get_one::<String>("hook-id")
        .cloned()
        .ok_or_else(|| WpsError::Validation("缺少 --hook-id".to_string()))?;
    let resp = execute(
        "POST",
        &format!("/v7/coop/dbsheet/{file_id}/hooks/{hook_id}/delete"),
        Some(serde_json::json!({})),
        &auth,
        dry,
        retry,
    )
    .await?;
    if !dry && !api_ok(&resp) {
        return Err(WpsError::Network(format!("删除 webhook 失败: {resp}")));
    }
    Ok(serde_json::json!({
        "ok": true,
        "msg": "ok",
        "data": {
            "scope_preflight": scope,
            "action": "webhook-delete",
            "file_id": file_id,
            "hook_id": hook_id,
            "result": payload(&resp),
        }
    }))
}

async fn init_cmd(s: &ArgMatches) -> Result<Value, WpsError> {
    let auth = effective_auth_type(s);
    let dry = s.get_flag("dry-run");
    let retry = *s.get_one::<u32>("retry").unwrap_or(&1);
    let scope = ensure_scope(&auth, dry, true).await?;
    let file_id = resolve_file_id(s, &auth, dry, retry).await?;
    let schema_path = s.get_one::<String>("schema").expect("required");
    let schema = load_schema(schema_path)?;
    let (sheet_key, sheet_def) = pick_sheet_definition(&schema, s.get_one::<String>("sheet-key"))?;
    let sheet_title = sheet_def
        .title
        .clone()
        .unwrap_or_else(|| sheet_key.clone());
    validate_schema_sheet(&sheet_key, sheet_def)?;
    let force_recreate = s.get_flag("force-recreate");

    let mut existing_sheet_id: Option<String> = None;
    if !dry {
        existing_sheet_id = find_sheet_id_by_name(&file_id, &sheet_title, &auth, retry).await?;
    }
    if let Some(existing_id) = existing_sheet_id.clone() {
        if !force_recreate {
            return Ok(serde_json::json!({
                "ok": true,
                "status": 200,
                "data": {
                    "code": 0,
                    "msg": "sheet already exists",
                    "data": {
                        "file_id": file_id,
                        "sheet_id": existing_id,
                        "sheet_name": sheet_title,
                        "sheet_key": sheet_key,
                        "created": false,
                        "scope_preflight": scope
                    }
                }
            }));
        }
        let del = execute(
            "POST",
            &format!("/v7/coop/dbsheet/{file_id}/sheets/{existing_id}/delete"),
            Some(serde_json::json!({})),
            &auth,
            dry,
            retry,
        )
        .await?;
        if !dry && !api_ok(&del) {
            return Err(WpsError::Network(format!("删除旧 sheet 失败: {del}")));
        }
    }

    let fields = schema_fields_to_wps_fields(&sheet_def.fields);
    let views = schema_views_to_wps_views(&sheet_def.views);
    let create_body = serde_json::json!({
        "name": sheet_title,
        "fields": fields,
        "views": views
    });
    let create = execute(
        "POST",
        &format!("/v7/coop/dbsheet/{file_id}/sheets/create"),
        Some(create_body),
        &auth,
        dry,
        retry,
    )
    .await?;
    if !dry && !api_ok(&create) {
        return Err(WpsError::Network(format!("创建 sheet 失败: {create}")));
    }

    let sheet_id = payload(&create)
        .get("sheet")
        .and_then(|v| v.get("id"))
        .and_then(any_to_string)
        .unwrap_or_else(|| {
            if dry {
                "DRY_RUN_SHEET_ID".to_string()
            } else {
                "".to_string()
            }
        });

    let mut clean_result = Value::Null;
    if sheet_def.auto_clean.unwrap_or(true) {
        clean_result = clean_sheet_defaults(&file_id, &sheet_id, &auth, dry, retry).await?;
    }

    Ok(serde_json::json!({
        "ok": true,
        "status": 200,
        "data": {
            "code": 0,
            "msg": "ok",
            "data": {
                "file_id": file_id,
                "sheet_id": sheet_id,
                "sheet_name": sheet_title,
                "sheet_key": sheet_key,
                "created": true,
                "auto_clean_result": clean_result,
                "scope_preflight": scope
            }
        }
    }))
}

async fn select_cmd(s: &ArgMatches) -> Result<Value, WpsError> {
    let auth = effective_auth_type(s);
    let dry = s.get_flag("dry-run");
    let retry = *s.get_one::<u32>("retry").unwrap_or(&1);
    let scope = ensure_scope(&auth, dry, false).await?;
    let file_id = resolve_file_id(s, &auth, dry, retry).await?;
    let sheet_id = parse_sheet_id(s)?;
    let where_clause = s.get_one::<String>("where").cloned();
    let limit = *s.get_one::<usize>("limit").unwrap_or(&100);
    let offset = *s.get_one::<usize>("offset").unwrap_or(&0);
    let page_size = *s.get_one::<usize>("page-size").unwrap_or(&100);
    let selected_fields = s
        .get_one::<String>("fields")
        .map(|x| x.split(',').map(|v| v.trim().to_string()).filter(|v| !v.is_empty()).collect::<Vec<_>>())
        .unwrap_or_default();

    if dry && where_clause.is_some() {
        return Err(WpsError::Validation(
            "--dry-run 与 --where 同时使用时无法本地过滤，请移除其一".to_string(),
        ));
    }

    let selected = sql_like_select_records(
        &file_id,
        &sheet_id,
        where_clause.clone(),
        selected_fields.clone(),
        limit,
        offset,
        page_size,
        &auth,
        dry,
        retry,
    )
    .await?;

    Ok(serde_json::json!({
        "ok": true,
        "status": 200,
        "data": {
            "code": 0,
            "msg": "ok",
            "data": {
                "file_id": file_id,
                "sheet_id": sheet_id,
                "where": where_clause,
                "fields": selected_fields,
                "total": selected.total,
                "offset": offset,
                "limit": limit,
                "records": selected.records,
                "scope_preflight": scope
            }
        }
    }))
}

async fn insert_cmd(s: &ArgMatches) -> Result<Value, WpsError> {
    let auth = effective_auth_type(s);
    let dry = s.get_flag("dry-run");
    let retry = *s.get_one::<u32>("retry").unwrap_or(&1);
    let scope = ensure_scope(&auth, dry, true).await?;
    let file_id = resolve_file_id(s, &auth, dry, retry).await?;
    let sheet_id = parse_sheet_id(s)?;
    let batch_size = *s.get_one::<usize>("batch-size").unwrap_or(&100);
    let prefer_id = s.get_flag("prefer-id");
    let input = read_data_input(s)?;
    let records = normalize_create_records(input)?;
    let created = batch_post_records_create(&file_id, &sheet_id, records, prefer_id, batch_size, &auth, dry, retry).await?;
    Ok(serde_json::json!({
        "ok": true,
        "status": 200,
        "data": {
            "code": 0,
            "msg": "ok",
            "data": {
                "file_id": file_id,
                "sheet_id": sheet_id,
                "action": "insert",
                "batch_size": batch_size,
                "created_records": created,
                "scope_preflight": scope
            }
        }
    }))
}

async fn update_cmd(s: &ArgMatches) -> Result<Value, WpsError> {
    let auth = effective_auth_type(s);
    let dry = s.get_flag("dry-run");
    let retry = *s.get_one::<u32>("retry").unwrap_or(&1);
    let scope = ensure_scope(&auth, dry, true).await?;
    let file_id = resolve_file_id(s, &auth, dry, retry).await?;
    let sheet_id = parse_sheet_id(s)?;
    let batch_size = *s.get_one::<usize>("batch-size").unwrap_or(&100);
    let prefer_id = s.get_flag("prefer-id");
    let input = read_data_input(s)?;
    let record_id = s.get_one::<String>("record-id").cloned();
    let records = normalize_update_records(input, record_id)?;
    let updated = batch_post_records_update(&file_id, &sheet_id, records, prefer_id, batch_size, &auth, dry, retry).await?;
    Ok(serde_json::json!({
        "ok": true,
        "status": 200,
        "data": {
            "code": 0,
            "msg": "ok",
            "data": {
                "file_id": file_id,
                "sheet_id": sheet_id,
                "action": "update",
                "batch_size": batch_size,
                "updated_records": updated,
                "scope_preflight": scope
            }
        }
    }))
}

async fn delete_cmd(s: &ArgMatches) -> Result<Value, WpsError> {
    let auth = effective_auth_type(s);
    let dry = s.get_flag("dry-run");
    let retry = *s.get_one::<u32>("retry").unwrap_or(&1);
    let scope = ensure_scope(&auth, dry, true).await?;
    let file_id = resolve_file_id(s, &auth, dry, retry).await?;
    let sheet_id = parse_sheet_id(s)?;
    let limit = *s.get_one::<usize>("limit").unwrap_or(&100);
    let batch_size = *s.get_one::<usize>("batch-size").unwrap_or(&100);

    let mut ids = Vec::new();
    if let Some(id) = s.get_one::<String>("record-id") {
        ids.push(id.to_string());
    }
    if let Some(id_list) = s.get_one::<String>("record-ids") {
        for p in id_list.split(',') {
            let v = p.trim();
            if !v.is_empty() {
                ids.push(v.to_string());
            }
        }
    }
    if let Some(where_clause) = s.get_one::<String>("where") {
        if dry {
            return Err(WpsError::Validation(
                "--dry-run 与 --where 同时使用时无法解析待删除记录，请移除其一".to_string(),
            ));
        }
        let expr = parse_where_clause(where_clause)?;
        let records = fetch_all_records(&file_id, &sheet_id, &auth, dry, retry, 100).await?;
        let parsed = records
            .iter()
            .map(parse_record_to_map)
            .collect::<Result<Vec<_>, _>>()?;
        for rec in parsed {
            if eval_filter_expr(&expr, &rec) {
                if let Some(id) = rec.get("id").and_then(any_to_string) {
                    ids.push(id);
                }
            }
            if ids.len() >= limit {
                break;
            }
        }
    }
    ids.sort();
    ids.dedup();
    if ids.is_empty() {
        return Err(WpsError::Validation(
            "delete 需要 --record-id / --record-ids / --where 至少一种条件".to_string(),
        ));
    }

    let deleted = batch_post_records_delete(
        &file_id,
        &sheet_id,
        ids.clone(),
        batch_size,
        &auth,
        dry,
        retry,
    )
    .await?;
    Ok(serde_json::json!({
        "ok": true,
        "status": 200,
        "data": {
            "code": 0,
            "msg": "ok",
            "data": {
                "file_id": file_id,
                "sheet_id": sheet_id,
                "action": "delete",
                "requested_record_ids": ids,
                "batch_size": batch_size,
                "deleted_result": deleted,
                "scope_preflight": scope
            }
        }
    }))
}

pub struct SelectedRows {
    pub total: usize,
    pub records: Vec<Value>,
}

pub async fn sql_like_select_records(
    file_id: &str,
    sheet_id: &str,
    where_clause: Option<String>,
    selected_fields: Vec<String>,
    limit: usize,
    offset: usize,
    page_size: usize,
    auth: &str,
    dry: bool,
    retry: u32,
) -> Result<SelectedRows, WpsError> {
    let mut records = fetch_all_records(file_id, sheet_id, auth, dry, retry, page_size).await?;
    let mut parsed_records = records
        .drain(..)
        .map(|r| parse_record_to_map(&r))
        .collect::<Result<Vec<_>, _>>()?;
    if let Some(w) = where_clause.as_ref() {
        let expr = parse_where_clause(w)?;
        parsed_records.retain(|rec| eval_filter_expr(&expr, rec));
    }
    let total = parsed_records.len();
    let sliced = parsed_records
        .into_iter()
        .skip(offset)
        .take(limit)
        .map(|r| project_record(r, &selected_fields))
        .collect::<Vec<_>>();
    Ok(SelectedRows { total, records: sliced })
}

pub async fn resolve_first_sheet_id(file_id: &str, auth: &str, dry: bool, retry: u32) -> Result<String, WpsError> {
    if dry {
        return Ok("DRY_RUN_SHEET_ID".to_string());
    }
    let resp = execute("GET", &format!("/v7/coop/dbsheet/{file_id}/schema"), None, auth, dry, retry).await?;
    if !api_ok(&resp) {
        return Err(WpsError::Network(format!("读取多维表 schema 失败: {resp}")));
    }
    let sheet_id = payload(&resp)
        .get("sheets")
        .and_then(|v| v.as_array())
        .and_then(|arr| arr.first())
        .and_then(|v| v.get("id"))
        .and_then(any_to_string)
        .ok_or_else(|| WpsError::Validation("未找到可用工作表，请先执行 dbsheet init".to_string()))?;
    Ok(sheet_id)
}

pub async fn sql_like_write_records(
    file_id: &str,
    sheet_id: &str,
    action: DocDbAction,
    input: Value,
    record_id: Option<String>,
    prefer_id: bool,
    batch_size: usize,
    auth: &str,
    dry: bool,
    retry: u32,
) -> Result<Value, WpsError> {
    match action {
        DocDbAction::Create => {
            let records = normalize_create_records(input)?;
            batch_post_records_create(file_id, sheet_id, records, prefer_id, batch_size, auth, dry, retry).await
        }
        DocDbAction::Update => {
            let records = normalize_update_records(input, record_id)?;
            batch_post_records_update(file_id, sheet_id, records, prefer_id, batch_size, auth, dry, retry).await
        }
        DocDbAction::Delete => {
            let ids = normalize_delete_ids(input)?;
            if ids.is_empty() {
                return Err(WpsError::Validation("delete 缺少可删除的记录 ID".to_string()));
            }
            batch_post_records_delete(file_id, sheet_id, ids, batch_size, auth, dry, retry).await
        }
    }
}

fn normalize_delete_ids(input: Value) -> Result<Vec<String>, WpsError> {
    match input {
        Value::Array(arr) => {
            let mut out = Vec::new();
            for item in arr {
                if let Some(id) = item.as_str() {
                    out.push(id.to_string());
                    continue;
                }
                if let Some(id) = item.get("id").and_then(any_to_string) {
                    out.push(id);
                    continue;
                }
                if let Some(id) = item.get("record_id").and_then(any_to_string) {
                    out.push(id);
                    continue;
                }
            }
            Ok(out)
        }
        Value::Object(mut obj) => {
            if let Some(v) = obj.remove("record_ids").or_else(|| obj.remove("records")) {
                return normalize_delete_ids(v);
            }
            if let Some(id) = obj.get("record_id").and_then(any_to_string) {
                return Ok(vec![id]);
            }
            if let Some(id) = obj.get("id").and_then(any_to_string) {
                return Ok(vec![id]);
            }
            Ok(vec![])
        }
        _ => Ok(vec![]),
    }
}

async fn clean_cmd(s: &ArgMatches) -> Result<Value, WpsError> {
    let auth = effective_auth_type(s);
    let dry = s.get_flag("dry-run");
    let retry = *s.get_one::<u32>("retry").unwrap_or(&1);
    let scope = ensure_scope(&auth, dry, true).await?;
    let file_id = resolve_file_id(s, &auth, dry, retry).await?;
    let sheet_id = parse_sheet_id(s)?;
    let result = clean_sheet_defaults(&file_id, &sheet_id, &auth, dry, retry).await?;
    Ok(serde_json::json!({
        "ok": true,
        "status": 200,
        "data": {
            "code": 0,
            "msg": "ok",
            "data": {
                "file_id": file_id,
                "sheet_id": sheet_id,
                "clean_result": result,
                "scope_preflight": scope
            }
        }
    }))
}

fn load_schema(path: &str) -> Result<DbSchemaDoc, WpsError> {
    let raw = std::fs::read_to_string(path)
        .map_err(|e| WpsError::Validation(format!("读取 schema 失败: {e}")))?;
    serde_yaml::from_str::<DbSchemaDoc>(&raw)
        .map_err(|e| WpsError::Validation(format!("schema YAML 解析失败: {e}")))
}

fn pick_sheet_definition<'a>(
    schema: &'a DbSchemaDoc,
    sheet_key: Option<&String>,
) -> Result<(String, &'a DbSchemaSheet), WpsError> {
    if schema.sheets.is_empty() {
        return Err(WpsError::Validation("schema.sheets 不能为空".to_string()));
    }
    if let Some(key) = sheet_key {
        let v = schema
            .sheets
            .get(key)
            .ok_or_else(|| WpsError::Validation(format!("sheet-key 不存在: {key}")))?;
        return Ok((key.clone(), v));
    }
    let (k, v) = schema
        .sheets
        .iter()
        .next()
        .ok_or_else(|| WpsError::Validation("schema.sheets 不能为空".to_string()))?;
    Ok((k.clone(), v))
}

fn validate_schema_sheet(sheet_key: &str, sheet: &DbSchemaSheet) -> Result<(), WpsError> {
    if sheet.fields.is_empty() {
        return Err(WpsError::Validation(format!(
            "sheet `{sheet_key}` fields 不能为空"
        )));
    }
    let first = &sheet.fields[0].field_type;
    if first != "AutoNumber" {
        return Err(WpsError::Validation(format!(
            "sheet `{sheet_key}` 首字段必须是 AutoNumber，当前为 `{first}`"
        )));
    }
    Ok(())
}

fn schema_fields_to_wps_fields(fields: &[DbSchemaField]) -> Vec<Value> {
    fields
        .iter()
        .map(|f| {
            let mut obj = serde_json::json!({
                "name": f.name,
                "type": f.field_type
            });
            if let Some(data) = &f.data {
                obj["data"] = data.clone();
            }
            obj
        })
        .collect()
}

fn schema_views_to_wps_views(views: &Option<Vec<DbSchemaView>>) -> Vec<Value> {
    let mut out = Vec::new();
    if let Some(vs) = views {
        for v in vs {
            out.push(serde_json::json!({
                "name": v.name.clone().unwrap_or_else(|| "表格视图".to_string()),
                "type": v.view_type.clone().unwrap_or_else(|| "Grid".to_string())
            }));
        }
    }
    if out.is_empty() {
        out.push(serde_json::json!({"name":"表格视图","type":"Grid"}));
    }
    out
}

async fn find_sheet_id_by_name(
    file_id: &str,
    sheet_name: &str,
    auth: &str,
    retry: u32,
) -> Result<Option<String>, WpsError> {
    let resp = execute("GET", &format!("/v7/coop/dbsheet/{file_id}/schema"), None, auth, false, retry).await?;
    if !api_ok(&resp) {
        return Err(WpsError::Network(format!("读取 schema 失败: {resp}")));
    }
    let sheets = payload(&resp)
        .get("sheets")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    for sheet in sheets {
        if sheet.get("name").and_then(|v| v.as_str()) == Some(sheet_name) {
            if let Some(id) = sheet.get("id").and_then(any_to_string) {
                return Ok(Some(id));
            }
        }
    }
    Ok(None)
}

async fn clean_sheet_defaults(
    file_id: &str,
    sheet_id: &str,
    auth: &str,
    dry: bool,
    retry: u32,
) -> Result<Value, WpsError> {
    let schema_resp = execute("GET", &format!("/v7/coop/dbsheet/{file_id}/schema"), None, auth, dry, retry).await?;
    let mut deleted_fields = Vec::new();
    if !dry {
        let sheets = payload(&schema_resp)
            .get("sheets")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();
        for sheet in sheets {
            let sid = sheet.get("id").and_then(any_to_string).unwrap_or_default();
            if sid == sheet_id {
                let fields = sheet
                    .get("fields")
                    .and_then(|v| v.as_array())
                    .cloned()
                    .unwrap_or_default();
                for field in fields {
                    let name = field.get("name").and_then(|v| v.as_str()).unwrap_or("");
                    if DEFAULT_FIELD_NAMES.contains(&name) {
                        if let Some(fid) = field.get("id").and_then(any_to_string) {
                            deleted_fields.push(fid);
                        }
                    }
                }
                break;
            }
        }
    }

    let mut field_delete_resp = Value::Null;
    if !deleted_fields.is_empty() || dry {
        let path = format!("/v7/coop/dbsheet/{file_id}/sheets/{sheet_id}/fields/delete");
        field_delete_resp = execute(
            "POST",
            &path,
            Some(serde_json::json!({"field_ids": deleted_fields})),
            auth,
            dry,
            retry,
        )
        .await?;
        if !dry && !api_ok(&field_delete_resp) {
            field_delete_resp = execute(
                "POST",
                &path,
                Some(serde_json::json!({"fields": deleted_fields})),
                auth,
                dry,
                retry,
            )
            .await?;
        }
    }

    let records = if dry {
        vec![]
    } else {
        fetch_all_records(file_id, sheet_id, auth, false, retry, 100).await?
    };
    let mut empty_ids = Vec::new();
    for rec in &records {
        let parsed = parse_record_to_map(rec)?;
        let mut is_empty = true;
        for (k, v) in &parsed {
            if k == "id" || k == "编号" {
                continue;
            }
            if !is_empty_value(v) {
                is_empty = false;
                break;
            }
        }
        if is_empty {
            if let Some(id) = parsed.get("id").and_then(any_to_string) {
                empty_ids.push(id);
            }
        }
    }
    let row_delete = if !empty_ids.is_empty() || dry {
        batch_post_records_delete(file_id, sheet_id, empty_ids.clone(), 100, auth, dry, retry).await?
    } else {
        serde_json::json!({"deleted": 0, "batches": []})
    };

    Ok(serde_json::json!({
        "deleted_default_field_ids": deleted_fields,
        "field_delete_response": field_delete_resp,
        "deleted_empty_row_ids": empty_ids,
        "row_delete_response": row_delete
    }))
}

fn read_data_input(s: &ArgMatches) -> Result<Value, WpsError> {
    let raw = if let Some(v) = s.get_one::<String>("data-json") {
        v.clone()
    } else if let Some(path) = s.get_one::<String>("data-file") {
        std::fs::read_to_string(path)
            .map_err(|e| WpsError::Validation(format!("读取 data-file 失败: {e}")))?
    } else {
        return Err(WpsError::Validation(
            "缺少数据输入：请提供 --data-json 或 --data-file".to_string(),
        ));
    };
    serde_json::from_str::<Value>(&raw)
        .map_err(|e| WpsError::Validation(format!("JSON 解析失败: {e}")))
}

fn normalize_create_records(input: Value) -> Result<Vec<Value>, WpsError> {
    let rows = extract_rows(input)?;
    let mut out = Vec::new();
    for row in rows {
        if row.get("fields_value").is_some() {
            out.push(Value::Object(row));
            continue;
        }
        let fields = row
            .get("fields")
            .cloned()
            .unwrap_or_else(|| Value::Object(row.clone()));
        out.push(serde_json::json!({
            "fields_value": fields.to_string()
        }));
    }
    Ok(out)
}

fn normalize_update_records(input: Value, record_id: Option<String>) -> Result<Vec<Value>, WpsError> {
    let mut rows = extract_rows(input)?;
    if let Some(id) = record_id {
        if rows.len() != 1 {
            return Err(WpsError::Validation(
                "--record-id 模式下 data 只能是一条对象记录".to_string(),
            ));
        }
        let row = rows.remove(0);
        let fields = row
            .get("fields")
            .cloned()
            .unwrap_or_else(|| Value::Object(row.clone()));
        return Ok(vec![serde_json::json!({
            "id": id,
            "fields_value": fields.to_string()
        })]);
    }

    let mut out = Vec::new();
    for row in rows {
        if row.get("id").is_none() {
            return Err(WpsError::Validation(
                "update 需要记录 id（可用 --record-id 或在每条记录中提供 id）".to_string(),
            ));
        }
        if row.get("fields_value").is_some() {
            out.push(Value::Object(row));
            continue;
        }
        let id = row.get("id").cloned().unwrap_or(Value::Null);
        let fields = row
            .get("fields")
            .cloned()
            .unwrap_or_else(|| {
                let mut obj = row.clone();
                obj.remove("id");
                Value::Object(obj)
            });
        out.push(serde_json::json!({
            "id": id,
            "fields_value": fields.to_string()
        }));
    }
    Ok(out)
}

fn extract_rows(input: Value) -> Result<Vec<Map<String, Value>>, WpsError> {
    match input {
        Value::Array(arr) => arr
            .into_iter()
            .map(|v| {
                v.as_object()
                    .cloned()
                    .ok_or_else(|| WpsError::Validation("数组中每一项必须是对象".to_string()))
            })
            .collect(),
        Value::Object(mut obj) => {
            if let Some(v) = obj.remove("records") {
                match v {
                    Value::Array(arr) => arr
                        .into_iter()
                        .map(|x| {
                            x.as_object()
                                .cloned()
                                .ok_or_else(|| WpsError::Validation("records 中每项必须是对象".to_string()))
                        })
                        .collect(),
                    _ => Err(WpsError::Validation("records 必须是数组".to_string())),
                }
            } else {
                Ok(vec![obj])
            }
        }
        _ => Err(WpsError::Validation("data 必须是对象或对象数组".to_string())),
    }
}

async fn batch_post_records_create(
    file_id: &str,
    sheet_id: &str,
    records: Vec<Value>,
    prefer_id: bool,
    batch_size: usize,
    auth: &str,
    dry: bool,
    retry: u32,
) -> Result<Value, WpsError> {
    let path = format!("/v7/coop/dbsheet/{file_id}/sheets/{sheet_id}/records/create");
    let mut created = Vec::new();
    let mut batches = Vec::new();
    for chunk in records.chunks(batch_size.max(1)) {
        let body = serde_json::json!({"prefer_id": prefer_id, "records": chunk});
        let resp = execute("POST", &path, Some(body), auth, dry, retry).await?;
        if !dry && !api_ok(&resp) {
            return Err(WpsError::Network(format!("批量创建记录失败: {resp}")));
        }
        let rs = payload(&resp)
            .get("records")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();
        created.extend(rs);
        batches.push(resp);
    }
    Ok(serde_json::json!({ "records": created, "batches": batches }))
}

async fn batch_post_records_update(
    file_id: &str,
    sheet_id: &str,
    records: Vec<Value>,
    prefer_id: bool,
    batch_size: usize,
    auth: &str,
    dry: bool,
    retry: u32,
) -> Result<Value, WpsError> {
    let path = format!("/v7/coop/dbsheet/{file_id}/sheets/{sheet_id}/records/update");
    let mut updated = Vec::new();
    let mut batches = Vec::new();
    for chunk in records.chunks(batch_size.max(1)) {
        let body = serde_json::json!({"prefer_id": prefer_id, "records": chunk});
        let resp = execute("POST", &path, Some(body), auth, dry, retry).await?;
        if !dry && !api_ok(&resp) {
            return Err(WpsError::Network(format!("批量更新记录失败: {resp}")));
        }
        let rs = payload(&resp)
            .get("records")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();
        updated.extend(rs);
        batches.push(resp);
    }
    Ok(serde_json::json!({ "records": updated, "batches": batches }))
}

async fn batch_post_records_delete(
    file_id: &str,
    sheet_id: &str,
    record_ids: Vec<String>,
    batch_size: usize,
    auth: &str,
    dry: bool,
    retry: u32,
) -> Result<Value, WpsError> {
    let path = format!("/v7/coop/dbsheet/{file_id}/sheets/{sheet_id}/records/batch_delete");
    let mut batches = Vec::new();
    for chunk in record_ids.chunks(batch_size.max(1)) {
        let ids = chunk.iter().map(|x| Value::String(x.clone())).collect::<Vec<_>>();
        let mut resp = execute(
            "POST",
            &path,
            Some(serde_json::json!({"records": ids})),
            auth,
            dry,
            retry,
        )
        .await?;
        if !dry && !api_ok(&resp) {
            resp = execute(
                "POST",
                &path,
                Some(serde_json::json!({"record_ids": ids})),
                auth,
                dry,
                retry,
            )
            .await?;
        }
        if !dry && !api_ok(&resp) {
            return Err(WpsError::Network(format!("批量删除记录失败: {resp}")));
        }
        batches.push(resp);
    }
    Ok(serde_json::json!({"deleted": record_ids.len(), "batches": batches}))
}

async fn fetch_all_records(
    file_id: &str,
    sheet_id: &str,
    auth: &str,
    dry: bool,
    retry: u32,
    page_size: usize,
) -> Result<Vec<Value>, WpsError> {
    if dry {
        return Ok(vec![]);
    }
    let mut out = Vec::new();
    let mut page_token: Option<String> = None;
    loop {
        let mut body = serde_json::json!({
            "page_size": page_size.max(1),
            "text_value": "text"
        });
        if let Some(token) = &page_token {
            body["page_token"] = Value::String(token.clone());
        }
        let resp = execute(
            "POST",
            &format!("/v7/coop/dbsheet/{file_id}/sheets/{sheet_id}/records"),
            Some(body),
            auth,
            false,
            retry,
        )
        .await?;
        if !api_ok(&resp) {
            return Err(WpsError::Network(format!("读取记录失败: {resp}")));
        }
        let p = payload(&resp);
        let records = p
            .get("records")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();
        out.extend(records);
        let next = p
            .get("page_token")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .filter(|s| !s.is_empty());
        if next.is_none() || next == page_token {
            break;
        }
        page_token = next;
    }
    Ok(out)
}

fn parse_record_to_map(record: &Value) -> Result<Map<String, Value>, WpsError> {
    let mut out = Map::new();
    if let Some(id) = record.get("id").and_then(any_to_string) {
        out.insert("id".to_string(), Value::String(id));
    }
    if let Some(fields) = record.get("fields") {
        match fields {
            Value::String(s) => {
                let parsed = serde_json::from_str::<Value>(s).unwrap_or(Value::Object(Map::new()));
                if let Some(obj) = parsed.as_object() {
                    for (k, v) in obj {
                        out.insert(k.clone(), v.clone());
                    }
                }
            }
            Value::Object(obj) => {
                for (k, v) in obj {
                    out.insert(k.clone(), v.clone());
                }
            }
            _ => {}
        }
    }
    Ok(out)
}

fn project_record(record: Map<String, Value>, fields: &[String]) -> Value {
    if fields.is_empty() {
        return Value::Object(record);
    }
    let mut out = Map::new();
    if let Some(id) = record.get("id") {
        out.insert("id".to_string(), id.clone());
    }
    for f in fields {
        if let Some(v) = record.get(f) {
            out.insert(f.clone(), v.clone());
        }
    }
    Value::Object(out)
}

fn any_to_string(v: &Value) -> Option<String> {
    if let Some(s) = v.as_str() {
        Some(s.to_string())
    } else if let Some(n) = v.as_i64() {
        Some(n.to_string())
    } else if let Some(n) = v.as_u64() {
        Some(n.to_string())
    } else {
        None
    }
}

fn is_empty_value(v: &Value) -> bool {
    match v {
        Value::Null => true,
        Value::String(s) => s.trim().is_empty(),
        Value::Array(a) => a.is_empty(),
        Value::Object(o) => o.is_empty(),
        _ => false,
    }
}

fn parse_where_clause(raw: &str) -> Result<FilterExpr, WpsError> {
    let text = raw.trim();
    if text.is_empty() {
        return Err(WpsError::Validation("where 条件不能为空".to_string()));
    }
    let or_parts = split_top_level(text, " OR ");
    if or_parts.len() > 1 {
        let mut items = Vec::new();
        for p in or_parts {
            items.push(parse_where_clause(p.trim())?);
        }
        return Ok(FilterExpr::Group {
            mode: LogicOp::Or,
            items,
        });
    }
    let and_parts = split_top_level(text, " AND ");
    if and_parts.len() > 1 {
        let mut items = Vec::new();
        for p in and_parts {
            items.push(parse_where_clause(p.trim())?);
        }
        return Ok(FilterExpr::Group {
            mode: LogicOp::And,
            items,
        });
    }
    Ok(FilterExpr::Cond(parse_single_condition(text)?))
}

fn split_top_level<'a>(text: &'a str, sep: &str) -> Vec<&'a str> {
    let mut out = Vec::new();
    let mut start = 0usize;
    let mut depth = 0i32;
    let mut quote: Option<char> = None;
    let mut iter = text.char_indices().peekable();
    while let Some((i, ch)) = iter.next() {
        if matches!(ch, '\'' | '"') {
            if quote.is_none() {
                quote = Some(ch);
            } else if quote == Some(ch) {
                quote = None;
            }
        }
        if quote.is_none() {
            if ch == '(' {
                depth += 1;
            } else if ch == ')' && depth > 0 {
                depth -= 1;
            }
            if depth == 0
                && bytes_starts_with_ignore_ascii_case(&text.as_bytes()[i..], sep.as_bytes())
            {
                out.push(&text[start..i]);
                start = i + sep.len();
                while let Some((j, _)) = iter.peek().copied() {
                    if j < start {
                        let _ = iter.next();
                    } else {
                        break;
                    }
                }
                continue;
            }
        }
    }
    out.push(&text[start..]);
    out
}

fn parse_single_condition(text: &str) -> Result<Condition, WpsError> {
    let cond = text.trim().trim_start_matches('(').trim_end_matches(')').trim();
    let upper = cond.to_uppercase();
    if let Some(idx) = upper.find(" IN ") {
        let field = cond[..idx].trim();
        let mut rhs = cond[idx + 4..].trim();
        if rhs.starts_with('(') && rhs.ends_with(')') && rhs.len() >= 2 {
            rhs = &rhs[1..rhs.len() - 1];
        }
        let values = rhs
            .split(',')
            .map(|x| trim_quotes(x.trim()).to_string())
            .filter(|x| !x.is_empty())
            .collect::<Vec<_>>();
        return Ok(Condition {
            field: field.to_string(),
            op: ConditionOp::In,
            values,
        });
    }
    if let Some(idx) = upper.find(" LIKE ") {
        let field = cond[..idx].trim();
        let value = trim_quotes(cond[idx + 6..].trim()).to_string();
        let (op, cleaned) = if value.starts_with('%') && value.ends_with('%') && value.len() > 1 {
            (ConditionOp::Contains, value[1..value.len() - 1].to_string())
        } else if value.starts_with('%') {
            (ConditionOp::EndsWith, value[1..].to_string())
        } else if value.ends_with('%') {
            (ConditionOp::StartsWith, value[..value.len() - 1].to_string())
        } else {
            (ConditionOp::Eq, value)
        };
        return Ok(Condition {
            field: field.to_string(),
            op,
            values: vec![cleaned],
        });
    }
    for (sym, op) in [
        (">=", ConditionOp::Ge),
        ("<=", ConditionOp::Le),
        ("!=", ConditionOp::Ne),
        ("=", ConditionOp::Eq),
        (">", ConditionOp::Gt),
        ("<", ConditionOp::Lt),
    ] {
        if let Some(idx) = find_operator_outside_quotes(cond, sym) {
            let field = cond[..idx].trim().to_string();
            let rhs = trim_quotes(cond[idx + sym.len()..].trim()).to_string();
            return Ok(Condition {
                field,
                op,
                values: vec![rhs],
            });
        }
    }
    Err(WpsError::Validation(format!("无法解析 where 条件: {text}")))
}

fn find_operator_outside_quotes(text: &str, op: &str) -> Option<usize> {
    let mut quote: Option<char> = None;
    let mut iter = text.char_indices().peekable();
    while let Some((i, ch)) = iter.next() {
        if matches!(ch, '\'' | '"') {
            if quote.is_none() {
                quote = Some(ch);
            } else if quote == Some(ch) {
                quote = None;
            }
        }
        if quote.is_none() && text[i..].starts_with(op) {
            return Some(i);
        }
    }
    None
}

fn bytes_starts_with_ignore_ascii_case(haystack: &[u8], needle: &[u8]) -> bool {
    if haystack.len() < needle.len() {
        return false;
    }
    for idx in 0..needle.len() {
        if haystack[idx].to_ascii_uppercase() != needle[idx].to_ascii_uppercase() {
            return false;
        }
    }
    true
}

fn trim_quotes(v: &str) -> &str {
    if (v.starts_with('"') && v.ends_with('"')) || (v.starts_with('\'') && v.ends_with('\'')) {
        &v[1..v.len() - 1]
    } else {
        v
    }
}

fn eval_filter_expr(expr: &FilterExpr, row: &Map<String, Value>) -> bool {
    match expr {
        FilterExpr::Group { mode, items } => match mode {
            LogicOp::And => items.iter().all(|x| eval_filter_expr(x, row)),
            LogicOp::Or => items.iter().any(|x| eval_filter_expr(x, row)),
        },
        FilterExpr::Cond(c) => eval_condition(c, row),
    }
}

fn eval_condition(c: &Condition, row: &Map<String, Value>) -> bool {
    let value = row
        .get(&c.field)
        .cloned()
        .unwrap_or(Value::Null);
    let lhs_str = value_to_string(&value);
    let rhs = c.values.first().cloned().unwrap_or_default();
    match c.op {
        ConditionOp::Eq => lhs_str == rhs,
        ConditionOp::Ne => lhs_str != rhs,
        ConditionOp::Gt => cmp_number_or_lex(&lhs_str, &rhs) > 0,
        ConditionOp::Lt => cmp_number_or_lex(&lhs_str, &rhs) < 0,
        ConditionOp::Ge => cmp_number_or_lex(&lhs_str, &rhs) >= 0,
        ConditionOp::Le => cmp_number_or_lex(&lhs_str, &rhs) <= 0,
        ConditionOp::In => c.values.iter().any(|x| x == &lhs_str),
        ConditionOp::Contains => lhs_str.contains(&rhs),
        ConditionOp::StartsWith => lhs_str.starts_with(&rhs),
        ConditionOp::EndsWith => lhs_str.ends_with(&rhs),
    }
}

fn value_to_string(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        Value::Number(n) => n.to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Null => "".to_string(),
        _ => serde_json::to_string(v).unwrap_or_default(),
    }
}

fn cmp_number_or_lex(lhs: &str, rhs: &str) -> i8 {
    let ln = lhs.parse::<f64>().ok();
    let rn = rhs.parse::<f64>().ok();
    if let (Some(a), Some(b)) = (ln, rn) {
        return if a > b {
            1
        } else if a < b {
            -1
        } else {
            0
        };
    }
    if lhs > rhs {
        1
    } else if lhs < rhs {
        -1
    } else {
        0
    }
}
