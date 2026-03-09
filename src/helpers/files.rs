use std::collections::HashMap;

use clap::{Arg, ArgAction, ArgMatches, Command};
use serde_json::Value;

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
    let status = if result.is_ok() { "ok" } else { "error" };
    let _ = store.append_operation_log(action, status, detail);
}

fn preview_json(v: &Value, max_len: usize) -> Value {
    let s = serde_json::to_string(v).unwrap_or_else(|_| "{}".to_string());
    if s.len() <= max_len {
        v.clone()
    } else {
        serde_json::json!({
            "truncated": true,
            "size": s.len(),
            "head": &s[..max_len]
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
    let mut drive_id = s.get_one::<String>("drive-id").cloned().unwrap_or_default();
    let mut file_id = s.get_one::<String>("file-id").cloned().unwrap_or_default();

    if file_id.is_empty() {
        let app = s
            .get_one::<String>("app")
            .ok_or_else(|| WpsError::Validation("缺少 --file-id 或 (--app + --file)".to_string()))?;
        let file = s
            .get_one::<String>("file")
            .ok_or_else(|| WpsError::Validation("缺少 --file-id 或 (--app + --file)".to_string()))?;
        drive_id = get_default_drive_id(&auth, dry, retry, if drive_id.is_empty() { None } else { Some(drive_id) }).await?;
        let app_folder = ensure_app_folder(&drive_id, app, &auth, dry, retry).await?;
        let app_folder_id = app_folder
            .get("id")
            .and_then(|v| v.as_str())
            .unwrap_or("0")
            .to_string();
        let items = list_all_children(&drive_id, &app_folder_id, &auth, dry, retry).await?;
        let found = items.iter().find(|x| x.get("name").and_then(|v| v.as_str()) == Some(file));
        if let Some(v) = found {
            file_id = v.get("id").and_then(|x| x.as_str()).unwrap_or("").to_string();
        }
        if file_id.is_empty() {
            return Err(WpsError::Validation(format!("未找到文件: {file}")));
        }
    } else if drive_id.is_empty() {
        drive_id = get_default_drive_id(&auth, dry, retry, None).await?;
    }

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
