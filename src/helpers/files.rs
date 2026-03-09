use std::collections::HashMap;
use std::io::Read;
use std::path::Path;
use std::time::Instant;

use clap::{Arg, ArgAction, ArgMatches, Command};
use reqwest::Method;
use serde_json::Value;
use sha2::{Digest, Sha256};
use tokio::io::AsyncWriteExt;

use crate::error::WpsError;
use crate::executor;
use crate::skill_runtime::{self, ScopePreflight, SkillStateStore};

const APP_ROOT_NAME: &str = "应用";
const OPENCLAW_ROOT_NAME: &str = "openclaw";
const SKILL_NAME: &str = skill_runtime::WPS_APP_FILES_SKILL;
const APP_REGISTRY_FILE: &str = "app_registry.json";
const FILE_REGISTRY_FILE: &str = "file_registry.json";
const REQUIRED_FILE_SCOPES: [&str; 2] = ["kso.file.read", "kso.file.readwrite"];

pub fn command() -> Command {
    Command::new("files")
        .about("应用目录与文件助手（wps-app-files）")
        .after_help(
            "示例：\n  \
             wpscli files list-apps --user-token\n  \
             wpscli files create --app \"Demo\" --file \"日报.otl\" --user-token\n  \
             wpscli files list-files --app \"Demo\" --user-token",
        )
        .subcommand(
            with_common_opts(Command::new("list-apps").about("列出 应用/openclaw 下的应用目录"))
                .arg(Arg::new("drive-id").long("drive-id").num_args(1).help("网盘 ID（不传则自动探测）")),
        )
        .subcommand(
            with_common_opts(Command::new("ensure-app").about("确保应用目录存在"))
                .arg(Arg::new("app").long("app").required(true).num_args(1).help("应用目录名称"))
                .arg(Arg::new("drive-id").long("drive-id").num_args(1).help("网盘 ID（不传则自动探测）")),
        )
        .subcommand(
            with_common_opts(Command::new("create").about("创建应用文件（会自动创建目录）"))
                .arg(Arg::new("app").long("app").required(true).num_args(1).help("应用目录名称"))
                .arg(Arg::new("file").long("file").required(true).num_args(1).help("文件名（含扩展名）"))
                .arg(
                    Arg::new("on-name-conflict")
                        .long("on-name-conflict")
                        .default_value("fail")
                        .value_parser(["fail", "rename", "replace"])
                        .help("同名冲突策略：fail/rename/replace"),
                )
                .arg(Arg::new("drive-id").long("drive-id").num_args(1).help("网盘 ID（不传则自动探测）")),
        )
        .subcommand(
            with_common_opts(Command::new("add-file").about("在已有应用下新增文件"))
                .visible_alias("create-file")
                .arg(Arg::new("app").long("app").required(true).num_args(1).help("应用目录名称"))
                .arg(Arg::new("file").long("file").required(true).num_args(1).help("文件名（含扩展名）"))
                .arg(
                    Arg::new("on-name-conflict")
                        .long("on-name-conflict")
                        .default_value("fail")
                        .value_parser(["fail", "rename", "replace"])
                        .help("同名冲突策略：fail/rename/replace"),
                )
                .arg(Arg::new("drive-id").long("drive-id").num_args(1).help("网盘 ID（不传则自动探测）")),
        )
        .subcommand(
            with_common_opts(Command::new("list-files").about("列出某应用下的文件"))
                .arg(Arg::new("app").long("app").required(true).num_args(1).help("应用目录名称"))
                .arg(Arg::new("drive-id").long("drive-id").num_args(1).help("网盘 ID（不传则自动探测）")),
        )
        .subcommand(
            with_common_opts(Command::new("get").about("查询文件信息"))
                .arg(Arg::new("drive-id").long("drive-id").num_args(1).help("网盘 ID"))
                .arg(Arg::new("file-id").long("file-id").num_args(1).help("文件 ID（优先）"))
                .arg(Arg::new("app").long("app").num_args(1).help("应用目录名称（与 --file 搭配）"))
                .arg(Arg::new("file").long("file").num_args(1).help("文件名（与 --app 搭配）")),
        )
        .subcommand(
            with_common_opts(Command::new("upload").about("上传本地文件（请求上传->存储上传->提交完成）"))
                .arg(
                    Arg::new("local-file")
                        .long("local-file")
                        .required(true)
                        .num_args(1)
                        .help("本地文件路径"),
                )
                .arg(Arg::new("name").long("name").num_args(1).help("远端文件名（默认取本地文件名）"))
                .arg(Arg::new("app").long("app").num_args(1).help("目标应用目录（与 --parent-id 二选一，默认根目录）"))
                .arg(Arg::new("parent-id").long("parent-id").num_args(1).help("目标父目录 ID（优先于 --app）"))
                .arg(
                    Arg::new("on-name-conflict")
                        .long("on-name-conflict")
                        .default_value("rename")
                        .value_parser(["fail", "rename", "overwrite", "replace"])
                        .help("同名冲突策略：fail/rename/overwrite/replace"),
                )
                .arg(Arg::new("internal").long("internal").action(ArgAction::SetTrue).help("优先请求内网上传地址"))
                .arg(Arg::new("drive-id").long("drive-id").num_args(1).help("网盘 ID（不传则自动探测）")),
        )
        .subcommand(
            with_common_opts(Command::new("download").about("下载文件到本地"))
                .arg(Arg::new("drive-id").long("drive-id").num_args(1).help("网盘 ID"))
                .arg(Arg::new("file-id").long("file-id").num_args(1).help("文件 ID（优先）"))
                .arg(Arg::new("app").long("app").num_args(1).help("应用目录名称（与 --file 搭配）"))
                .arg(Arg::new("file").long("file").num_args(1).help("文件名（与 --app 搭配）"))
                .arg(Arg::new("output").long("output").num_args(1).help("本地输出路径（可为目录或文件路径）"))
                .arg(Arg::new("overwrite").long("overwrite").action(ArgAction::SetTrue).help("允许覆盖本地已存在文件"))
                .arg(Arg::new("with-hash").long("with-hash").action(ArgAction::SetTrue).help("请求下载信息时附带 hashes"))
                .arg(Arg::new("internal").long("internal").action(ArgAction::SetTrue).help("优先请求内网下载地址"))
                .arg(
                    Arg::new("storage-base-domain")
                        .long("storage-base-domain")
                        .num_args(1)
                        .help("下载域名偏好：wps.cn/kdocs.cn/wps365.com"),
                ),
        )
        .subcommand(
            with_common_opts(Command::new("transfer").about("统一传输视图（带阶段耗时与恢复建议）"))
                .arg(
                    Arg::new("mode")
                        .long("mode")
                        .required(true)
                        .value_parser(["upload", "download"])
                        .help("传输模式：upload/download"),
                )
                .arg(
                    Arg::new("local-file")
                        .long("local-file")
                        .num_args(1)
                        .required_if_eq("mode", "upload")
                        .help("upload 模式：本地文件路径"),
                )
                .arg(Arg::new("name").long("name").num_args(1).help("upload 模式：远端文件名（默认本地文件名）"))
                .arg(Arg::new("app").long("app").num_args(1).help("应用目录（与 --parent-id 或 --file-id 组合）"))
                .arg(Arg::new("parent-id").long("parent-id").num_args(1).help("upload 模式：目标父目录 ID（优先于 --app）"))
                .arg(
                    Arg::new("on-name-conflict")
                        .long("on-name-conflict")
                        .default_value("rename")
                        .value_parser(["fail", "rename", "overwrite", "replace"])
                        .help("upload 模式：同名冲突策略"),
                )
                .arg(Arg::new("drive-id").long("drive-id").num_args(1).help("网盘 ID（不传则自动探测）"))
                .arg(Arg::new("file-id").long("file-id").num_args(1).help("download 模式：文件 ID（优先）"))
                .arg(Arg::new("file").long("file").num_args(1).help("download 模式：文件名（与 --app 搭配）"))
                .arg(Arg::new("output").long("output").num_args(1).help("download 模式：本地输出路径"))
                .arg(Arg::new("overwrite").long("overwrite").action(ArgAction::SetTrue).help("download 模式：允许覆盖本地文件"))
                .arg(Arg::new("with-hash").long("with-hash").action(ArgAction::SetTrue).help("download 模式：返回 hash 校验值"))
                .arg(Arg::new("internal").long("internal").action(ArgAction::SetTrue).help("优先请求内网传输地址"))
                .arg(
                    Arg::new("storage-base-domain")
                        .long("storage-base-domain")
                        .num_args(1)
                        .help("download 模式：下载域名偏好 wps.cn/kdocs.cn/wps365.com"),
                ),
        )
        .subcommand(
            with_common_opts(Command::new("state").about("查看本地状态仓库（registry/log）"))
                .arg(
                    Arg::new("limit")
                        .long("limit")
                        .default_value("20")
                        .value_parser(clap::value_parser!(usize))
                        .help("返回最近操作日志条数"),
                ),
        )
}

pub async fn handle(args: &[String]) -> Result<serde_json::Value, WpsError> {
    let mut argv = vec!["files".to_string()];
    argv.extend_from_slice(args);
    let m = command()
        .try_get_matches_from(argv)
        .map_err(|e| WpsError::Validation(e.to_string()))?;

    let (action, result) = match m.subcommand() {
        Some(("list-apps", s)) => ("list-apps", list_apps(s).await),
        Some(("ensure-app", s)) => ("ensure-app", ensure_app(s).await),
        Some(("create", s)) | Some(("add-file", s)) => ("create", create_file(s).await),
        Some(("list-files", s)) => ("list-files", list_files(s).await),
        Some(("get", s)) => ("get", get_file(s).await),
        Some(("upload", s)) => ("upload", upload_file(s).await),
        Some(("download", s)) => ("download", download_file(s).await),
        Some(("transfer", s)) => ("transfer", transfer(s).await),
        Some(("state", s)) => ("state", read_state(s).await),
        _ => {
            return Err(WpsError::Validation(
                "unknown files helper subcommand".to_string(),
            ))
        }
    };
    persist_operation_log(action, args, &result);
    result
}

fn persist_operation_log(action: &str, args: &[String], result: &Result<Value, WpsError>) {
    let Ok(store) = SkillStateStore::new(SKILL_NAME) else {
        return;
    };
    let detail = match result {
        Ok(v) => serde_json::json!({
            "args": args,
            "result_preview": preview_json(v, 1200),
        }),
        Err(e) => serde_json::json!({
            "args": args,
            "error_code": e.code(),
            "error_message": e.to_string(),
        }),
    };
    let status = match result {
        Ok(v) => {
            if v.get("ok").and_then(|x| x.as_bool()) == Some(false) {
                "error"
            } else {
                "ok"
            }
        }
        Err(_) => "error",
    };
    let _ = store.append_operation_log(action, status, detail);
}

fn preview_json(v: &Value, max_len: usize) -> Value {
    let s = serde_json::to_string(v).unwrap_or_else(|_| "{}".to_string());
    if s.len() <= max_len {
        v.clone()
    } else {
        let mut cut = 0usize;
        for (idx, _) in s.char_indices() {
            if idx > max_len {
                break;
            }
            cut = idx;
        }
        if cut == 0 && !s.is_empty() {
            cut = s
                .char_indices()
                .nth(1)
                .map(|(i, _)| i)
                .unwrap_or(s.len());
        }
        serde_json::json!({
            "truncated": true,
            "size": s.len(),
            "head": &s[..cut]
        })
    }
}

async fn preflight(auth: &str, dry: bool) -> Result<ScopePreflight, WpsError> {
    skill_runtime::ensure_user_scope(auth, &REQUIRED_FILE_SCOPES, dry).await
}

fn state_paths_value() -> Value {
    serde_json::to_value(skill_runtime::default_state_paths(SKILL_NAME)).unwrap_or(Value::Null)
}

fn now_iso8601() -> String {
    chrono::Utc::now().to_rfc3339()
}

fn guess_file_type(name: &str) -> String {
    let ext = name.rsplit('.').next().unwrap_or("").to_ascii_lowercase();
    if ext.is_empty() {
        "unknown".to_string()
    } else {
        ext
    }
}

fn vstr(v: &Value, key: &str) -> Option<String> {
    v.get(key).and_then(|x| x.as_str()).map(|s| s.to_string())
}

fn update_app_registry(
    store: &SkillStateStore,
    app: &str,
    drive_id: &str,
    folder: &Value,
) -> Result<(), WpsError> {
    let app_id = vstr(folder, "id").unwrap_or_default();
    if app_id.is_empty() {
        return Ok(());
    }
    let mut app_map = store.load_registry_map(APP_REGISTRY_FILE)?;
    let now = now_iso8601();
    let files = app_map
        .get(app)
        .and_then(|v| v.get("files"))
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    app_map.insert(
        app.to_string(),
        serde_json::json!({
            "id": app_id,
            "name": app.rsplit('/').next().unwrap_or(app),
            "path": format!("{APP_ROOT_NAME}/{OPENCLAW_ROOT_NAME}/{app}"),
            "drive_id": drive_id,
            "created_at": app_map.get(app)
                .and_then(|v| v.get("created_at"))
                .and_then(|v| v.as_str())
                .unwrap_or(&now),
            "updated_at": now,
            "files": files,
        }),
    );
    store.save_registry_map(APP_REGISTRY_FILE, &app_map)
}

fn update_file_registry(
    store: &SkillStateStore,
    app: &str,
    drive_id: &str,
    parent_id: &str,
    created: &Value,
) -> Result<(), WpsError> {
    let file_id = vstr(created, "id").unwrap_or_default();
    if file_id.is_empty() {
        return Ok(());
    }
    let file_name = vstr(created, "name").unwrap_or_else(|| "unnamed".to_string());
    let mut file_map = store.load_registry_map(FILE_REGISTRY_FILE)?;
    file_map.insert(
        file_id.clone(),
        serde_json::json!({
            "id": file_id,
            "name": file_name,
            "type": guess_file_type(&file_name),
            "app_name": app,
            "drive_id": drive_id,
            "parent_id": parent_id,
            "link_id": created.get("linkid").or_else(|| created.get("link_id")).cloned().unwrap_or(Value::Null),
            "link_url": created.get("link").or_else(|| created.get("url")).cloned().unwrap_or(Value::Null),
            "created_at": now_iso8601(),
        }),
    );
    store.save_registry_map(FILE_REGISTRY_FILE, &file_map)?;

    let mut app_map = store.load_registry_map(APP_REGISTRY_FILE)?;
    let mut file_ids = app_map
        .get(app)
        .and_then(|v| v.get("files"))
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    if !file_ids
        .iter()
        .any(|v| v.as_str().map(|x| x == file_id).unwrap_or(false))
    {
        file_ids.push(Value::String(file_id.clone()));
    }
    let current = app_map
        .get(app)
        .cloned()
        .unwrap_or_else(|| serde_json::json!({}));
    let mut obj = current.as_object().cloned().unwrap_or_default();
    obj.insert("files".to_string(), Value::Array(file_ids));
    obj.insert("updated_at".to_string(), Value::String(now_iso8601()));
    app_map.insert(app.to_string(), Value::Object(obj));
    store.save_registry_map(APP_REGISTRY_FILE, &app_map)?;
    Ok(())
}

async fn read_state(s: &ArgMatches) -> Result<Value, WpsError> {
    let limit = *s.get_one::<usize>("limit").unwrap_or(&20);
    let store = SkillStateStore::new(SKILL_NAME)?;
    let app_registry = store.load_registry_map(APP_REGISTRY_FILE)?;
    let file_registry = store.load_registry_map(FILE_REGISTRY_FILE)?;
    let operations = store.read_recent_operations(limit)?;
    Ok(serde_json::json!({
        "ok": true,
        "data": {
            "code": 0,
            "msg": "ok",
            "data": {
                "paths": state_paths_value(),
                "registry": {
                    "apps": app_registry,
                    "files": file_registry,
                },
                "recent_operations": operations,
            }
        }
    }))
}

fn register_app_if_possible(app: &str, drive_id: &str, folder: &Value) {
    if let Ok(store) = SkillStateStore::new(SKILL_NAME) {
        let _ = update_app_registry(&store, app, drive_id, folder);
    }
}

fn register_file_if_possible(app: &str, drive_id: &str, parent_id: &str, created: &Value) {
    if let Ok(store) = SkillStateStore::new(SKILL_NAME) {
        let _ = update_file_registry(&store, app, drive_id, parent_id, created);
    }
}

fn with_common_opts(cmd: Command) -> Command {
    cmd.arg(
        Arg::new("auth-type")
            .long("auth-type")
            .value_parser(["app", "user"])
            .default_value("user")
            .help("鉴权类型：app 或 user"),
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

fn effective_auth_type(s: &ArgMatches) -> String {
    if s.get_flag("user-token") {
        "user".to_string()
    } else {
        s.get_one::<String>("auth-type")
            .cloned()
            .unwrap_or_else(|| "user".to_string())
    }
}

fn api_payload(v: &Value) -> Value {
    v.get("data")
        .and_then(|x| x.get("data"))
        .cloned()
        .unwrap_or(Value::Null)
}

fn api_ok(v: &Value) -> bool {
    v.get("ok").and_then(|x| x.as_bool()).unwrap_or(false)
        && v.get("data")
            .and_then(|x| x.get("code"))
            .and_then(|x| x.as_i64())
            .unwrap_or(-1)
            == 0
}

async fn execute(
    method: &str,
    path: &str,
    query: HashMap<String, String>,
    body: Option<Value>,
    auth_type: &str,
    dry: bool,
    retry: u32,
) -> Result<Value, WpsError> {
    executor::execute_raw(
        method,
        path,
        query,
        HashMap::new(),
        body.map(|b| b.to_string()),
        auth_type,
        dry,
        retry,
    )
    .await
}

async fn get_default_drive_id(
    auth_type: &str,
    dry: bool,
    retry: u32,
    preferred: Option<String>,
) -> Result<String, WpsError> {
    if let Some(d) = preferred {
        if !d.trim().is_empty() {
            return Ok(d);
        }
    }
    let mut q = HashMap::new();
    q.insert("allotee_type".to_string(), "user".to_string());
    q.insert("page_size".to_string(), "20".to_string());
    let resp = execute("GET", "/v7/drives", q, None, auth_type, dry, retry).await?;
    if dry {
        return Ok("DRY_RUN_DRIVE_ID".to_string());
    }
    if !api_ok(&resp) {
        return Err(WpsError::Network(format!("获取 drives 失败: {resp}")));
    }
    let items = api_payload(&resp)
        .get("items")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    let picked = items
        .iter()
        .find(|x| x.get("source").and_then(|v| v.as_str()) == Some("special"))
        .or_else(|| items.first())
        .and_then(|x| x.get("id"))
        .and_then(|v| v.as_str())
        .ok_or_else(|| WpsError::Validation("未找到可用 drive_id".to_string()))?;
    Ok(picked.to_string())
}

async fn list_all_children(
    drive_id: &str,
    parent_id: &str,
    auth_type: &str,
    dry: bool,
    retry: u32,
) -> Result<Vec<Value>, WpsError> {
    if dry {
        return Ok(vec![]);
    }
    let mut out = Vec::new();
    let mut page_token: Option<String> = None;
    loop {
        let mut q = HashMap::new();
        q.insert("page_size".to_string(), "100".to_string());
        if let Some(t) = &page_token {
            q.insert("page_token".to_string(), t.clone());
        }
        let resp = execute(
            "GET",
            &format!("/v7/drives/{drive_id}/files/{parent_id}/children"),
            q,
            None,
            auth_type,
            dry,
            retry,
        )
        .await?;
        if !api_ok(&resp) {
            return Err(WpsError::Network(format!("读取子目录失败: {resp}")));
        }
        let payload = api_payload(&resp);
        let batch = payload
            .get("items")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();
        out.extend(batch);
        let next = payload
            .get("next_page_token")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .or_else(|| payload.get("page_token").and_then(|v| v.as_str()).map(|s| s.to_string()));
        if next.as_deref().unwrap_or("").is_empty() || next == page_token {
            break;
        }
        page_token = next;
    }
    Ok(out)
}

async fn ensure_folder(
    drive_id: &str,
    parent_id: &str,
    folder_name: &str,
    auth_type: &str,
    dry: bool,
    retry: u32,
) -> Result<Value, WpsError> {
    let children = list_all_children(drive_id, parent_id, auth_type, dry, retry).await?;
    if let Some(found) = children.iter().find(|x| {
        x.get("name").and_then(|v| v.as_str()) == Some(folder_name)
            && (x.get("type").and_then(|v| v.as_str()) == Some("folder")
                || x.get("file_type").and_then(|v| v.as_str()) == Some("folder"))
    }) {
        return Ok(found.clone());
    }
    let body = serde_json::json!({
        "name": folder_name,
        "file_type": "folder",
        "on_name_conflict": "fail"
    });
    let resp = execute(
        "POST",
        &format!("/v7/drives/{drive_id}/files/{parent_id}/create"),
        HashMap::new(),
        Some(body),
        auth_type,
        dry,
        retry,
    )
    .await?;
    if !dry && !api_ok(&resp) {
        return Err(WpsError::Network(format!("创建目录失败: {resp}")));
    }
    Ok(api_payload(&resp))
}

async fn ensure_app_folder(
    drive_id: &str,
    app_path: &str,
    auth_type: &str,
    dry: bool,
    retry: u32,
) -> Result<Value, WpsError> {
    let app_root = ensure_folder(drive_id, "0", APP_ROOT_NAME, auth_type, dry, retry).await?;
    let app_root_id = app_root
        .get("id")
        .and_then(|v| v.as_str())
        .unwrap_or("0")
        .to_string();
    let openclaw = ensure_folder(drive_id, &app_root_id, OPENCLAW_ROOT_NAME, auth_type, dry, retry).await?;
    let mut current = openclaw;
    for seg in app_path.split('/').filter(|s| !s.trim().is_empty()) {
        let parent_id = current
            .get("id")
            .and_then(|v| v.as_str())
            .unwrap_or("0")
            .to_string();
        current = ensure_folder(drive_id, &parent_id, seg, auth_type, dry, retry).await?;
    }
    Ok(current)
}

async fn ensure_app(s: &ArgMatches) -> Result<Value, WpsError> {
    let auth = effective_auth_type(s);
    let dry = s.get_flag("dry-run");
    let retry = *s.get_one::<u32>("retry").unwrap_or(&1);
    let app = s.get_one::<String>("app").expect("required");
    let scope = preflight(&auth, dry).await?;
    let drive_id = get_default_drive_id(
        &auth,
        dry,
        retry,
        s.get_one::<String>("drive-id").cloned(),
    )
    .await?;
    let folder = ensure_app_folder(&drive_id, app, &auth, dry, retry).await?;
    if !dry {
        register_app_if_possible(app, &drive_id, &folder);
    }
    Ok(serde_json::json!({
        "ok": true,
        "data": {
            "code": 0,
            "msg": "ok",
            "data": {
                "drive_id": drive_id,
                "app": app,
                "folder": folder,
                "scope_preflight": scope.to_json(),
                "state_paths": state_paths_value()
            }
        }
    }))
}

async fn create_file(s: &ArgMatches) -> Result<Value, WpsError> {
    let auth = effective_auth_type(s);
    let dry = s.get_flag("dry-run");
    let retry = *s.get_one::<u32>("retry").unwrap_or(&1);
    let app = s.get_one::<String>("app").expect("required");
    let file = s.get_one::<String>("file").expect("required");
    let scope = preflight(&auth, dry).await?;
    let on_conflict = s
        .get_one::<String>("on-name-conflict")
        .cloned()
        .unwrap_or_else(|| "fail".to_string());
    let drive_id = get_default_drive_id(
        &auth,
        dry,
        retry,
        s.get_one::<String>("drive-id").cloned(),
    )
    .await?;
    let app_folder = ensure_app_folder(&drive_id, app, &auth, dry, retry).await?;
    let parent_id = app_folder
        .get("id")
        .and_then(|v| v.as_str())
        .unwrap_or("0")
        .to_string();
    let body = serde_json::json!({
        "name": file,
        "file_type": "file",
        "on_name_conflict": on_conflict
    });
    let resp = execute(
        "POST",
        &format!("/v7/drives/{drive_id}/files/{parent_id}/create"),
        HashMap::new(),
        Some(body),
        &auth,
        dry,
        retry,
    )
    .await?;
    if !dry && !api_ok(&resp) {
        return Err(WpsError::Network(format!("创建文件失败: {resp}")));
    }
    let created = api_payload(&resp);
    if !dry {
        register_app_if_possible(app, &drive_id, &app_folder);
        register_file_if_possible(app, &drive_id, &parent_id, &created);
    }
    Ok(serde_json::json!({
        "ok": true,
        "data": {
            "code": 0,
            "msg": "ok",
            "data": {
                "drive_id": drive_id,
                "app": app,
                "parent_id": parent_id,
                "created": created,
                "scope_preflight": scope.to_json(),
                "state_paths": state_paths_value(),
                "workflow": [
                    "resolve_drive",
                    "ensure_app_folder",
                    "create_file",
                    "persist_registry"
                ]
            }
        }
    }))
}

async fn list_apps(s: &ArgMatches) -> Result<Value, WpsError> {
    let auth = effective_auth_type(s);
    let dry = s.get_flag("dry-run");
    let retry = *s.get_one::<u32>("retry").unwrap_or(&1);
    let scope = preflight(&auth, dry).await?;
    let drive_id = get_default_drive_id(
        &auth,
        dry,
        retry,
        s.get_one::<String>("drive-id").cloned(),
    )
    .await?;
    let app_root = ensure_folder(&drive_id, "0", APP_ROOT_NAME, &auth, dry, retry).await?;
    let app_root_id = app_root
        .get("id")
        .and_then(|v| v.as_str())
        .unwrap_or("0")
        .to_string();
    let openclaw = ensure_folder(&drive_id, &app_root_id, OPENCLAW_ROOT_NAME, &auth, dry, retry).await?;
    let openclaw_id = openclaw
        .get("id")
        .and_then(|v| v.as_str())
        .unwrap_or("0")
        .to_string();
    let items = list_all_children(&drive_id, &openclaw_id, &auth, dry, retry).await?;
    let apps = items
        .into_iter()
        .filter(|x| {
            x.get("type").and_then(|v| v.as_str()) == Some("folder")
                || x.get("file_type").and_then(|v| v.as_str()) == Some("folder")
        })
        .collect::<Vec<_>>();
    Ok(serde_json::json!({
        "ok": true,
        "data": {
            "code": 0,
            "msg": "ok",
            "data": {
                "drive_id": drive_id,
                "apps_root_id": openclaw_id,
                "apps": apps,
                "scope_preflight": scope.to_json()
            }
        }
    }))
}

async fn list_files(s: &ArgMatches) -> Result<Value, WpsError> {
    let auth = effective_auth_type(s);
    let dry = s.get_flag("dry-run");
    let retry = *s.get_one::<u32>("retry").unwrap_or(&1);
    let app = s.get_one::<String>("app").expect("required");
    let scope = preflight(&auth, dry).await?;
    let drive_id = get_default_drive_id(
        &auth,
        dry,
        retry,
        s.get_one::<String>("drive-id").cloned(),
    )
    .await?;
    let app_folder = ensure_app_folder(&drive_id, app, &auth, dry, retry).await?;
    let app_folder_id = app_folder
        .get("id")
        .and_then(|v| v.as_str())
        .unwrap_or("0")
        .to_string();
    let items = list_all_children(&drive_id, &app_folder_id, &auth, dry, retry).await?;
    Ok(serde_json::json!({
        "ok": true,
        "data": {
            "code": 0,
            "msg": "ok",
            "data": {
                "drive_id": drive_id,
                "app": app,
                "app_folder_id": app_folder_id,
                "items": items,
                "scope_preflight": scope.to_json()
            }
        }
    }))
}

async fn get_file(s: &ArgMatches) -> Result<Value, WpsError> {
    let auth = effective_auth_type(s);
    let dry = s.get_flag("dry-run");
    let retry = *s.get_one::<u32>("retry").unwrap_or(&1);
    let _scope = preflight(&auth, dry).await?;
    let (drive_id, file_id) = resolve_drive_file(s, &auth, dry, retry).await?;

    execute(
        "GET",
        &format!("/v7/drives/{drive_id}/files/{file_id}/meta"),
        HashMap::new(),
        None,
        &auth,
        dry,
        retry,
    )
    .await
}

async fn resolve_drive_file(
    s: &ArgMatches,
    auth: &str,
    dry: bool,
    retry: u32,
) -> Result<(String, String), WpsError> {
    let mut drive_id = s.get_one::<String>("drive-id").cloned().unwrap_or_default();
    let mut file_id = s.get_one::<String>("file-id").cloned().unwrap_or_default();

    if file_id.is_empty() {
        let app = s
            .get_one::<String>("app")
            .ok_or_else(|| WpsError::Validation("缺少 --file-id 或 (--app + --file)".to_string()))?;
        let file = s
            .get_one::<String>("file")
            .ok_or_else(|| WpsError::Validation("缺少 --file-id 或 (--app + --file)".to_string()))?;
        drive_id = get_default_drive_id(
            auth,
            dry,
            retry,
            if drive_id.is_empty() { None } else { Some(drive_id) },
        )
        .await?;
        let app_folder = ensure_app_folder(&drive_id, app, auth, dry, retry).await?;
        let app_folder_id = app_folder
            .get("id")
            .and_then(|v| v.as_str())
            .unwrap_or("0")
            .to_string();
        let items = list_all_children(&drive_id, &app_folder_id, auth, dry, retry).await?;
        let found = items.iter().find(|x| x.get("name").and_then(|v| v.as_str()) == Some(file));
        if let Some(v) = found {
            file_id = v.get("id").and_then(|x| x.as_str()).unwrap_or("").to_string();
        }
        if file_id.is_empty() {
            return Err(WpsError::Validation(format!("未找到文件: {file}")));
        }
    } else if drive_id.is_empty() {
        drive_id = get_default_drive_id(auth, dry, retry, None).await?;
    }
    Ok((drive_id, file_id))
}

fn file_name_from_path(local_file: &str) -> Result<String, WpsError> {
    Path::new(local_file)
        .file_name()
        .and_then(|v| v.to_str())
        .filter(|v| !v.trim().is_empty())
        .map(|v| v.to_string())
        .ok_or_else(|| WpsError::Validation(format!("无法从路径推断文件名: {local_file}")))
}

fn sha256_file_hex(local_file: &str) -> Result<String, WpsError> {
    let mut f = std::fs::File::open(local_file)
        .map_err(|e| WpsError::Validation(format!("读取本地文件失败 {local_file}: {e}")))?;
    let mut hasher = Sha256::new();
    let mut buf = [0u8; 1024 * 64];
    loop {
        let n = f
            .read(&mut buf)
            .map_err(|e| WpsError::Validation(format!("读取本地文件失败 {local_file}: {e}")))?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    let digest = hasher.finalize();
    Ok(digest.iter().map(|b| format!("{b:02x}")).collect::<String>())
}

fn phase_ok(
    phase: &str,
    started: Instant,
    retryable: bool,
    suggested_action: &str,
    detail: Value,
) -> Value {
    serde_json::json!({
        "phase": phase,
        "status": "ok",
        "duration_ms": started.elapsed().as_millis() as u64,
        "retryable": retryable,
        "suggested_action": suggested_action,
        "detail": detail
    })
}

fn phase_failed(
    phase: &str,
    started: Instant,
    retryable: bool,
    suggested_action: &str,
    error: &WpsError,
) -> Value {
    serde_json::json!({
        "phase": phase,
        "status": "failed",
        "duration_ms": started.elapsed().as_millis() as u64,
        "retryable": retryable,
        "suggested_action": suggested_action,
        "error": {
            "code": error.code(),
            "message": error.to_string(),
        }
    })
}

fn transfer_summary(phases: &[Value]) -> Value {
    let failed = phases
        .iter()
        .find(|p| p.get("status").and_then(|v| v.as_str()) == Some("failed"))
        .and_then(|p| p.get("phase").and_then(|v| v.as_str()))
        .map(|s| s.to_string());
    serde_json::json!({
        "phase_count": phases.len(),
        "has_failed_phase": failed.is_some(),
        "failed_phase": failed,
    })
}

fn transfer_failure(mode: &str, phases: Vec<Value>, error: &WpsError) -> Value {
    serde_json::json!({
        "ok": false,
        "data": {
            "code": 1,
            "msg": "transfer_failed",
            "data": {
                "mode": mode,
                "summary": transfer_summary(&phases),
                "phases": phases,
                "error": {
                    "code": error.code(),
                    "message": error.to_string()
                }
            }
        }
    })
}

async fn transfer(s: &ArgMatches) -> Result<Value, WpsError> {
    let mode = s
        .get_one::<String>("mode")
        .cloned()
        .unwrap_or_else(|| "upload".to_string());
    match mode.as_str() {
        "upload" => transfer_upload(s).await,
        "download" => transfer_download(s).await,
        _ => Err(WpsError::Validation(format!("不支持的 transfer mode: {mode}"))),
    }
}

async fn transfer_upload(s: &ArgMatches) -> Result<Value, WpsError> {
    let mut phases = Vec::new();
    let auth = effective_auth_type(s);
    let dry = s.get_flag("dry-run");
    let retry = *s.get_one::<u32>("retry").unwrap_or(&1);

    let t = Instant::now();
    let scope = match preflight(&auth, dry).await {
        Ok(v) => {
            phases.push(phase_ok(
                "scope_preflight",
                t,
                false,
                "若缺少 scope，请执行 wpscli auth login --user 并补齐 kso.file.readwrite",
                v.to_json(),
            ));
            v
        }
        Err(e) => {
            phases.push(phase_failed(
                "scope_preflight",
                t,
                false,
                "执行 wpscli auth status / auth login 检查授权",
                &e,
            ));
            return Ok(transfer_failure("upload", phases, &e));
        }
    };

    let local_file = s.get_one::<String>("local-file").expect("required");
    let t = Instant::now();
    let meta = match std::fs::metadata(local_file) {
        Ok(m) if m.is_file() => {
            phases.push(phase_ok(
                "local_file_check",
                t,
                false,
                "确认本地路径存在且可读",
                serde_json::json!({"local_file": local_file, "size": m.len()}),
            ));
            m
        }
        Ok(_) => {
            let e = WpsError::Validation(format!("不是有效文件: {local_file}"));
            phases.push(phase_failed(
                "local_file_check",
                t,
                false,
                "传入可读文件路径，例如 --local-file /path/a.txt",
                &e,
            ));
            return Ok(transfer_failure("upload", phases, &e));
        }
        Err(err) => {
            let e = WpsError::Validation(format!("读取本地文件元信息失败 {local_file}: {err}"));
            phases.push(phase_failed(
                "local_file_check",
                t,
                false,
                "检查文件路径和权限",
                &e,
            ));
            return Ok(transfer_failure("upload", phases, &e));
        }
    };
    let size = meta.len();

    let file_name = s
        .get_one::<String>("name")
        .cloned()
        .unwrap_or(file_name_from_path(local_file)?);
    let on_name_conflict = s
        .get_one::<String>("on-name-conflict")
        .cloned()
        .unwrap_or_else(|| "rename".to_string());

    let t = Instant::now();
    let drive_id = match get_default_drive_id(
        &auth,
        dry,
        retry,
        s.get_one::<String>("drive-id").cloned(),
    )
    .await
    {
        Ok(v) => {
            phases.push(phase_ok(
                "resolve_drive",
                t,
                true,
                "若失败可重试，或显式传 --drive-id",
                serde_json::json!({"drive_id": v}),
            ));
            v
        }
        Err(e) => {
            phases.push(phase_failed(
                "resolve_drive",
                t,
                true,
                "重试或手动指定 --drive-id",
                &e,
            ));
            return Ok(transfer_failure("upload", phases, &e));
        }
    };

    let app = s.get_one::<String>("app").cloned();
    let t = Instant::now();
    let parent_id = if let Some(pid) = s.get_one::<String>("parent-id").cloned() {
        phases.push(phase_ok(
            "resolve_parent",
            t,
            false,
            "已使用显式 parent-id",
            serde_json::json!({"parent_id": pid}),
        ));
        pid
    } else if let Some(app_name) = &app {
        match ensure_app_folder(&drive_id, app_name, &auth, dry, retry).await {
            Ok(folder) => {
                let pid = folder
                    .get("id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("0")
                    .to_string();
                phases.push(phase_ok(
                    "resolve_parent",
                    t,
                    true,
                    "应用目录不存在时会自动创建",
                    serde_json::json!({"app": app_name, "parent_id": pid, "folder": folder}),
                ));
                pid
            }
            Err(e) => {
                phases.push(phase_failed(
                    "resolve_parent",
                    t,
                    true,
                    "检查目录权限，或改用 --parent-id",
                    &e,
                ));
                return Ok(transfer_failure("upload", phases, &e));
            }
        }
    } else {
        phases.push(phase_ok(
            "resolve_parent",
            t,
            false,
            "默认上传到根目录 parent_id=0",
            serde_json::json!({"parent_id":"0"}),
        ));
        "0".to_string()
    };

    let sha256 = sha256_file_hex(local_file)?;
    let mut req_body = serde_json::json!({
        "name": file_name,
        "size": size,
        "on_name_conflict": on_name_conflict,
        "hashes": [{"type":"sha256","sum": sha256}],
    });
    if s.get_flag("internal") {
        req_body["internal"] = Value::Bool(true);
    }

    let t = Instant::now();
    let request_resp = match execute(
        "POST",
        &format!("/v7/drives/{drive_id}/files/{parent_id}/request_upload"),
        HashMap::new(),
        Some(req_body),
        &auth,
        dry,
        retry,
    )
    .await
    {
        Ok(v) => {
            phases.push(phase_ok(
                "request_upload",
                t,
                true,
                "若报 scope 或参数错误，请检查鉴权与上传参数",
                serde_json::json!({"status_ok": api_ok(&v)}),
            ));
            v
        }
        Err(e) => {
            phases.push(phase_failed(
                "request_upload",
                t,
                true,
                "重试；若持续失败请检查 scope/parent_id/name/size",
                &e,
            ));
            return Ok(transfer_failure("upload", phases, &e));
        }
    };
    if dry {
        return Ok(serde_json::json!({
            "ok": true,
            "data": {
                "code": 0,
                "msg": "dry-run",
                "data": {
                    "mode": "upload",
                    "summary": transfer_summary(&phases),
                    "phases": phases,
                    "scope_preflight": scope.to_json(),
                    "request_upload": request_resp,
                    "workflow": ["request_upload", "upload_to_store", "commit_upload"]
                }
            }
        }));
    }
    if !api_ok(&request_resp) {
        let e = WpsError::Network(format!("请求文件上传信息失败: {request_resp}"));
        return Ok(transfer_failure("upload", phases, &e));
    }

    let request_payload = api_payload(&request_resp);
    let upload_id = match request_payload.get("upload_id").and_then(|v| v.as_str()) {
        Some(v) => v.to_string(),
        None => {
            let e = WpsError::Network(format!("上传响应缺少 upload_id: {request_resp}"));
            return Ok(transfer_failure("upload", phases, &e));
        }
    };
    let store_request = request_payload
        .get("store_request")
        .cloned()
        .unwrap_or(Value::Null);
    let upload_url = match store_request.get("url").and_then(|v| v.as_str()) {
        Some(v) => v.to_string(),
        None => {
            let e = WpsError::Network(format!("上传响应缺少 store_request.url: {request_resp}"));
            return Ok(transfer_failure("upload", phases, &e));
        }
    };
    let upload_method = store_request
        .get("method")
        .and_then(|v| v.as_str())
        .unwrap_or("PUT")
        .to_uppercase();
    let method = match Method::from_bytes(upload_method.as_bytes()) {
        Ok(v) => v,
        Err(err) => {
            let e = WpsError::Validation(format!("非法上传方法 {upload_method}: {err}"));
            return Ok(transfer_failure("upload", phases, &e));
        }
    };

    let t = Instant::now();
    let file_bytes = match tokio::fs::read(local_file).await {
        Ok(v) => v,
        Err(err) => {
            let e = WpsError::Validation(format!("读取本地文件失败 {local_file}: {err}"));
            phases.push(phase_failed(
                "upload_to_store",
                t,
                false,
                "检查本地文件权限",
                &e,
            ));
            return Ok(transfer_failure("upload", phases, &e));
        }
    };
    let mut req = reqwest::Client::new()
        .request(method, &upload_url)
        .header("Content-Type", "application/octet-stream");
    if let Some(headers) = store_request.get("headers").and_then(|v| v.as_object()) {
        for (k, v) in headers {
            if let Some(vs) = v.as_str() {
                req = req.header(k, vs);
            }
        }
    }
    let upload_resp = match req.body(file_bytes).send().await {
        Ok(v) => v,
        Err(err) => {
            let e = WpsError::Network(format!("上传实体文件失败: {err}"));
            phases.push(phase_failed(
                "upload_to_store",
                t,
                true,
                "可重试；检查上传地址是否过期，必要时重新 request_upload",
                &e,
            ));
            return Ok(transfer_failure("upload", phases, &e));
        }
    };
    let upload_status = upload_resp.status().as_u16();
    let upload_text = upload_resp.text().await.unwrap_or_default();
    if upload_status >= 400 {
        let e = WpsError::Network(format!(
            "上传实体文件失败(status={upload_status}): {upload_text}"
        ));
        phases.push(phase_failed(
            "upload_to_store",
            t,
            true,
            "重试；若地址失效请重新执行 request_upload",
            &e,
        ));
        return Ok(transfer_failure("upload", phases, &e));
    }
    phases.push(phase_ok(
        "upload_to_store",
        t,
        false,
        "实体文件上传成功",
        serde_json::json!({"status": upload_status}),
    ));

    let t = Instant::now();
    let commit_resp = match execute(
        "POST",
        &format!("/v7/drives/{drive_id}/files/{parent_id}/commit_upload"),
        HashMap::new(),
        Some(serde_json::json!({ "upload_id": upload_id })),
        &auth,
        dry,
        retry,
    )
    .await
    {
        Ok(v) => v,
        Err(e) => {
            phases.push(phase_failed(
                "commit_upload",
                t,
                true,
                "可重试 commit_upload（upload_id 短时间内有效）",
                &e,
            ));
            return Ok(transfer_failure("upload", phases, &e));
        }
    };
    if !api_ok(&commit_resp) {
        let e = WpsError::Network(format!("提交上传完成失败: {commit_resp}"));
        phases.push(phase_failed(
            "commit_upload",
            t,
            true,
            "检查 upload_id 或重新执行上传流程",
            &e,
        ));
        return Ok(transfer_failure("upload", phases, &e));
    }
    phases.push(phase_ok(
        "commit_upload",
        t,
        false,
        "上传已完成并生成文件元信息",
        serde_json::json!({"status_ok": true}),
    ));

    let created = api_payload(&commit_resp);
    if let Some(app_name) = &app {
        register_file_if_possible(app_name, &drive_id, &parent_id, &created);
    }
    Ok(serde_json::json!({
        "ok": true,
        "data": {
            "code": 0,
            "msg": "ok",
            "data": {
                "mode": "upload",
                "summary": transfer_summary(&phases),
                "phases": phases,
                "scope_preflight": scope.to_json(),
                "result": {
                    "drive_id": drive_id,
                    "parent_id": parent_id,
                    "app": app,
                    "file_name": file_name,
                    "size": size,
                    "sha256": sha256,
                    "upload_id": upload_id,
                    "store_request_host": reqwest::Url::parse(&upload_url).ok().and_then(|u| u.host_str().map(|s| s.to_string())),
                    "created": created,
                }
            }
        }
    }))
}

async fn transfer_download(s: &ArgMatches) -> Result<Value, WpsError> {
    let mut phases = Vec::new();
    let auth = effective_auth_type(s);
    let dry = s.get_flag("dry-run");
    let retry = *s.get_one::<u32>("retry").unwrap_or(&1);

    let t = Instant::now();
    let scope = match preflight(&auth, dry).await {
        Ok(v) => {
            phases.push(phase_ok(
                "scope_preflight",
                t,
                false,
                "若缺少 scope，请执行 wpscli auth login --user 并补齐 kso.file.read",
                v.to_json(),
            ));
            v
        }
        Err(e) => {
            phases.push(phase_failed(
                "scope_preflight",
                t,
                false,
                "执行 wpscli auth status / auth login 检查授权",
                &e,
            ));
            return Ok(transfer_failure("download", phases, &e));
        }
    };

    let t = Instant::now();
    let (drive_id, file_id) = match resolve_drive_file(s, &auth, dry, retry).await {
        Ok(v) => {
            phases.push(phase_ok(
                "resolve_target",
                t,
                true,
                "可重试，或显式传 --drive-id --file-id",
                serde_json::json!({"drive_id": v.0, "file_id": v.1}),
            ));
            v
        }
        Err(e) => {
            phases.push(phase_failed(
                "resolve_target",
                t,
                false,
                "检查 --file-id 或 --app + --file 参数",
                &e,
            ));
            return Ok(transfer_failure("download", phases, &e));
        }
    };

    let mut query = HashMap::new();
    if s.get_flag("with-hash") {
        query.insert("with_hash".to_string(), "true".to_string());
    }
    if s.get_flag("internal") {
        query.insert("internal".to_string(), "true".to_string());
    }
    if let Some(base_domain) = s.get_one::<String>("storage-base-domain") {
        query.insert("storage_base_domain".to_string(), base_domain.clone());
    }

    let t = Instant::now();
    let info_resp = match execute(
        "GET",
        &format!("/v7/drives/{drive_id}/files/{file_id}/download"),
        query,
        None,
        &auth,
        dry,
        retry,
    )
    .await
    {
        Ok(v) => {
            phases.push(phase_ok(
                "get_download_info",
                t,
                true,
                "若失败可重试，或检查文件权限",
                serde_json::json!({"status_ok": api_ok(&v)}),
            ));
            v
        }
        Err(e) => {
            phases.push(phase_failed(
                "get_download_info",
                t,
                true,
                "重试；若 403 请检查文件访问权限",
                &e,
            ));
            return Ok(transfer_failure("download", phases, &e));
        }
    };
    if dry {
        return Ok(serde_json::json!({
            "ok": true,
            "data": {
                "code": 0,
                "msg": "dry-run",
                "data": {
                    "mode": "download",
                    "summary": transfer_summary(&phases),
                    "phases": phases,
                    "scope_preflight": scope.to_json(),
                    "request_download_info": info_resp
                }
            }
        }));
    }
    if !api_ok(&info_resp) {
        let e = WpsError::Network(format!("获取下载信息失败: {info_resp}"));
        return Ok(transfer_failure("download", phases, &e));
    }
    let payload = api_payload(&info_resp);
    let download_url = match payload.get("url").and_then(|v| v.as_str()) {
        Some(v) => v.to_string(),
        None => {
            let e = WpsError::Network(format!("下载响应缺少 url: {info_resp}"));
            return Ok(transfer_failure("download", phases, &e));
        }
    };

    let t = Instant::now();
    let meta_resp = match fetch_file_meta(&drive_id, &file_id, &auth, dry, retry).await {
        Ok(v) => {
            phases.push(phase_ok(
                "fetch_meta",
                t,
                true,
                "获取文件名用于落盘",
                serde_json::json!({"status_ok": api_ok(&v)}),
            ));
            v
        }
        Err(e) => {
            phases.push(phase_failed(
                "fetch_meta",
                t,
                true,
                "可重试；失败时可通过 --output 显式给出文件名",
                &e,
            ));
            return Ok(transfer_failure("download", phases, &e));
        }
    };
    let payload_meta = api_payload(&meta_resp);
    let default_name = payload_meta
        .get("name")
        .and_then(|v| v.as_str())
        .filter(|v| !v.trim().is_empty())
        .unwrap_or("download.bin")
        .to_string();

    let t = Instant::now();
    let output_path = match output_path_for_download(
        s.get_one::<String>("output"),
        &default_name,
        s.get_flag("overwrite"),
    ) {
        Ok(v) => {
            phases.push(phase_ok(
                "resolve_output",
                t,
                false,
                "已生成本地保存路径",
                serde_json::json!({"output": v.display().to_string()}),
            ));
            v
        }
        Err(e) => {
            phases.push(phase_failed(
                "resolve_output",
                t,
                false,
                "检查本地路径权限或使用 --overwrite",
                &e,
            ));
            return Ok(transfer_failure("download", phases, &e));
        }
    };

    let t = Instant::now();
    let client = reqwest::Client::new();
    let mut resp = match client.get(&download_url).send().await {
        Ok(v) => v,
        Err(err) => {
            let e = WpsError::Network(format!("下载文件失败: {err}"));
            phases.push(phase_failed(
                "download_stream",
                t,
                true,
                "可重试；若下载地址过期请重新获取 download 信息",
                &e,
            ));
            return Ok(transfer_failure("download", phases, &e));
        }
    };
    let status = resp.status().as_u16();
    if status >= 400 {
        let txt = resp.text().await.unwrap_or_default();
        let e = WpsError::Network(format!("下载文件失败(status={status}): {txt}"));
        phases.push(phase_failed(
            "download_stream",
            t,
            true,
            "可重试；若失败持续请重新获取下载地址",
            &e,
        ));
        return Ok(transfer_failure("download", phases, &e));
    }
    let mut file = match tokio::fs::File::create(&output_path).await {
        Ok(v) => v,
        Err(err) => {
            let e = WpsError::Validation(format!(
                "创建本地文件失败 {}: {err}",
                output_path.display()
            ));
            phases.push(phase_failed(
                "download_stream",
                t,
                false,
                "检查输出目录权限",
                &e,
            ));
            return Ok(transfer_failure("download", phases, &e));
        }
    };
    let mut bytes: u64 = 0;
    while let Some(chunk) = match resp.chunk().await {
        Ok(v) => v,
        Err(err) => {
            let e = WpsError::Network(format!("读取下载数据块失败: {err}"));
            phases.push(phase_failed(
                "download_stream",
                t,
                true,
                "可重试下载，或重新获取下载地址",
                &e,
            ));
            return Ok(transfer_failure("download", phases, &e));
        }
    } {
        if let Err(err) = file.write_all(&chunk).await {
            let e = WpsError::Network(format!("写入本地文件失败 {}: {err}", output_path.display()));
            phases.push(phase_failed(
                "download_stream",
                t,
                false,
                "检查磁盘空间和写权限",
                &e,
            ));
            return Ok(transfer_failure("download", phases, &e));
        }
        bytes += chunk.len() as u64;
    }
    if let Err(err) = file.flush().await {
        let e = WpsError::Network(format!("刷新本地文件失败 {}: {err}", output_path.display()));
        phases.push(phase_failed(
            "download_stream",
            t,
            false,
            "检查磁盘写入状态",
            &e,
        ));
        return Ok(transfer_failure("download", phases, &e));
    }
    phases.push(phase_ok(
        "download_stream",
        t,
        false,
        "下载并写入本地完成",
        serde_json::json!({"bytes": bytes, "saved_file": output_path.display().to_string()}),
    ));

    Ok(serde_json::json!({
        "ok": true,
        "data": {
            "code": 0,
            "msg": "ok",
            "data": {
                "mode": "download",
                "summary": transfer_summary(&phases),
                "phases": phases,
                "scope_preflight": scope.to_json(),
                "result": {
                    "drive_id": drive_id,
                    "file_id": file_id,
                    "download_url_host": reqwest::Url::parse(&download_url).ok().and_then(|u| u.host_str().map(|s| s.to_string())),
                    "saved_file": output_path.display().to_string(),
                    "bytes": bytes,
                    "hashes": payload.get("hashes").cloned().unwrap_or(Value::Array(vec![])),
                }
            }
        }
    }))
}

async fn upload_file(s: &ArgMatches) -> Result<Value, WpsError> {
    let auth = effective_auth_type(s);
    let dry = s.get_flag("dry-run");
    let retry = *s.get_one::<u32>("retry").unwrap_or(&1);
    let scope = preflight(&auth, dry).await?;
    let local_file = s.get_one::<String>("local-file").expect("required");
    let meta = std::fs::metadata(local_file)
        .map_err(|e| WpsError::Validation(format!("读取本地文件元信息失败 {local_file}: {e}")))?;
    if !meta.is_file() {
        return Err(WpsError::Validation(format!("不是有效文件: {local_file}")));
    }
    let size = meta.len();
    let file_name = s
        .get_one::<String>("name")
        .cloned()
        .unwrap_or(file_name_from_path(local_file)?);
    let on_name_conflict = s
        .get_one::<String>("on-name-conflict")
        .cloned()
        .unwrap_or_else(|| "rename".to_string());
    let drive_id = get_default_drive_id(
        &auth,
        dry,
        retry,
        s.get_one::<String>("drive-id").cloned(),
    )
    .await?;
    let app = s.get_one::<String>("app").cloned();
    let parent_id = if let Some(pid) = s.get_one::<String>("parent-id").cloned() {
        pid
    } else if let Some(app_name) = &app {
        ensure_app_folder(&drive_id, app_name, &auth, dry, retry)
            .await?
            .get("id")
            .and_then(|v| v.as_str())
            .unwrap_or("0")
            .to_string()
    } else {
        "0".to_string()
    };
    let sha256 = sha256_file_hex(local_file)?;
    let mut req_body = serde_json::json!({
        "name": file_name,
        "size": size,
        "on_name_conflict": on_name_conflict,
        "hashes": [{"type":"sha256","sum": sha256}],
    });
    if s.get_flag("internal") {
        req_body["internal"] = Value::Bool(true);
    }

    let request_resp = execute(
        "POST",
        &format!("/v7/drives/{drive_id}/files/{parent_id}/request_upload"),
        HashMap::new(),
        Some(req_body),
        &auth,
        dry,
        retry,
    )
    .await?;
    if dry {
        return Ok(serde_json::json!({
            "ok": true,
            "data": {
                "code": 0,
                "msg": "dry-run",
                "data": {
                    "drive_id": drive_id,
                    "parent_id": parent_id,
                    "file_name": file_name,
                    "size": size,
                    "sha256": sha256,
                    "request_upload": request_resp,
                    "workflow": ["request_upload", "put_to_store_request.url", "commit_upload"],
                    "scope_preflight": scope.to_json()
                }
            }
        }));
    }
    if !api_ok(&request_resp) {
        return Err(WpsError::Network(format!("请求文件上传信息失败: {request_resp}")));
    }
    let request_payload = api_payload(&request_resp);
    let upload_id = request_payload
        .get("upload_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| WpsError::Network(format!("上传响应缺少 upload_id: {request_resp}")))?;
    let store_request = request_payload
        .get("store_request")
        .cloned()
        .unwrap_or(Value::Null);
    let upload_url = store_request
        .get("url")
        .and_then(|v| v.as_str())
        .ok_or_else(|| WpsError::Network(format!("上传响应缺少 store_request.url: {request_resp}")))?;
    let upload_method = store_request
        .get("method")
        .and_then(|v| v.as_str())
        .unwrap_or("PUT")
        .to_uppercase();
    let method = Method::from_bytes(upload_method.as_bytes())
        .map_err(|e| WpsError::Validation(format!("非法上传方法 {upload_method}: {e}")))?;
    let file_bytes = tokio::fs::read(local_file)
        .await
        .map_err(|e| WpsError::Validation(format!("读取本地文件失败 {local_file}: {e}")))?;

    let mut req = reqwest::Client::new()
        .request(method, upload_url)
        .header("Content-Type", "application/octet-stream");
    if let Some(headers) = store_request.get("headers").and_then(|v| v.as_object()) {
        for (k, v) in headers {
            if let Some(vs) = v.as_str() {
                req = req.header(k, vs);
            }
        }
    }
    let upload_resp = req
        .body(file_bytes.clone())
        .send()
        .await
        .map_err(|e| WpsError::Network(format!("上传实体文件失败: {e}")))?;
    let upload_status = upload_resp.status().as_u16();
    let upload_text = upload_resp.text().await.unwrap_or_default();
    if upload_status >= 400 {
        return Err(WpsError::Network(format!(
            "上传实体文件失败(status={upload_status}): {upload_text}"
        )));
    }

    let commit_resp = execute(
        "POST",
        &format!("/v7/drives/{drive_id}/files/{parent_id}/commit_upload"),
        HashMap::new(),
        Some(serde_json::json!({ "upload_id": upload_id })),
        &auth,
        dry,
        retry,
    )
    .await?;
    if !api_ok(&commit_resp) {
        return Err(WpsError::Network(format!("提交上传完成失败: {commit_resp}")));
    }
    let created = api_payload(&commit_resp);
    if let Some(app_name) = &app {
        register_file_if_possible(app_name, &drive_id, &parent_id, &created);
    }

    Ok(serde_json::json!({
        "ok": true,
        "data": {
            "code": 0,
            "msg": "ok",
            "data": {
                "drive_id": drive_id,
                "parent_id": parent_id,
                "app": app,
                "file_name": file_name,
                "size": size,
                "sha256": sha256,
                "upload_id": upload_id,
                "store_request": {
                    "method": upload_method,
                    "host": reqwest::Url::parse(upload_url).ok().and_then(|u| u.host_str().map(|s| s.to_string())),
                    "status": upload_status
                },
                "created": created,
                "scope_preflight": scope.to_json(),
                "workflow": ["request_upload", "upload_to_store", "commit_upload", "persist_registry"]
            }
        }
    }))
}

async fn fetch_file_meta(
    drive_id: &str,
    file_id: &str,
    auth: &str,
    dry: bool,
    retry: u32,
) -> Result<Value, WpsError> {
    execute(
        "GET",
        &format!("/v7/drives/{drive_id}/files/{file_id}/meta"),
        HashMap::new(),
        None,
        auth,
        dry,
        retry,
    )
    .await
}

fn output_path_for_download(
    output: Option<&String>,
    default_name: &str,
    overwrite: bool,
) -> Result<std::path::PathBuf, WpsError> {
    let mut path = if let Some(out) = output {
        let p = std::path::PathBuf::from(out);
        if p.exists() && p.is_dir() {
            p.join(default_name)
        } else {
            p
        }
    } else {
        std::env::current_dir()
            .map_err(|e| WpsError::Validation(format!("获取当前目录失败: {e}")))?
            .join(default_name)
    };
    if path.is_dir() {
        path = path.join(default_name);
    }
    if path.exists() && !overwrite {
        return Err(WpsError::Validation(format!(
            "目标文件已存在: {}（可加 --overwrite）",
            path.display()
        )));
    }
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent).map_err(|e| {
                WpsError::Validation(format!("创建下载目录失败 {}: {e}", parent.display()))
            })?;
        }
    }
    Ok(path)
}

async fn download_file(s: &ArgMatches) -> Result<Value, WpsError> {
    let auth = effective_auth_type(s);
    let dry = s.get_flag("dry-run");
    let retry = *s.get_one::<u32>("retry").unwrap_or(&1);
    let scope = preflight(&auth, dry).await?;
    let (drive_id, file_id) = resolve_drive_file(s, &auth, dry, retry).await?;

    let mut query = HashMap::new();
    if s.get_flag("with-hash") {
        query.insert("with_hash".to_string(), "true".to_string());
    }
    if s.get_flag("internal") {
        query.insert("internal".to_string(), "true".to_string());
    }
    if let Some(base_domain) = s.get_one::<String>("storage-base-domain") {
        query.insert("storage_base_domain".to_string(), base_domain.clone());
    }

    let info_resp = execute(
        "GET",
        &format!("/v7/drives/{drive_id}/files/{file_id}/download"),
        query,
        None,
        &auth,
        dry,
        retry,
    )
    .await?;
    if dry {
        return Ok(serde_json::json!({
            "ok": true,
            "data": {
                "code": 0,
                "msg": "dry-run",
                "data": {
                    "drive_id": drive_id,
                    "file_id": file_id,
                    "request_download_info": info_resp,
                    "workflow": ["get_download_info", "http_get_download_url", "save_local_file"],
                    "scope_preflight": scope.to_json()
                }
            }
        }));
    }
    if !api_ok(&info_resp) {
        return Err(WpsError::Network(format!("获取下载信息失败: {info_resp}")));
    }
    let payload = api_payload(&info_resp);
    let download_url = payload
        .get("url")
        .and_then(|v| v.as_str())
        .ok_or_else(|| WpsError::Network(format!("下载响应缺少 url: {info_resp}")))?;

    let meta_resp = fetch_file_meta(&drive_id, &file_id, &auth, dry, retry).await?;
    let default_name = api_payload(&meta_resp)
        .get("name")
        .and_then(|v| v.as_str())
        .filter(|v| !v.trim().is_empty())
        .unwrap_or("download.bin")
        .to_string();
    let output_path = output_path_for_download(
        s.get_one::<String>("output"),
        &default_name,
        s.get_flag("overwrite"),
    )?;

    let client = reqwest::Client::new();
    let mut resp = client
        .get(download_url)
        .send()
        .await
        .map_err(|e| WpsError::Network(format!("下载文件失败: {e}")))?;
    let status = resp.status().as_u16();
    if status >= 400 {
        let txt = resp.text().await.unwrap_or_default();
        return Err(WpsError::Network(format!(
            "下载文件失败(status={status}): {txt}"
        )));
    }
    let mut file = tokio::fs::File::create(&output_path).await.map_err(|e| {
        WpsError::Validation(format!(
            "创建本地文件失败 {}: {e}",
            output_path.display()
        ))
    })?;
    let mut bytes: u64 = 0;
    while let Some(chunk) = resp
        .chunk()
        .await
        .map_err(|e| WpsError::Network(format!("读取下载数据块失败: {e}")))?
    {
        file.write_all(&chunk).await.map_err(|e| {
            WpsError::Network(format!("写入本地文件失败 {}: {e}", output_path.display()))
        })?;
        bytes += chunk.len() as u64;
    }
    file.flush().await.map_err(|e| {
        WpsError::Network(format!("刷新本地文件失败 {}: {e}", output_path.display()))
    })?;

    Ok(serde_json::json!({
        "ok": true,
        "data": {
            "code": 0,
            "msg": "ok",
            "data": {
                "drive_id": drive_id,
                "file_id": file_id,
                "download_url_host": reqwest::Url::parse(download_url).ok().and_then(|u| u.host_str().map(|s| s.to_string())),
                "saved_file": output_path.display().to_string(),
                "bytes": bytes,
                "hashes": payload.get("hashes").cloned().unwrap_or(Value::Array(vec![])),
                "scope_preflight": scope.to_json(),
                "workflow": ["get_download_info", "download_file", "write_local_file"]
            }
        }
    }))
}
