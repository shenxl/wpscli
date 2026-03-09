use crate::error::WpsError;

pub fn resolve_service(name: &str) -> Result<String, WpsError> {
    let normalized = match name {
        "dbt" => "coop_dbsheet",
        "dbsheet" => "coop_dbsheet",
        "calendar" => "calendars",
        "chat" => "chats",
        "file" => "files",
        "user" => "users",
        other => other,
    };
    if normalized.is_empty() {
        return Err(WpsError::Validation("empty service".to_string()));
    }
    Ok(normalized.to_string())
}
