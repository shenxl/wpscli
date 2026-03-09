use std::collections::{HashMap, HashSet};

use clap::{Arg, ArgAction, ArgMatches, Command};
use serde_json::Value;

use crate::error::WpsError;
use crate::executor;

pub fn command() -> Command {
    Command::new("users")
        .about("用户与组织架构助手（wps-users）")
        .after_help(
            "示例：\n  \
             wpscli users scope --user-token\n  \
             wpscli users depts --dept-id root --user-token\n  \
             wpscli users members --dept-id 123 --recursive true --user-token\n  \
             wpscli users find --name 张三 --user-token",
        )
        .subcommand(with_common_opts(Command::new("scope").about("查看通讯录权限范围（org）")))
        .subcommand(
            with_common_opts(Command::new("depts").about("列出指定部门的子部门"))
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
            with_common_opts(Command::new("members").about("列出部门成员"))
                .arg(Arg::new("dept-id").long("dept-id").required(true).num_args(1).help("部门 ID"))
                .arg(
                    Arg::new("status")
                        .long("status")
                        .default_value("active,notactive,disabled")
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
            with_common_opts(Command::new("user").about("查询指定用户详情"))
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
            with_common_opts(Command::new("list").about("按条件查询用户列表"))
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
            with_common_opts(Command::new("find").about("按姓名关键字搜索用户"))
                .arg(Arg::new("name").long("name").required(true).num_args(1).help("姓名关键字")),
        )
        .subcommand(
            with_common_opts(Command::new("sync").about("同步组织架构摘要（轻量版）"))
                .arg(
                    Arg::new("max-depts")
                        .long("max-depts")
                        .default_value("200")
                        .value_parser(clap::value_parser!(u32))
                        .help("最多拉取部门数量"),
                ),
        )
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
        Some(("sync", s)) => sync_summary(s).await,
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

fn effective_auth_type(s: &ArgMatches) -> String {
    if s.get_flag("user-token") {
        "user".to_string()
    } else {
        s.get_one::<String>("auth-type")
            .cloned()
            .unwrap_or_else(|| "app".to_string())
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

async fn get_scope(s: &ArgMatches) -> Result<Value, WpsError> {
    let auth = effective_auth_type(s);
    let dry = s.get_flag("dry-run");
    let retry = *s.get_one::<u32>("retry").unwrap_or(&1);
    let mut q = HashMap::new();
    q.insert("scopes".to_string(), "org".to_string());
    execute("GET", "/v7/contacts/permissions_scope", q, None, &auth, dry, retry).await
}

async fn get_depts(s: &ArgMatches) -> Result<Value, WpsError> {
    let auth = effective_auth_type(s);
    let dry = s.get_flag("dry-run");
    let retry = *s.get_one::<u32>("retry").unwrap_or(&1);
    let dept_id = s.get_one::<String>("dept-id").expect("required");
    let mut q = HashMap::new();
    q.insert(
        "page_size".to_string(),
        s.get_one::<u32>("page-size").copied().unwrap_or(50).to_string(),
    );
    if let Some(v) = s.get_one::<String>("page-token") {
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
    let dry = s.get_flag("dry-run");
    let retry = *s.get_one::<u32>("retry").unwrap_or(&1);
    let dept_id = s.get_one::<String>("dept-id").expect("required");
    let mut q = HashMap::new();
    q.insert("status".to_string(), s.get_one::<String>("status").cloned().unwrap_or_default());
    q.insert(
        "page_size".to_string(),
        s.get_one::<u32>("page-size").copied().unwrap_or(50).to_string(),
    );
    q.insert(
        "recursive".to_string(),
        s.get_one::<String>("recursive")
            .cloned()
            .unwrap_or_else(|| "false".to_string()),
    );
    q.insert(
        "with_user_detail".to_string(),
        s.get_one::<String>("with-user-detail")
            .cloned()
            .unwrap_or_else(|| "true".to_string()),
    );
    if let Some(v) = s.get_one::<String>("page-token") {
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
    let dry = s.get_flag("dry-run");
    let retry = *s.get_one::<u32>("retry").unwrap_or(&1);
    let user_id = s.get_one::<String>("user-id").expect("required");
    let mut q = HashMap::new();
    q.insert(
        "with_dept".to_string(),
        s.get_one::<String>("with-dept")
            .cloned()
            .unwrap_or_else(|| "false".to_string()),
    );
    execute("GET", &format!("/v7/users/{user_id}"), q, None, &auth, dry, retry).await
}

async fn list_users(s: &ArgMatches) -> Result<Value, WpsError> {
    let auth = effective_auth_type(s);
    let dry = s.get_flag("dry-run");
    let retry = *s.get_one::<u32>("retry").unwrap_or(&1);
    let mut q = HashMap::new();
    if let Some(v) = s.get_one::<String>("keyword") {
        q.insert("keyword".to_string(), v.clone());
    }
    q.insert(
        "page_size".to_string(),
        s.get_one::<u32>("page-size").copied().unwrap_or(50).to_string(),
    );
    if let Some(v) = s.get_one::<String>("page-token") {
        q.insert("page_token".to_string(), v.clone());
    }
    execute("GET", "/v7/users", q, None, &auth, dry, retry).await
}

async fn find_users(s: &ArgMatches) -> Result<Value, WpsError> {
    let auth = effective_auth_type(s);
    let dry = s.get_flag("dry-run");
    let retry = *s.get_one::<u32>("retry").unwrap_or(&1);
    let mut q = HashMap::new();
    q.insert(
        "keyword".to_string(),
        s.get_one::<String>("name").expect("required").clone(),
    );
    execute("GET", "/v7/users", q, None, &auth, dry, retry).await
}

async fn sync_summary(s: &ArgMatches) -> Result<Value, WpsError> {
    let auth = effective_auth_type(s);
    let dry = s.get_flag("dry-run");
    let retry = *s.get_one::<u32>("retry").unwrap_or(&1);
    let max_depts = s.get_one::<u32>("max-depts").copied().unwrap_or(200);

    let mut q = HashMap::new();
    q.insert("scopes".to_string(), "org".to_string());
    let scope_resp = execute("GET", "/v7/contacts/permissions_scope", q, None, &auth, dry, retry).await?;
    if dry || !api_ok(&scope_resp) {
        return Ok(scope_resp);
    }
    let scope_items = api_payload(&scope_resp)
        .get("items")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    let mut roots = Vec::new();
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

    let mut visited: HashSet<String> = HashSet::new();
    let mut queue = roots.clone();
    let mut dept_count: u32 = 0;
    let mut member_count: u32 = 0;
    let mut sample_users: Vec<Value> = Vec::new();

    while let Some(dept_id) = queue.pop() {
        if visited.contains(&dept_id) || visited.len() as u32 >= max_depts {
            continue;
        }
        visited.insert(dept_id.clone());
        dept_count += 1;

        let mut q_child = HashMap::new();
        q_child.insert("page_size".to_string(), "50".to_string());
        let child_resp = execute(
            "GET",
            &format!("/v7/depts/{dept_id}/children"),
            q_child,
            None,
            &auth,
            false,
            retry,
        )
        .await?;
        if api_ok(&child_resp) {
            let items = api_payload(&child_resp)
                .get("items")
                .and_then(|v| v.as_array())
                .cloned()
                .unwrap_or_default();
            for it in items {
                if let Some(id) = it.get("id").and_then(|v| v.as_str()) {
                    if !visited.contains(id) {
                        queue.push(id.to_string());
                    }
                }
            }
        }

        let mut q_member = HashMap::new();
        q_member.insert("status".to_string(), "active,notactive,disabled".to_string());
        q_member.insert("page_size".to_string(), "50".to_string());
        q_member.insert("recursive".to_string(), "false".to_string());
        q_member.insert("with_user_detail".to_string(), "true".to_string());
        let members_resp = execute(
            "GET",
            &format!("/v7/depts/{dept_id}/members"),
            q_member,
            None,
            &auth,
            false,
            retry,
        )
        .await?;
        if api_ok(&members_resp) {
            let items = api_payload(&members_resp)
                .get("items")
                .and_then(|v| v.as_array())
                .cloned()
                .unwrap_or_default();
            member_count += items.len() as u32;
            for u in items.into_iter().take(3) {
                if sample_users.len() < 10 {
                    sample_users.push(u);
                }
            }
        }
    }

    Ok(serde_json::json!({
        "ok": true,
        "data": {
            "code": 0,
            "msg": "ok",
            "data": {
                "mode": "sync_summary",
                "roots": roots,
                "dept_count": dept_count,
                "member_count": member_count,
                "sample_users": sample_users,
                "truncated": visited.len() as u32 >= max_depts,
                "max_depts": max_depts
            }
        }
    }))
}
