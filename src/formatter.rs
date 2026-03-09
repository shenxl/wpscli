use serde_json::Value;

#[derive(Debug, Clone, Copy)]
pub enum OutputFormat {
    Json,
    Compact,
    Table,
}

impl OutputFormat {
    pub fn parse(value: Option<&String>) -> Self {
        match value.map(|s| s.as_str()) {
            Some("compact") => Self::Compact,
            Some("table") => Self::Table,
            _ => Self::Json,
        }
    }
}

pub fn print_value(value: &Value, format: OutputFormat) {
    match format {
        OutputFormat::Json => {
            println!(
                "{}",
                serde_json::to_string_pretty(value).unwrap_or_else(|_| "{}".to_string())
            );
        }
        OutputFormat::Compact => {
            println!(
                "{}",
                serde_json::to_string(value).unwrap_or_else(|_| "{}".to_string())
            );
        }
        OutputFormat::Table => {
            // Lightweight fallback: table mode currently prints compact JSON.
            println!(
                "{}",
                serde_json::to_string(value).unwrap_or_else(|_| "{}".to_string())
            );
        }
    }
}
