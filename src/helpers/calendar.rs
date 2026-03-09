use std::collections::HashMap;

use clap::{Arg, Command};

use crate::error::WpsError;
use crate::executor;

pub fn command() -> Command {
    Command::new("calendar")
        .about("日历助手命令")
        .after_help(
            "示例：\n  \
             wpscli calendar query --calendar-id <id> --start-time 2026-03-01T00:00:00Z --end-time 2026-03-31T23:59:59Z\n  \
             wpscli calendar busy --start-time 2026-03-10T09:00:00Z --end-time 2026-03-10T18:00:00Z",
        )
        .subcommand(
            Command::new("query")
                .about("查询日历事件")
                .arg(Arg::new("calendar-id").long("calendar-id").required(true).num_args(1).help("日历 ID"))
                .arg(Arg::new("start-time").long("start-time").num_args(1).help("开始时间（ISO8601）"))
                .arg(Arg::new("end-time").long("end-time").num_args(1).help("结束时间（ISO8601）")),
        )
        .subcommand(
            Command::new("busy")
                .about("查询忙闲状态")
                .arg(Arg::new("start-time").long("start-time").required(true).num_args(1).help("开始时间（ISO8601）"))
                .arg(Arg::new("end-time").long("end-time").required(true).num_args(1).help("结束时间（ISO8601）")),
        )
}

pub async fn handle(args: &[String]) -> Result<serde_json::Value, WpsError> {
    let mut argv = vec!["calendar".to_string()];
    argv.extend_from_slice(args);
    let m = command()
        .try_get_matches_from(argv)
        .map_err(|e| WpsError::Validation(e.to_string()))?;
    match m.subcommand() {
        Some(("query", s)) => {
            let mut q = HashMap::new();
            if let Some(v) = s.get_one::<String>("start-time") {
                q.insert("start_time".to_string(), v.clone());
            }
            if let Some(v) = s.get_one::<String>("end-time") {
                q.insert("end_time".to_string(), v.clone());
            }
            let calendar_id = s.get_one::<String>("calendar-id").expect("required");
            executor::execute_raw(
                "GET",
                &format!("/v7/calendars/{calendar_id}/events"),
                q,
                HashMap::new(),
                None,
                "user",
                false,
                1,
            )
            .await
        }
        Some(("busy", s)) => {
            let mut q = HashMap::new();
            q.insert(
                "start_time".to_string(),
                s.get_one::<String>("start-time").expect("required").clone(),
            );
            q.insert(
                "end_time".to_string(),
                s.get_one::<String>("end-time").expect("required").clone(),
            );
            executor::execute_raw(
                "GET",
                "/v7/free_busy_list",
                q,
                HashMap::new(),
                None,
                "user",
                false,
                1,
            )
            .await
        }
        _ => Err(WpsError::Validation("unknown calendar subcommand".to_string())),
    }
}
