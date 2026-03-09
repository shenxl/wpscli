use std::collections::HashMap;

use clap::{Arg, Command};

use crate::error::WpsError;
use crate::executor;

pub fn command() -> Command {
    Command::new("meeting")
        .about("会议纪要助手命令")
        .after_help("示例：\n  wpscli meeting analyze --minute-id <minute_id>")
        .subcommand(
            Command::new("analyze")
                .about("读取并分析会议纪要详情")
                .arg(Arg::new("minute-id").long("minute-id").required(true).num_args(1).help("会议纪要 ID")),
        )
}

pub async fn handle(args: &[String]) -> Result<serde_json::Value, WpsError> {
    let mut argv = vec!["meeting".to_string()];
    argv.extend_from_slice(args);
    let m = command()
        .try_get_matches_from(argv)
        .map_err(|e| WpsError::Validation(e.to_string()))?;
    match m.subcommand() {
        Some(("analyze", s)) => {
            let minute_id = s.get_one::<String>("minute-id").expect("required");
            executor::execute_raw(
                "GET",
                &format!("/v7/minutes/{minute_id}"),
                HashMap::new(),
                HashMap::new(),
                None,
                "user",
                false,
                1,
            )
            .await
        }
        _ => Err(WpsError::Validation("unknown meeting subcommand".to_string())),
    }
}
