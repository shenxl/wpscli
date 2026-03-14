use crate::descriptor::{self, ServiceDescriptor};
use crate::error::WpsError;
use crate::help_schema_contract;

fn find_endpoint<'a>(
    desc: &'a ServiceDescriptor,
    endpoint: &str,
) -> Option<&'a crate::descriptor::EndpointDescriptor> {
    let needle = endpoint.replace('_', "-");
    desc.endpoints.iter().find(|e| {
        if e.id == endpoint || e.id.replace('_', "-") == needle {
            return true;
        }
        let canonical = e.id.replace('_', "-");
        help_schema_contract::endpoint_aliases(&canonical)
            .into_iter()
            .any(|a| a == needle)
    })
}

pub fn run(
    service: &str,
    endpoint: Option<&str>,
    mode: &str,
    emit_template: Option<&str>,
) -> Result<serde_json::Value, WpsError> {
    let desc = descriptor::load_service_descriptor(service)?;
    if !matches!(mode, "raw" | "invoke") {
        return Err(WpsError::Validation(format!(
            "invalid schema mode: {mode} (supported: raw, invoke)"
        )));
    }

    let output = if let Some(ep_name) = endpoint {
        let found = find_endpoint(&desc, ep_name)
            .ok_or_else(|| WpsError::Validation(format!("endpoint not found: {ep_name}")))?;
        if mode == "invoke" {
            serde_json::to_value(help_schema_contract::endpoint_contract(
                service,
                &desc.base_url,
                found,
            ))
            .map_err(|e| WpsError::Execution(format!("failed to serialize invoke schema: {e}")))?
        } else {
            serde_json::to_value(found)
                .map_err(|e| WpsError::Execution(format!("failed to serialize schema output: {e}")))?
        }
    } else if mode == "invoke" {
        let endpoints = desc
            .endpoints
            .iter()
            .map(|ep| {
                let auth_types = help_schema_contract::supported_auth_types(ep);
                let auth = help_schema_contract::recommended_auth_type(&auth_types);
                serde_json::json!({
                    "endpoint": ep.id,
                    "name": ep.name,
                    "method": ep.http_method,
                    "path": ep.path,
                    "summary": ep.summary,
                    "supported_auth_types": auth_types,
                    "recommended_auth_type": auth,
                    "required_path_params": help_schema_contract::required_names(&ep.params.path),
                    "required_query_params": help_schema_contract::required_names(&ep.params.query),
                    "required_header_params": help_schema_contract::required_names(&ep.params.header),
                    "command_template": help_schema_contract::command_template(service, ep, &auth),
                    "invoke_template": help_schema_contract::invoke_template(ep)
                })
            })
            .collect::<Vec<_>>();
        serde_json::json!({
            "contract_version": help_schema_contract::HELP_SCHEMA_CONTRACT_VERSION,
            "service": desc.service,
            "base_url": desc.base_url,
            "endpoint_count": endpoints.len(),
            "endpoints": endpoints
        })
    } else {
        serde_json::to_value(&desc)
            .map_err(|e| WpsError::Execution(format!("failed to serialize schema output: {e}")))?
    };

    if let Some(path) = emit_template {
        let Some(ep_name) = endpoint else {
            return Err(WpsError::Validation(
                "--emit-template requires endpoint argument: wpscli schema <service> <endpoint>".to_string(),
            ));
        };
        let found = find_endpoint(&desc, ep_name)
            .ok_or_else(|| WpsError::Validation(format!("endpoint not found: {ep_name}")))?;
        let template = help_schema_contract::invoke_template(found);
        let text = serde_json::to_string_pretty(&template)
            .map_err(|e| WpsError::Execution(format!("failed to serialize invoke template: {e}")))?;
        std::fs::write(path, text)
            .map_err(|e| WpsError::Execution(format!("failed to write template file {path}: {e}")))?;
    }

    Ok(output)
}
