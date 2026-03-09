use std::collections::HashMap;

use clap::{Arg, Command};

use crate::error::WpsError;
use crate::executor;

pub fn command() -> Command {
    Command::new("chat")
        .about("会话与消息助手命令")
        .after_help(
            "示例：\n  \
             wpscli chat chats\n  \
             wpscli chat push <chat_id> --text \"你好，今天 3 点开会\"",
        )
        .subcommand(Command::new("chats").about("列出当前可见会话"))
        .subcommand(
            Command::new("push")
                .about("发送文本消息")
                .arg(Arg::new("chat-id").required(true).help("会话 ID"))
                .arg(Arg::new("text").long("text").required(true).num_args(1).help("消息内容")),
        )
}

pub async fn handle(args: &[String]) -> Result<serde_json::Value, WpsError> {
    let mut argv = vec!["chat".to_string()];
    argv.extend_from_slice(args);
    let m = command()
        .try_get_matches_from(argv)
        .map_err(|e| WpsError::Validation(e.to_string()))?;
    match m.subcommand() {
        Some(("chats", _)) => executor::execute_raw(
            "GET",
            "/v7/chats",
            HashMap::new(),
            HashMap::new(),
            None,
            "user",
            false,
            1,
        )
        .await,
        Some(("push", s)) => {
            let chat_id = s.get_one::<String>("chat-id").expect("required");
            let text = s.get_one::<String>("text").expect("required");
            let body = serde_json::json!({
                "chat_id": chat_id,
                "text": text
            })
            .to_string();
            executor::execute_raw(
                "POST",
                "/v7/messages/create",
                HashMap::new(),
                HashMap::new(),
                Some(body),
                "user",
                false,
                1,
            )
            .await
        }
        _ => Err(WpsError::Validation("unknown chat subcommand".to_string())),
    }
}
