use crate::error::WpsError;

pub fn parse_key_value_pairs(values: &[String]) -> Result<Vec<(String, String)>, WpsError> {
    let mut out = Vec::new();
    for v in values {
        let (k, val) = v
            .split_once('=')
            .ok_or_else(|| WpsError::Validation(format!("invalid key=value pair: {v}")))?;
        if k.trim().is_empty() {
            return Err(WpsError::Validation("empty key in key=value pair".to_string()));
        }
        out.push((k.trim().to_string(), val.to_string()));
    }
    Ok(out)
}

pub fn encode_path_segment(v: &str) -> String {
    percent_encoding::utf8_percent_encode(v, percent_encoding::NON_ALPHANUMERIC).to_string()
}
