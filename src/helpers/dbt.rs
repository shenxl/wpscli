use std::collections::HashMap;

use clap::{Arg, Command};

use crate::error::WpsError;
use crate::executor;

pub fn command() -> Command {
    Command::new("dbt")
        .about("DBSheet 兼容助手（旧版命令）")
        .after_help(
            "示例：\n  \
             wpscli dbt schema <file-id>\n  \
             wpscli dbt list <file-id> <sheet-id>\n  \
             wpscli dbt create <file-id> <sheet-id> --body '{\"records\":[...]}'",
        )
        .subcommand(
            Command::new("schema")
                .about("读取多维表 schema")
                .arg(Arg::new("file-id").required(true).help("多维表 file_id")),
        )
        .subcommand(
            Command::new("list")
                .about("列出记录")
                .arg(Arg::new("file-id").required(true).help("多维表 file_id"))
                .arg(Arg::new("sheet-id").required(true).help("工作表 sheet_id")),
        )
        .subcommand(
            Command::new("create")
                .about("创建记录")
                .arg(Arg::new("file-id").required(true).help("多维表 file_id"))
                .arg(Arg::new("sheet-id").required(true).help("工作表 sheet_id"))
                .arg(Arg::new("body").long("body").required(true).num_args(1).help("请求体 JSON 字符串")),
        )
        .subcommand(
            Command::new("update")
                .about("更新记录")
                .arg(Arg::new("file-id").required(true).help("多维表 file_id"))
                .arg(Arg::new("sheet-id").required(true).help("工作表 sheet_id"))
                .arg(Arg::new("body").long("body").required(true).num_args(1).help("请求体 JSON 字符串")),
        )
        .subcommand(
            Command::new("delete")
                .about("删除记录")
                .arg(Arg::new("file-id").required(true).help("多维表 file_id"))
                .arg(Arg::new("sheet-id").required(true).help("工作表 sheet_id"))
                .arg(Arg::new("body").long("body").required(true).num_args(1).help("请求体 JSON 字符串")),
        )
        .subcommand(
            Command::new("import-csv")
                .about("从 CSV 导入记录")
                .arg(Arg::new("file-id").required(true).help("多维表 file_id"))
                .arg(Arg::new("sheet-id").required(true).help("工作表 sheet_id"))
                .arg(Arg::new("csv").long("csv").required(true).num_args(1).help("CSV 文件路径")),
        )
}

pub async fn handle(args: &[String]) -> Result<serde_json::Value, WpsError> {
    let mut argv = vec!["dbt".to_string()];
    argv.extend_from_slice(args);
    let m = command()
        .try_get_matches_from(argv)
        .map_err(|e| WpsError::Validation(e.to_string()))?;
    match m.subcommand() {
        Some(("schema", s)) => {
            let file_id = s.get_one::<String>("file-id").expect("required");
            executor::execute_raw(
                "GET",
                &format!("/v7/coop/dbsheet/{file_id}/schema"),
                HashMap::new(),
                HashMap::new(),
                None,
                "app",
                false,
                1,
            )
            .await
        }
        Some(("list", s)) => {
            let file_id = s.get_one::<String>("file-id").expect("required");
            let sheet_id = s.get_one::<String>("sheet-id").expect("required");
            executor::execute_raw(
                "POST",
                &format!("/v7/coop/dbsheet/{file_id}/sheets/{sheet_id}/records"),
                HashMap::new(),
                HashMap::new(),
                Some("{}".to_string()),
                "app",
                false,
                1,
            )
            .await
        }
        Some(("create", s)) => {
            let file_id = s.get_one::<String>("file-id").expect("required");
            let sheet_id = s.get_one::<String>("sheet-id").expect("required");
            let body = s.get_one::<String>("body").expect("required").clone();
            executor::execute_raw(
                "POST",
                &format!("/v7/coop/dbsheet/{file_id}/sheets/{sheet_id}/records/create"),
                HashMap::new(),
                HashMap::new(),
                Some(body),
                "app",
                false,
                1,
            )
            .await
        }
        Some(("update", s)) => {
            let file_id = s.get_one::<String>("file-id").expect("required");
            let sheet_id = s.get_one::<String>("sheet-id").expect("required");
            let body = s.get_one::<String>("body").expect("required").clone();
            executor::execute_raw(
                "POST",
                &format!("/v7/coop/dbsheet/{file_id}/sheets/{sheet_id}/records/update"),
                HashMap::new(),
                HashMap::new(),
                Some(body),
                "app",
                false,
                1,
            )
            .await
        }
        Some(("delete", s)) => {
            let file_id = s.get_one::<String>("file-id").expect("required");
            let sheet_id = s.get_one::<String>("sheet-id").expect("required");
            let body = s.get_one::<String>("body").expect("required").clone();
            executor::execute_raw(
                "POST",
                &format!("/v7/coop/dbsheet/{file_id}/sheets/{sheet_id}/records/batch_delete"),
                HashMap::new(),
                HashMap::new(),
                Some(body),
                "app",
                false,
                1,
            )
            .await
        }
        Some(("import-csv", s)) => {
            let csv = std::fs::read_to_string(s.get_one::<String>("csv").expect("required"))
                .map_err(|e| WpsError::Validation(format!("failed to read csv: {e}")))?;
            let file_id = s.get_one::<String>("file-id").expect("required");
            let sheet_id = s.get_one::<String>("sheet-id").expect("required");
            // Import uses batch create contract; caller should provide mapped rows if needed.
            let body = serde_json::json!({"csv": csv, "sheet_id": sheet_id}).to_string();
            executor::execute_raw(
                "POST",
                &format!("/v7/coop/dbsheet/{file_id}/sheets/{sheet_id}/records/create"),
                HashMap::new(),
                HashMap::new(),
                Some(body),
                "app",
                false,
                1,
            )
            .await
        }
        _ => Err(WpsError::Validation("unknown dbt subcommand".to_string())),
    }
}
