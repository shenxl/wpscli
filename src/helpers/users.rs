use std::collections::{HashMap, HashSet, VecDeque};
use std::path::PathBuf;

use clap::{Arg, ArgAction, ArgMatches, Command};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::error::WpsError;
use crate::executor;
use crate::scope_catalog::{self, ScopeType};
use crate::skill_runtime::SkillStateStore;

const SKILL_NAME: &str = "wps-users";
const ORG_CACHE_FILE: &str = "org_cache.json";
const DEFAULT_MEMBER_STATUS: &str = "active,notactive,disabled";
const DEFAULT_CACHE_TTL_SECONDS: u64 = 60 * 60 * 6;
const DEFAULT_QUERY_MAX_DEPTS: u32 = 500;
const USER_LIST_STATUSES: [&str; 3] = ["active", "notactive", "disabled"];
const REMOTE_SCAN_MAX_PAGES_PER_STATUS: u32 = 80;
const SYNC_MAX_CHILD_PAGES_PER_DEPT: u32 = 60;
const SYNC_MAX_MEMBER_PAGES_PER_DEPT: u32 = 60;
const USERS_REQUIRED_DELEGATED_SCOPES: [&str; 2] = ["kso.contact.read", "kso.contact.readwrite"];
const USERS_REQUIRED_APP_ROLE_SCOPES: [&str; 2] = ["kso.contact.read", "kso.contact.readwrite"];

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct OrgCache {
    #[serde(default)]
    generated_at: String,
    #[serde(default)]
    generated_ts: u64,
    #[serde(default)]
    ttl_seconds: u64,
    #[serde(default)]
    scope_mode: String,
    #[serde(default)]
    scope_error: Option<String>,
    #[serde(default)]
    scope_guidance: Option<Value>,
    #[serde(default)]
    roots: Vec<String>,
    #[serde(default)]
    depts: Vec<Value>,
    #[serde(default)]
    members_by_dept: HashMap<String, Vec<Value>>,
    #[serde(default)]
    users_by_id: HashMap<String, Value>,
    #[serde(default)]
    user_scan_warnings: Vec<Value>,
}

#[derive(Debug, Clone, Default)]
struct RemoteUserScanResult {
    users: Vec<Value>,
    warnings: Vec<Value>,
}

pub fn command() -> Command {
    Command::new("users")
        .about("用户与组织架构助手（wps-users）")
        .after_help(
            "示例：\n  \
             wpscli users sync --auth-type app --max-depts 300\n  \
             wpscli users cache-status\n  \
             wpscli users find --name 张三 --auth-type app\n  \
             wpscli users members --dept-id root --recursive true --auth-type app",
        )
        .subcommand(with_common_opts(Command::new("scope").about("查看通讯录权限范围（org）")))
        .subcommand(
            with_cache_opts(with_common_opts(Command::new("depts").about("列出指定部门的子部门（优先缓存）")))
                .arg(Arg::new("dept-id").long("dept-id").default_value("root").help("部门 ID，默认 root"))
                .arg(
                    Arg::new("page-size")
                        .long("page-size")
                        .default_value("50")
                        .value_parser(clap::value_parser!(u32))
                        .help("每页数量"),
                )
                .arg(Arg::new("page-token").long("page-token").num_args(1).help("翻页游标")),
        )
        .subcommand(
            with_cache_opts(with_common_opts(Command::new("members").about("列出部门成员（优先缓存）")))
                .arg(Arg::new("dept-id").long("dept-id").required(true).num_args(1).help("部门 ID"))
                .arg(
                    Arg::new("status")
                        .long("status")
                        .default_value(DEFAULT_MEMBER_STATUS)
                        .help("成员状态，逗号分隔"),
                )
                .arg(
                    Arg::new("recursive")
                        .long("recursive")
                        .default_value("false")
                        .value_parser(["true", "false"])
                        .help("是否递归子部门"),
                )
                .arg(
                    Arg::new("with-user-detail")
                        .long("with-user-detail")
                        .default_value("true")
                        .value_parser(["true", "false"])
                        .help("是否返回用户详细信息"),
                )
                .arg(
                    Arg::new("page-size")
                        .long("page-size")
                        .default_value("50")
                        .value_parser(clap::value_parser!(u32))
                        .help("每页数量"),
                )
                .arg(Arg::new("page-token").long("page-token").num_args(1).help("翻页游标")),
        )
        .subcommand(
            with_cache_opts(with_common_opts(Command::new("user").about("查询指定用户详情（优先缓存）")))
                .arg(Arg::new("user-id").long("user-id").required(true).num_args(1).help("用户 ID"))
                .arg(
                    Arg::new("with-dept")
                        .long("with-dept")
                        .default_value("false")
                        .value_parser(["true", "false"])
                        .help("是否携带部门信息"),
                ),
        )
        .subcommand(
            with_cache_opts(with_common_opts(Command::new("list").about("按条件查询用户列表（优先缓存）")))
                .arg(Arg::new("keyword").long("keyword").num_args(1).help("姓名/邮箱关键字"))
                .arg(
                    Arg::new("page-size")
                        .long("page-size")
                        .default_value("50")
                        .value_parser(clap::value_parser!(u32))
                        .help("每页数量"),
                )
                .arg(Arg::new("page-token").long("page-token").num_args(1).help("翻页游标")),
        )
        .subcommand(
            with_cache_opts(with_common_opts(Command::new("find").about("按姓名关键字搜索用户（优先缓存）")))
                .arg(Arg::new("name").long("name").required(true).num_args(1).help("姓名关键字"))
                .arg(
                    Arg::new("page-size")
                        .long("page-size")
                        .default_value("50")
                        .value_parser(clap::value_parser!(u32))
                        .help("每页数量"),
                )
                .arg(Arg::new("page-token").long("page-token").num_args(1).help("翻页游标")),
        )
        .subcommand(
            with_common_opts(Command::new("sync").about("同步并刷新用户/部门缓存"))
                .arg(
                    Arg::new("max-depts")
                        .long("max-depts")
                        .default_value("300")
                        .value_parser(clap::value_parser!(u32))
                        .help("最多拉取部门数量"),
                )
                .arg(
                    Arg::new("cache-ttl-seconds")
                        .long("cache-ttl-seconds")
                        .default_value("21600")
                        .value_parser(clap::value_parser!(u64))
                        .help("缓存有效期（秒）"),
                ),
        )
        .subcommand(with_common_opts(Command::new("cache-status").about("查看本地 users 缓存状态")))
        .subcommand(with_common_opts(Command::new("cache-clear").about("清空本地 users 缓存")))
}

pub async fn handle(args: &[String]) -> Result<serde_json::Value, WpsError> {
    let mut argv = vec!["users".to_string()];
    argv.extend_from_slice(args);
    let m = command()
        .try_get_matches_from(argv)
        .map_err(|e| WpsError::Validation(e.to_string()))?;
    match m.subcommand() {
        Some(("scope", s)) => get_scope(s).await,
        Some(("depts", s)) => get_depts(s).await,
        Some(("members", s)) => get_members(s).await,
        Some(("user", s)) => get_user(s).await,
        Some(("list", s)) => list_users(s).await,
        Some(("find", s)) => find_users(s).await,
        Some(("sync", s)) => sync_cache(s).await,
        Some(("cache-status", s)) => cache_status(s).await,
        Some(("cache-clear", s)) => cache_clear(s).await,
        _ => Err(WpsError::Validation("unknown users helper subcommand".to_string())),
    }
}

fn with_common_opts(cmd: Command) -> Command {
    cmd.arg(
        Arg::new("auth-type")
            .long("auth-type")
            .value_parser(["app", "user"])
            .default_value("app")
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

fn with_cache_opts(cmd: Command) -> Command {
    cmd.arg(Arg::new("no-cache").long("no-cache").action(ArgAction::SetTrue).help("不使用本地缓存，直接请求远端"))
        .arg(Arg::new("refresh-cache").long("refresh-cache").action(ArgAction::SetTrue).help("先强制刷新缓存再查询"))
        .arg(
            Arg::new("cache-ttl-seconds")
                .long("cache-ttl-seconds")
                .default_value("21600")
                .value_parser(clap::value_parser!(u64))
                .help("缓存有效期（秒）"),
        )
}

fn effective_auth_type(s: &ArgMatches) -> String {
    if s.get_flag("user-token") {
        "user".to_string()
    } else {
        s.get_one::<String>("auth-type")
            .cloned()
            .unwrap_or_else(|| "app".to_string())
    }
}

fn ensure_users_auth_type(auth: &str) -> Result<(), WpsError> {
    if auth != "app" {
        return Err(WpsError::Auth(
            "users 助手命令仅支持 app token（组织通讯录接口）。请改用 `--auth-type app` 并移除 `--user-token`。".to_string(),
        ));
    }
    Ok(())
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

fn ok_data(data: Value) -> Value {
    serde_json::json!({
        "ok": true,
        "data": {
            "code": 0,
            "msg": "ok",
            "data": data
        }
    })
}

fn now_ts() -> u64 {
    chrono::Utc::now().timestamp().max(0) as u64
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

fn cache_store() -> Result<SkillStateStore, WpsError> {
    SkillStateStore::new(SKILL_NAME)
}

fn cache_path(store: &SkillStateStore) -> PathBuf {
    store.registry_path(ORG_CACHE_FILE)
}

fn load_cache(store: &SkillStateStore) -> Result<Option<OrgCache>, WpsError> {
    let path = cache_path(store);
    if !path.exists() {
        return Ok(None);
    }
    let raw = std::fs::read_to_string(&path)
        .map_err(|e| WpsError::Execution(format!("读取 users 缓存失败 {}: {e}", path.display())))?;
    let v: OrgCache = serde_json::from_str(&raw)
        .map_err(|e| WpsError::Execution(format!("解析 users 缓存失败 {}: {e}", path.display())))?;
    Ok(Some(v))
}

fn save_cache(store: &SkillStateStore, cache: &OrgCache) -> Result<(), WpsError> {
    let path = cache_path(store);
    let payload = serde_json::to_string_pretty(cache)
        .map_err(|e| WpsError::Execution(format!("序列化 users 缓存失败: {e}")))?;
    std::fs::write(&path, payload)
        .map_err(|e| WpsError::Execution(format!("写入 users 缓存失败 {}: {e}", path.display())))?;
    Ok(())
}

fn cache_is_fresh(cache: &OrgCache, ttl_seconds: u64) -> bool {
    if ttl_seconds == 0 {
        return false;
    }
    now_ts().saturating_sub(cache.generated_ts) <= ttl_seconds
}

fn parse_page_start(page_token: Option<&String>) -> usize {
    page_token
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(0)
}

fn paginate(items: &[Value], page_size: u32, page_token: Option<&String>) -> (Vec<Value>, Option<String>) {
    let size = page_size.max(1) as usize;
    let start = parse_page_start(page_token).min(items.len());
    let end = (start + size).min(items.len());
    let next = if end < items.len() {
        Some(end.to_string())
    } else {
        None
    };
    (items[start..end].to_vec(), next)
}

fn extract_next_page_token(payload: &Value) -> Option<String> {
    payload
        .get("page_token")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .or_else(|| {
            payload
                .get("next_page_token")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
        })
}

fn extract_roots(scope_payload: &Value) -> Vec<String> {
    let mut roots = Vec::new();
    let scope_items = scope_payload
        .get("items")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    for item in scope_items {
        let dept_ids = item
            .get("dept_ids")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();
        for v in dept_ids {
            if let Some(id) = v.as_str() {
                roots.push(id.to_string());
            }
        }
    }
    if roots.is_empty() {
        roots.push("root".to_string());
    }
    roots
}

fn user_id_from_member(item: &Value) -> Option<String> {
    item.get("user")
        .and_then(|u| u.get("id"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .or_else(|| item.get("id").and_then(|v| v.as_str()).map(|s| s.to_string()))
        .or_else(|| item.get("user_id").and_then(|v| v.as_str()).map(|s| s.to_string()))
}

fn user_obj_from_member(item: &Value) -> Option<Value> {
    if let Some(v) = item.get("user") {
        return Some(v.clone());
    }
    if item.is_object() {
        return Some(item.clone());
    }
    None
}

fn extract_user_dept_ids(user: &Value) -> Vec<String> {
    let mut out = Vec::<String>::new();
    if let Some(s) = user.get("dept_id").and_then(|v| v.as_str()) {
        if !s.trim().is_empty() {
            out.push(s.to_string());
        }
    }
    if let Some(arr) = user.get("dept_ids").and_then(|v| v.as_array()) {
        for v in arr {
            if let Some(s) = v.as_str() {
                if !s.trim().is_empty() {
                    out.push(s.to_string());
                }
            }
        }
    }
    if let Some(arr) = user.get("depts").and_then(|v| v.as_array()) {
        for v in arr {
            if let Some(s) = v.as_str() {
                if !s.trim().is_empty() {
                    out.push(s.to_string());
                }
                continue;
            }
            if let Some(id) = v.get("id").and_then(|x| x.as_str()) {
                if !id.trim().is_empty() {
                    out.push(id.to_string());
                }
            }
        }
    }
    out.sort();
    out.dedup();
    out
}

fn merge_users_into_member_index(
    users_by_id: &HashMap<String, Value>,
    members_by_dept: &mut HashMap<String, Vec<Value>>,
) {
    for (uid, user) in users_by_id {
        let dept_ids = extract_user_dept_ids(user);
        if dept_ids.is_empty() {
            continue;
        }
        let status = user
            .get("status")
            .and_then(|v| v.as_str())
            .unwrap_or("active")
            .to_string();
        for did in dept_ids {
            let entry = members_by_dept.entry(did).or_default();
            let exists = entry
                .iter()
                .any(|m| user_id_from_member(m).as_deref() == Some(uid.as_str()));
            if exists {
                continue;
            }
            entry.push(serde_json::json!({
                "user_id": uid,
                "status": status,
                "user": user
            }));
        }
    }
}

fn enrich_member_with_user(
    cache: &OrgCache,
    did: &str,
    member: &Value,
    with_user_detail: bool,
) -> Value {
    let st = member_status(member);
    let uid = user_id_from_member(member).unwrap_or_default();
    if !with_user_detail {
        if uid.is_empty() {
            return member.clone();
        }
        return serde_json::json!({
            "user_id": uid,
            "status": st,
            "dept_id": did
        });
    }
    if member.get("user").is_some() {
        return member.clone();
    }
    if !uid.is_empty() {
        if let Some(user) = cache.users_by_id.get(&uid) {
            return serde_json::json!({
                "user_id": uid,
                "status": st,
                "dept_id": did,
                "user": user
            });
        }
    }
    member.clone()
}

fn value_as_str(v: &Value, key: &str) -> Option<String> {
    v.get(key).and_then(|x| x.as_str()).map(|s| s.to_string())
}

fn normalize_status_set(raw: &str) -> HashSet<String> {
    raw.split(',')
        .map(|s| s.trim().to_ascii_lowercase())
        .filter(|s| !s.is_empty())
        .collect()
}

fn member_status(item: &Value) -> String {
    value_as_str(item, "status")
        .or_else(|| item.get("user").and_then(|u| value_as_str(u, "status")))
        .unwrap_or_else(|| "active".to_string())
        .to_ascii_lowercase()
}

fn user_match_keyword(user: &Value, keyword: &str) -> bool {
    if keyword.trim().is_empty() {
        return true;
    }
    let kw_lower = keyword.to_ascii_lowercase();
    let mut fields = vec![
        "name",
        "nickname",
        "email",
        "mobile",
        "phone",
        "employee_id",
        "ex_user_id",
    ];
    // Some endpoints return "mail"/"emails" variants.
    fields.extend(["mail", "display_name"]);
    for f in fields {
        if let Some(s) = user.get(f).and_then(|v| v.as_str()) {
            if s.contains(keyword) || s.to_ascii_lowercase().contains(&kw_lower) {
                return true;
            }
        }
    }
    if let Some(arr) = user.get("emails").and_then(|v| v.as_array()) {
        for e in arr {
            if let Some(s) = e.as_str() {
                if s.contains(keyword) || s.to_ascii_lowercase().contains(&kw_lower) {
                    return true;
                }
            }
        }
    }
    false
}

fn dept_children(cache: &OrgCache, parent_id: &str) -> Vec<Value> {
    if parent_id == "root" {
        let mut roots = Vec::new();
        for rid in &cache.roots {
            if let Some(d) = cache
                .depts
                .iter()
                .find(|d| d.get("id").and_then(|v| v.as_str()) == Some(rid.as_str()))
            {
                roots.push(d.clone());
            } else {
                roots.push(serde_json::json!({ "id": rid, "name": rid, "parent_id": "root" }));
            }
        }
        return roots;
    }
    cache
        .depts
        .iter()
        .filter(|d| {
            d.get("parent_id")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                == parent_id
        })
        .cloned()
        .collect()
}

fn dept_id(v: &Value) -> Option<String> {
    v.get("id").and_then(|x| x.as_str()).map(|s| s.to_string())
}

fn collect_recursive_depts(cache: &OrgCache, start: &str) -> HashSet<String> {
    let mut out = HashSet::new();
    let mut q = VecDeque::new();
    q.push_back(start.to_string());
    if start == "root" {
        for rid in &cache.roots {
            q.push_back(rid.clone());
        }
    }
    while let Some(cur) = q.pop_front() {
        if !out.insert(cur.clone()) {
            continue;
        }
        for child in dept_children(cache, &cur) {
            if let Some(id) = dept_id(&child) {
                q.push_back(id);
            }
        }
    }
    out
}

fn depts_for_user(cache: &OrgCache, uid: &str) -> Vec<Value> {
    let mut out = Vec::new();
    for (dept_id, members) in &cache.members_by_dept {
        if members.iter().any(|m| user_id_from_member(m).as_deref() == Some(uid)) {
            if let Some(d) = cache
                .depts
                .iter()
                .find(|d| d.get("id").and_then(|v| v.as_str()) == Some(dept_id))
            {
                out.push(d.clone());
            } else {
                out.push(serde_json::json!({ "id": dept_id }));
            }
        }
    }
    out
}

fn cache_meta(cache: &OrgCache, path: &PathBuf) -> Value {
    serde_json::json!({
        "source": "cache",
        "cache_path": path.display().to_string(),
        "cache_generated_at": cache.generated_at,
        "cache_generated_ts": cache.generated_ts,
        "cache_ttl_seconds": cache.ttl_seconds,
        "scope_mode": cache.scope_mode,
        "scope_error": cache.scope_error,
        "scope_guidance": cache.scope_guidance,
        "cache_age_seconds": now_ts().saturating_sub(cache.generated_ts),
        "dept_count": cache.depts.len(),
        "user_count": cache.users_by_id.len(),
        "user_scan_warnings": cache.user_scan_warnings
    })
}

fn scope_guidance_for_permissions_scope_error(auth: &str, scope_error: &str) -> Value {
    let required_delegated = USERS_REQUIRED_DELEGATED_SCOPES
        .iter()
        .map(|s| s.to_string())
        .collect::<Vec<_>>();
    let required_app_role = USERS_REQUIRED_APP_ROLE_SCOPES
        .iter()
        .map(|s| s.to_string())
        .collect::<Vec<_>>();

    let delegated = scope_catalog::analyze_required(&required_delegated, ScopeType::Delegated);
    let app_role = scope_catalog::analyze_required(&required_app_role, ScopeType::AppRole);
    let recommended_delegated = delegated.available.clone();
    let missing_delegated = delegated.missing.clone();
    let recommended_app_role = app_role.available.clone();
    let missing_app_role = app_role.missing.clone();

    let reauth_scopes = if recommended_delegated.is_empty() {
        required_delegated.clone()
    } else {
        recommended_delegated.clone()
    };
    let reauth_scope_arg = reauth_scopes.join(",");

    let mode_tip = if auth == "user" {
        "当前为 user 授权，优先执行 reauth。"
    } else {
        "当前为 app 授权，建议先在应用权限中心补齐 app_role，再按需切 user token 做 reauth。"
    };

    serde_json::json!({
        "reason": "permissions_scope_error",
        "error": scope_error,
        "mode_tip": mode_tip,
        "catalog_source": delegated.source,
        "required": {
            "delegated": required_delegated,
            "app_role": required_app_role
        },
        "available": {
            "delegated": recommended_delegated,
            "app_role": recommended_app_role
        },
        "missing": {
            "delegated": missing_delegated,
            "app_role": missing_app_role
        },
        "actions": {
            "reauth_local": format!("wpscli auth login --user --mode local --scope {reauth_scope_arg}"),
            "reauth_remote": format!("wpscli auth login --user --mode remote --scope {reauth_scope_arg}"),
            "verify_auth": "wpscli auth status",
            "refresh_cache": "wpscli users sync --auth-type app --refresh-cache",
            "app_scope_apply_hint": "请在应用权限管理中确保 app_role 至少包含 kso.contact.read 或 kso.contact.readwrite"
        }
    })
}

async fn build_org_cache(
    auth: &str,
    retry: u32,
    max_depts: u32,
    ttl_seconds: u64,
) -> Result<OrgCache, WpsError> {
    let mut q = HashMap::new();
    q.insert("scopes".to_string(), "org".to_string());
    let (roots, scope_mode, scope_error, scope_guidance) = match execute(
        "GET",
        "/v7/contacts/permissions_scope",
        q,
        None,
        auth,
        false,
        retry,
    )
    .await
    {
        Ok(scope_resp) if api_ok(&scope_resp) => (
            extract_roots(&api_payload(&scope_resp)),
            "permissions_scope".to_string(),
            None,
            None,
        ),
        Ok(scope_resp) => (
            vec!["root".to_string()],
            "fallback_root".to_string(),
            {
                let err = format!("permissions_scope_not_ok: {scope_resp}");
                Some(err)
            },
            {
                let err = format!("permissions_scope_not_ok: {scope_resp}");
                Some(scope_guidance_for_permissions_scope_error(auth, &err))
            },
        ),
        Err(e) => (
            vec!["root".to_string()],
            "fallback_root".to_string(),
            {
                let err = e.to_string();
                Some(err)
            },
            {
                let err = e.to_string();
                Some(scope_guidance_for_permissions_scope_error(auth, &err))
            },
        ),
    };

    let mut visited: HashSet<String> = HashSet::new();
    let mut queue: VecDeque<String> = roots.clone().into_iter().collect();
    let mut dept_map: HashMap<String, Value> = HashMap::new();
    let mut members_by_dept: HashMap<String, Vec<Value>> = HashMap::new();
    let mut users_by_id: HashMap<String, Value> = HashMap::new();

    dept_map.insert(
        "root".to_string(),
        serde_json::json!({
            "id": "root",
            "name": "root",
            "parent_id": ""
        }),
    );

    while let Some(dept_id) = queue.pop_front() {
        if visited.contains(&dept_id) || visited.len() as u32 >= max_depts {
            continue;
        }
        visited.insert(dept_id.clone());

        // children with pagination
        let mut child_page_token: Option<String> = None;
        let mut child_pages = 0u32;
        let mut child_empty_streak = 0u32;
        loop {
            child_pages += 1;
            if child_pages > SYNC_MAX_CHILD_PAGES_PER_DEPT {
                break;
            }
            let mut q_child = HashMap::new();
            q_child.insert("page_size".to_string(), "50".to_string());
            if let Some(tk) = &child_page_token {
                q_child.insert("page_token".to_string(), tk.clone());
            }
            let child_resp = execute(
                "GET",
                &format!("/v7/depts/{dept_id}/children"),
                q_child,
                None,
                auth,
                false,
                retry,
            )
            .await;
            let Ok(child_resp) = child_resp else {
                break;
            };
            if !api_ok(&child_resp) {
                break;
            }
            let payload = api_payload(&child_resp);
            let items = payload
                .get("items")
                .and_then(|v| v.as_array())
                .cloned()
                .unwrap_or_default();
            if items.is_empty() {
                child_empty_streak += 1;
            } else {
                child_empty_streak = 0;
            }
            for mut it in items {
                let cid = it.get("id").and_then(|v| v.as_str()).unwrap_or_default().to_string();
                if cid.is_empty() {
                    continue;
                }
                if it.get("parent_id").and_then(|v| v.as_str()).unwrap_or_default().is_empty() {
                    it["parent_id"] = Value::String(dept_id.clone());
                }
                dept_map.insert(cid.clone(), it);
                if !visited.contains(&cid) {
                    queue.push_back(cid);
                }
            }
            let next_child_token = extract_next_page_token(&payload);
            if next_child_token.is_some() && next_child_token == child_page_token {
                break;
            }
            if child_empty_streak >= 2 && next_child_token.is_some() {
                break;
            }
            child_page_token = next_child_token;
            if child_page_token.is_none() {
                break;
            }
        }

        // members with pagination
        let mut dept_members: Vec<Value> = Vec::new();
        let mut member_page_token: Option<String> = None;
        let mut member_pages = 0u32;
        let mut member_empty_streak = 0u32;
        loop {
            member_pages += 1;
            if member_pages > SYNC_MAX_MEMBER_PAGES_PER_DEPT {
                break;
            }
            let mut q_member = HashMap::new();
            q_member.insert("status".to_string(), DEFAULT_MEMBER_STATUS.to_string());
            q_member.insert("page_size".to_string(), "50".to_string());
            q_member.insert("recursive".to_string(), "false".to_string());
            q_member.insert("with_user_detail".to_string(), "true".to_string());
            if let Some(tk) = &member_page_token {
                q_member.insert("page_token".to_string(), tk.clone());
            }
            let members_resp = execute(
                "GET",
                &format!("/v7/depts/{dept_id}/members"),
                q_member,
                None,
                auth,
                false,
                retry,
            )
            .await;
            let Ok(members_resp) = members_resp else {
                break;
            };
            if !api_ok(&members_resp) {
                break;
            }
            let payload = api_payload(&members_resp);
            let items = payload
                .get("items")
                .and_then(|v| v.as_array())
                .cloned()
                .unwrap_or_default();
            if items.is_empty() {
                member_empty_streak += 1;
            } else {
                member_empty_streak = 0;
            }
            for it in items.clone() {
                if let Some(uid) = user_id_from_member(&it) {
                    if let Some(uobj) = user_obj_from_member(&it) {
                        users_by_id.entry(uid).or_insert(uobj);
                    }
                }
            }
            dept_members.extend(items);
            let next_member_token = extract_next_page_token(&payload);
            if next_member_token.is_some() && next_member_token == member_page_token {
                break;
            }
            if member_empty_streak >= 2 && next_member_token.is_some() {
                break;
            }
            member_page_token = next_member_token;
            if member_page_token.is_none() {
                break;
            }
        }
        if !dept_members.is_empty() {
            members_by_dept.insert(dept_id.clone(), dept_members);
        }
    }

    // 强制补齐用户明细：即使部门成员接口未返回 user 详情，也通过全量用户扫描填充 users_by_id。
    let scan = fetch_all_users_remote(auth, false, retry).await?;
    for u in scan.users {
        let uid = value_as_str(&u, "id").unwrap_or_else(|| serde_json::to_string(&u).unwrap_or_default());
        if !uid.is_empty() {
            users_by_id.insert(uid, u);
        }
    }
    merge_users_into_member_index(&users_by_id, &mut members_by_dept);

    let mut depts = dept_map.into_values().collect::<Vec<_>>();
    depts.sort_by(|a, b| {
        let na = a.get("name").and_then(|v| v.as_str()).unwrap_or_default();
        let nb = b.get("name").and_then(|v| v.as_str()).unwrap_or_default();
        na.cmp(nb)
    });

    Ok(OrgCache {
        generated_at: chrono::Utc::now().to_rfc3339(),
        generated_ts: now_ts(),
        ttl_seconds,
        scope_mode,
        scope_error,
        scope_guidance,
        roots,
        depts,
        members_by_dept,
        users_by_id,
        user_scan_warnings: scan.warnings,
    })
}

async fn ensure_cache(
    s: &ArgMatches,
    auth: &str,
    retry: u32,
    default_max_depts: u32,
) -> Result<Option<OrgCache>, WpsError> {
    if s.get_flag("dry-run") || s.get_flag("no-cache") {
        return Ok(None);
    }
    let ttl_seconds = s
        .get_one::<u64>("cache-ttl-seconds")
        .copied()
        .unwrap_or(DEFAULT_CACHE_TTL_SECONDS);
    let force_refresh = s.get_flag("refresh-cache");
    let has_max_depts = s.ids().any(|id| id.as_str() == "max-depts");
    let max_depts = if has_max_depts {
        s.get_one::<u32>("max-depts")
            .copied()
            .unwrap_or(default_max_depts)
    } else {
        default_max_depts
    };
    let store = cache_store()?;
    if !force_refresh {
        if let Some(cache) = load_cache(&store)? {
            if cache_is_fresh(&cache, ttl_seconds) {
                return Ok(Some(cache));
            }
        }
    }
    let Ok(cache) = build_org_cache(auth, retry, max_depts, ttl_seconds).await else {
        // 缓存刷新失败时降级为远端直连，避免查询命令整体失败。
        return Ok(None);
    };
    save_cache(&store, &cache)?;
    Ok(Some(cache))
}

async fn get_scope(s: &ArgMatches) -> Result<Value, WpsError> {
    let auth = effective_auth_type(s);
    ensure_users_auth_type(&auth)?;
    let dry = s.get_flag("dry-run");
    let retry = *s.get_one::<u32>("retry").unwrap_or(&1);
    let mut q = HashMap::new();
    q.insert("scopes".to_string(), "org".to_string());
    execute("GET", "/v7/contacts/permissions_scope", q, None, &auth, dry, retry).await
}

async fn get_depts(s: &ArgMatches) -> Result<Value, WpsError> {
    let auth = effective_auth_type(s);
    ensure_users_auth_type(&auth)?;
    let dry = s.get_flag("dry-run");
    let retry = *s.get_one::<u32>("retry").unwrap_or(&1);
    let dept_id = s.get_one::<String>("dept-id").expect("required");
    let page_size = s.get_one::<u32>("page-size").copied().unwrap_or(50);
    let page_token = s.get_one::<String>("page-token");

    if let Some(cache) = ensure_cache(s, &auth, retry, DEFAULT_QUERY_MAX_DEPTS).await? {
        let mut items = dept_children(&cache, dept_id);
        items.sort_by(|a, b| {
            let na = a.get("name").and_then(|v| v.as_str()).unwrap_or_default();
            let nb = b.get("name").and_then(|v| v.as_str()).unwrap_or_default();
            na.cmp(nb)
        });
        let (page_items, next_token) = paginate(&items, page_size, page_token);
        let store = cache_store()?;
        return Ok(ok_data(serde_json::json!({
            "items": page_items,
            "page_token": next_token,
            "has_more": next_token.is_some(),
            "meta": cache_meta(&cache, &cache_path(&store))
        })));
    }

    let mut q = HashMap::new();
    q.insert("page_size".to_string(), page_size.to_string());
    if let Some(v) = page_token {
        q.insert("page_token".to_string(), v.clone());
    }
    execute(
        "GET",
        &format!("/v7/depts/{dept_id}/children"),
        q,
        None,
        &auth,
        dry,
        retry,
    )
    .await
}

async fn get_members(s: &ArgMatches) -> Result<Value, WpsError> {
    let auth = effective_auth_type(s);
    ensure_users_auth_type(&auth)?;
    let dry = s.get_flag("dry-run");
    let retry = *s.get_one::<u32>("retry").unwrap_or(&1);
    let dept_id = s.get_one::<String>("dept-id").expect("required");
    let status_raw = s
        .get_one::<String>("status")
        .cloned()
        .unwrap_or_else(|| DEFAULT_MEMBER_STATUS.to_string());
    let status_set = normalize_status_set(&status_raw);
    let recursive = s
        .get_one::<String>("recursive")
        .map(|v| v == "true")
        .unwrap_or(false);
    let with_user_detail = s
        .get_one::<String>("with-user-detail")
        .map(|v| v == "true")
        .unwrap_or(true);
    let page_size = s.get_one::<u32>("page-size").copied().unwrap_or(50);
    let page_token = s.get_one::<String>("page-token");

    if let Some(cache) = ensure_cache(s, &auth, retry, DEFAULT_QUERY_MAX_DEPTS).await? {
        let target_depts = if recursive {
            collect_recursive_depts(&cache, dept_id)
        } else {
            let mut hs = HashSet::new();
            hs.insert(dept_id.to_string());
            hs
        };
        let mut all = Vec::<Value>::new();
        let mut seen_uid = HashSet::<String>::new();
        for did in target_depts {
            let members = cache.members_by_dept.get(&did).cloned().unwrap_or_default();
            for m in members {
                let st = member_status(&m);
                if !status_set.is_empty() && !status_set.contains(&st) {
                    continue;
                }
                if recursive {
                    if let Some(uid) = user_id_from_member(&m) {
                        if !seen_uid.insert(uid) {
                            continue;
                        }
                    }
                }
                let item = enrich_member_with_user(&cache, &did, &m, with_user_detail);
                all.push(item);
            }
        }
        let (page_items, next_token) = paginate(&all, page_size, page_token);
        let store = cache_store()?;
        return Ok(ok_data(serde_json::json!({
            "items": page_items,
            "page_token": next_token,
            "has_more": next_token.is_some(),
            "meta": cache_meta(&cache, &cache_path(&store))
        })));
    }

    let mut q = HashMap::new();
    q.insert("status".to_string(), status_raw);
    q.insert("page_size".to_string(), page_size.to_string());
    q.insert("recursive".to_string(), if recursive { "true" } else { "false" }.to_string());
    q.insert(
        "with_user_detail".to_string(),
        if with_user_detail { "true" } else { "false" }.to_string(),
    );
    if let Some(v) = page_token {
        q.insert("page_token".to_string(), v.clone());
    }
    execute(
        "GET",
        &format!("/v7/depts/{dept_id}/members"),
        q,
        None,
        &auth,
        dry,
        retry,
    )
    .await
}

async fn get_user(s: &ArgMatches) -> Result<Value, WpsError> {
    let auth = effective_auth_type(s);
    ensure_users_auth_type(&auth)?;
    let dry = s.get_flag("dry-run");
    let retry = *s.get_one::<u32>("retry").unwrap_or(&1);
    let user_id = s.get_one::<String>("user-id").expect("required");
    let with_dept = s
        .get_one::<String>("with-dept")
        .map(|v| v == "true")
        .unwrap_or(false);

    if let Some(cache) = ensure_cache(s, &auth, retry, DEFAULT_QUERY_MAX_DEPTS).await? {
        if let Some(user) = cache.users_by_id.get(user_id) {
            let mut data = serde_json::json!({
                "user": user,
                "meta": cache_meta(&cache, &cache_path(&cache_store()?))
            });
            if with_dept {
                data["depts"] = Value::Array(depts_for_user(&cache, user_id));
            }
            return Ok(ok_data(data));
        }
        return Ok(serde_json::json!({
            "ok": false,
            "data": {
                "code": 404,
                "msg": format!("用户不存在（缓存中未找到）: {user_id}")
            }
        }));
    }

    let mut q = HashMap::new();
    q.insert("with_dept".to_string(), if with_dept { "true" } else { "false" }.to_string());
    execute("GET", &format!("/v7/users/{user_id}"), q, None, &auth, dry, retry).await
}

fn sorted_cache_users(cache: &OrgCache) -> Vec<Value> {
    let mut users = cache.users_by_id.values().cloned().collect::<Vec<_>>();
    users.sort_by(|a, b| {
        let na = a.get("name").and_then(|v| v.as_str()).unwrap_or_default();
        let nb = b.get("name").and_then(|v| v.as_str()).unwrap_or_default();
        if na == nb {
            let ia = a.get("id").and_then(|v| v.as_str()).unwrap_or_default();
            let ib = b.get("id").and_then(|v| v.as_str()).unwrap_or_default();
            ia.cmp(ib)
        } else {
            na.cmp(nb)
        }
    });
    users
}

async fn fetch_all_users_remote(
    auth: &str,
    dry: bool,
    retry: u32,
) -> Result<RemoteUserScanResult, WpsError> {
    let mut result = RemoteUserScanResult::default();
    let mut seen = HashSet::<String>::new();
    let mut successful_pages = 0u32;
    for status in USER_LIST_STATUSES {
        let mut token: Option<String> = None;
        let mut empty_page_streak = 0u32;
        let mut rounds = 0u32;
        loop {
            rounds += 1;
            if rounds > REMOTE_SCAN_MAX_PAGES_PER_STATUS {
                result.warnings.push(serde_json::json!({
                    "status": status,
                    "kind": "page_guard_break",
                    "message": format!("扫描页数超过上限({REMOTE_SCAN_MAX_PAGES_PER_STATUS})，提前结束该状态扫描")
                }));
                break;
            }
            let mut q = HashMap::new();
            q.insert("status".to_string(), status.to_string());
            q.insert("with_dept".to_string(), "true".to_string());
            q.insert("page_size".to_string(), "50".to_string());
            if let Some(tk) = &token {
                q.insert("page_token".to_string(), tk.clone());
            }
            let resp = match execute("GET", "/v7/users", q, None, auth, dry, retry).await {
                Ok(v) => v,
                Err(e) => {
                    result.warnings.push(serde_json::json!({
                        "status": status,
                        "page_token": token,
                        "kind": "network_error",
                        "message": e.to_string()
                    }));
                    break;
                }
            };
            if !api_ok(&resp) {
                result.warnings.push(serde_json::json!({
                    "status": status,
                    "page_token": token,
                    "kind": "api_not_ok",
                    "response": resp
                }));
                break;
            }
            successful_pages += 1;
            let payload = api_payload(&resp);
            let items = payload
                .get("items")
                .and_then(|v| v.as_array())
                .cloned()
                .unwrap_or_default();
            if items.is_empty() {
                empty_page_streak += 1;
            } else {
                empty_page_streak = 0;
            }
            for u in items {
                let uid = value_as_str(&u, "id").unwrap_or_else(|| serde_json::to_string(&u).unwrap_or_default());
                if seen.insert(uid) {
                    result.users.push(u);
                }
            }
            let next_token = extract_next_page_token(&payload);
            if next_token.is_some() && next_token == token {
                result.warnings.push(serde_json::json!({
                    "status": status,
                    "kind": "stagnant_page_token",
                    "page_token": token,
                    "message": "分页游标未推进，提前结束该状态扫描"
                }));
                break;
            }
            if empty_page_streak >= 2 && next_token.is_some() {
                result.warnings.push(serde_json::json!({
                    "status": status,
                    "kind": "consecutive_empty_pages",
                    "page_token": next_token,
                    "message": "连续空页，提前结束该状态扫描"
                }));
                break;
            }
            token = next_token;
            if token.is_none() {
                break;
            }
        }
    }
    if successful_pages == 0 && !result.warnings.is_empty() {
        result.warnings.push(serde_json::json!({
            "kind": "no_successful_page",
            "message": "未成功拉取到任何用户分页，结果可能为空"
        }));
    }
    Ok(result)
}

async fn list_users(s: &ArgMatches) -> Result<Value, WpsError> {
    let auth = effective_auth_type(s);
    ensure_users_auth_type(&auth)?;
    let dry = s.get_flag("dry-run");
    let retry = *s.get_one::<u32>("retry").unwrap_or(&1);
    let keyword = s.get_one::<String>("keyword").cloned().unwrap_or_default();
    let page_size = s.get_one::<u32>("page-size").copied().unwrap_or(50);
    let page_token = s.get_one::<String>("page-token");

    if let Some(cache) = ensure_cache(s, &auth, retry, DEFAULT_QUERY_MAX_DEPTS).await? {
        let users = sorted_cache_users(&cache);
        let filtered = users
            .into_iter()
            .filter(|u| user_match_keyword(u, &keyword))
            .collect::<Vec<_>>();
        let total = filtered.len();
        let (items, next_token) = paginate(&filtered, page_size, page_token);
        let store = cache_store()?;
        return Ok(ok_data(serde_json::json!({
            "items": items,
            "total": total,
            "page_token": next_token,
            "has_more": next_token.is_some(),
            "meta": cache_meta(&cache, &cache_path(&store))
        })));
    }

    let scan = fetch_all_users_remote(&auth, dry, retry).await?;
    let filtered = scan
        .users
        .into_iter()
        .filter(|u| user_match_keyword(u, &keyword))
        .collect::<Vec<_>>();
    let total = filtered.len();
    let (items, next_token) = paginate(&filtered, page_size, page_token);
    Ok(ok_data(serde_json::json!({
        "items": items,
        "total": total,
        "page_token": next_token,
        "has_more": next_token.is_some(),
        "meta": {
            "source": "remote_full_scan_filter",
            "warnings": scan.warnings
        }
    })))
}

async fn find_users(s: &ArgMatches) -> Result<Value, WpsError> {
    let auth = effective_auth_type(s);
    ensure_users_auth_type(&auth)?;
    let dry = s.get_flag("dry-run");
    let retry = *s.get_one::<u32>("retry").unwrap_or(&1);
    let name = s.get_one::<String>("name").expect("required").clone();
    let page_size = s.get_one::<u32>("page-size").copied().unwrap_or(50);
    let page_token = s.get_one::<String>("page-token");

    if let Some(cache) = ensure_cache(s, &auth, retry, DEFAULT_QUERY_MAX_DEPTS).await? {
        let users = sorted_cache_users(&cache);
        let matched = users
            .into_iter()
            .filter(|u| user_match_keyword(u, &name))
            .collect::<Vec<_>>();
        let total = matched.len();
        let (items, next_token) = paginate(&matched, page_size, page_token);
        let store = cache_store()?;
        return Ok(ok_data(serde_json::json!({
            "items": items,
            "total": total,
            "page_token": next_token,
            "has_more": next_token.is_some(),
            "meta": cache_meta(&cache, &cache_path(&store))
        })));
    }

    let scan = fetch_all_users_remote(&auth, dry, retry).await?;
    let matched = scan
        .users
        .into_iter()
        .filter(|u| user_match_keyword(u, &name))
        .collect::<Vec<_>>();
    let total = matched.len();
    let (items, next_token) = paginate(&matched, page_size, page_token);
    Ok(ok_data(serde_json::json!({
        "items": items,
        "total": total,
        "page_token": next_token,
        "has_more": next_token.is_some(),
        "meta": {
            "source": "remote_full_scan_filter",
            "warnings": scan.warnings
        }
    })))
}

async fn sync_cache(s: &ArgMatches) -> Result<Value, WpsError> {
    let auth = effective_auth_type(s);
    ensure_users_auth_type(&auth)?;
    let dry = s.get_flag("dry-run");
    let retry = *s.get_one::<u32>("retry").unwrap_or(&1);
    let max_depts = s.get_one::<u32>("max-depts").copied().unwrap_or(300);
    let ttl_seconds = s
        .get_one::<u64>("cache-ttl-seconds")
        .copied()
        .unwrap_or(DEFAULT_CACHE_TTL_SECONDS);

    if dry {
        return Ok(ok_data(serde_json::json!({
            "mode": "sync_cache",
            "dry_run": true,
            "max_depts": max_depts,
            "cache_ttl_seconds": ttl_seconds
        })));
    }

    let store = cache_store()?;
    let cache = build_org_cache(&auth, retry, max_depts, ttl_seconds).await?;
    save_cache(&store, &cache)?;

    Ok(ok_data(serde_json::json!({
        "mode": "sync_cache",
        "cache_path": cache_path(&store).display().to_string(),
        "generated_at": cache.generated_at,
        "generated_ts": cache.generated_ts,
        "cache_ttl_seconds": cache.ttl_seconds,
        "scope_mode": cache.scope_mode,
        "scope_error": cache.scope_error,
        "scope_guidance": cache.scope_guidance,
        "roots": cache.roots,
        "dept_count": cache.depts.len(),
        "user_count": cache.users_by_id.len(),
        "member_relation_count": cache.members_by_dept.values().map(|v| v.len()).sum::<usize>(),
        "user_scan_warnings": cache.user_scan_warnings,
        "query_strategy": "后续 users 查询优先走本地缓存，缓存过期会自动刷新"
    })))
}

async fn cache_status(s: &ArgMatches) -> Result<Value, WpsError> {
    let dry = s.get_flag("dry-run");
    if dry {
        return Ok(ok_data(serde_json::json!({
            "mode": "cache_status",
            "dry_run": true
        })));
    }
    let store = cache_store()?;
    let path = cache_path(&store);
    let cache = load_cache(&store)?;
    if let Some(cache) = cache {
        return Ok(ok_data(serde_json::json!({
            "exists": true,
            "cache_path": path.display().to_string(),
            "generated_at": cache.generated_at,
            "generated_ts": cache.generated_ts,
            "cache_age_seconds": now_ts().saturating_sub(cache.generated_ts),
            "cache_ttl_seconds": cache.ttl_seconds,
            "scope_mode": cache.scope_mode,
            "scope_error": cache.scope_error,
            "scope_guidance": cache.scope_guidance,
            "fresh_by_default_ttl": cache_is_fresh(&cache, cache.ttl_seconds),
            "dept_count": cache.depts.len(),
            "user_count": cache.users_by_id.len(),
            "member_relation_count": cache.members_by_dept.values().map(|v| v.len()).sum::<usize>(),
            "user_scan_warnings": cache.user_scan_warnings
        })));
    }
    Ok(ok_data(serde_json::json!({
        "exists": false,
        "cache_path": path.display().to_string(),
        "hint": "执行 `wpscli users sync --auth-type app` 先构建缓存"
    })))
}

async fn cache_clear(s: &ArgMatches) -> Result<Value, WpsError> {
    let dry = s.get_flag("dry-run");
    let store = cache_store()?;
    let path = cache_path(&store);
    if dry {
        return Ok(ok_data(serde_json::json!({
            "mode": "cache_clear",
            "dry_run": true,
            "cache_path": path.display().to_string()
        })));
    }
    if path.exists() {
        std::fs::remove_file(&path)
            .map_err(|e| WpsError::Execution(format!("删除 users 缓存失败 {}: {e}", path.display())))?;
    }
    Ok(ok_data(serde_json::json!({
        "deleted": true,
        "cache_path": path.display().to_string()
    })))
}
