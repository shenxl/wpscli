use crate::descriptor;
use crate::error::WpsError;

pub fn run(service: &str, endpoint: Option<&str>) -> Result<serde_json::Value, WpsError> {
    let desc = descriptor::load_service_descriptor(service)?;
    if let Some(ep) = endpoint {
        let needle = ep.replace('_', "-");
        let found = desc
            .endpoints
            .iter()
            .find(|e| e.id == ep || e.id.replace('_', "-") == needle)
            .ok_or_else(|| WpsError::Validation(format!("endpoint not found: {ep}")))?;
        return Ok(serde_json::to_value(found)
            .map_err(|e| WpsError::Execution(format!("failed to serialize schema output: {e}")))?);
    }
    Ok(serde_json::to_value(desc)
        .map_err(|e| WpsError::Execution(format!("failed to serialize schema output: {e}")))?)
}
