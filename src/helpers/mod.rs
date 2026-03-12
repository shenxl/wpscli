pub mod airpage;
pub mod calendar;
pub mod chat;
pub mod dbsheet;
pub mod dbt;
pub mod doc;
pub mod files;
pub mod meeting;
pub mod sheets;
pub mod users;

pub fn command(first: &str) -> Option<clap::Command> {
    match first {
        "doc" => Some(doc::command()),
        "dbt" => Some(dbt::command()),
        "dbsheet" => Some(dbsheet::command()),
        "chat" => Some(chat::command()),
        "calendar" => Some(calendar::command()),
        "users" => Some(users::command()),
        "meeting" => Some(meeting::command()),
        "airpage" => Some(airpage::command()),
        "files" | "app-files" => Some(files::command()),
        _ => None,
    }
}

pub async fn dispatch(first: &str, args: &[String]) -> Option<Result<serde_json::Value, crate::error::WpsError>> {
    match first {
        "doc" => Some(doc::handle(args).await),
        "dbt" => Some(dbt::handle(args).await),
        "dbsheet" => {
            if args.is_empty()
                || matches!(
                    args[0].as_str(),
                    "schema"
                        | "list-sheets"
                        | "init"
                        | "select"
                        | "insert"
                        | "update"
                        | "delete"
                        | "view-list"
                        | "view-get"
                        | "view-create"
                        | "view-update"
                        | "view-delete"
                        | "webhook-list"
                        | "webhook-create"
                        | "webhook-delete"
                        | "share-status"
                        | "share-enable"
                        | "share-disable"
                        | "share-permission-update"
                        | "form-meta"
                        | "form-meta-update"
                        | "form-fields"
                        | "form-field-update"
                        | "dashboard-list"
                        | "dashboard-copy"
                        | "clean"
                        | "-h"
                        | "--help"
                        | "help"
                )
            {
                Some(dbsheet::handle(args).await)
            } else {
                None
            }
        }
        "chat" => Some(chat::handle(args).await),
        "calendar" => Some(calendar::handle(args).await),
        "users" => {
            // Do not shadow dynamic `users` service routes unless it is an explicit helper command.
            if args.is_empty()
                || matches!(
                    args[0].as_str(),
                    "scope"
                        | "depts"
                        | "members"
                        | "user"
                        | "list"
                        | "find"
                        | "sync"
                        | "cache-status"
                        | "cache-clear"
                        | "-h"
                        | "--help"
                        | "help"
                )
            {
                Some(users::handle(args).await)
            } else {
                None
            }
        }
        "meeting" => Some(meeting::handle(args).await),
        "airpage" => Some(airpage::handle(args).await),
        "files" | "app-files" => {
            // Keep dynamic `files` service usable by falling through unknown subcommands.
            if args.is_empty()
                || matches!(
                    args[0].as_str(),
                    "list-apps"
                        | "ensure-app"
                        | "create"
                        | "add-file"
                        | "create-file"
                        | "list-files"
                        | "upload"
                        | "download"
                        | "transfer"
                        | "state"
                        | "get"
                        | "-h"
                        | "--help"
                        | "help"
                )
            {
                Some(files::handle(args).await)
            } else {
                None
            }
        }
        _ => None,
    }
}
