use std::collections::HashMap;

use crate::descriptor::{self, EndpointDescriptor, ParamSpec, ServiceDescriptor};
use crate::error::WpsError;
use crate::scope_catalog::{self, ScopeType};

fn endpoint_aliases(endpoint_name: &str) -> Vec<String> {
    let mut aliases = Vec::new();
    if endpoint_name.starts_with("get-") && endpoint_name.ends_with("-list") {
        let inner = endpoint_name.trim_start_matches("get-").trim_end_matches("-list");
        if !inner.is_empty() {
            let plural = if inner.ends_with('s') {
                inner.to_string()
            } else {
                format!("{inner}s")
            };
            aliases.push(format!("list-{plural}"));
        }
    }
    aliases
}

fn find_endpoint<'a>(desc: &'a ServiceDescriptor, endpoint: &str) -> Option<&'a EndpointDescriptor> {
    let needle = endpoint.replace('_', "-");
    desc.endpoints.iter().find(|e| {
        if e.id == endpoint || e.id.replace('_', "-") == needle {
            return true;
        }
        let canonical = e.id.replace('_', "-");
        endpoint_aliases(&canonical).into_iter().any(|a| a == needle)
    })
}

fn supported_auth_types(ep: &EndpointDescriptor) -> Vec<String> {
    if ep.cookie_only {
        return vec!["cookie".to_string()];
    }
    if !ep.auth_types.is_empty() {
        let mut seen = std::collections::BTreeSet::new();
        for item in &ep.auth_types {
            let v = item.trim().to_ascii_lowercase();
            if !v.is_empty() {
                seen.insert(v);
            }
        }
        if !seen.is_empty() {
            return seen.into_iter().collect();
        }
    }
    if ep.scopes.is_empty() {
        return vec!["app".to_string(), "user".to_string()];
    }
    let app_scopes = scope_catalog::filter_supported(&ep.scopes, ScopeType::AppRole);
    let delegated_scopes = scope_catalog::filter_supported(&ep.scopes, ScopeType::Delegated);
    match (app_scopes.is_empty(), delegated_scopes.is_empty()) {
        (false, false) => vec!["app".to_string(), "user".to_string()],
        (false, true) => vec!["app".to_string()],
        (true, false) => vec!["user".to_string()],
        (true, true) => vec!["app".to_string(), "user".to_string()],
    }
}

fn recommended_auth_type(supported: &[String]) -> &'static str {
    if supported.iter().any(|v| v == "cookie") && supported.len() == 1 {
        return "cookie";
    }
    if supported.iter().any(|v| v == "app") {
        return "app";
    }
    if supported.iter().any(|v| v == "user") {
        return "user";
    }
    "app"
}

fn body_supported(method: &str) -> bool {
    matches!(method.to_ascii_uppercase().as_str(), "POST" | "PUT" | "PATCH" | "DELETE")
}

fn required_names(params: &[ParamSpec]) -> Vec<String> {
    params
        .iter()
        .filter(|p| p.required)
        .map(|p| p.name.clone())
        .collect::<Vec<_>>()
}

fn command_template(service: &str, ep: &EndpointDescriptor, auth_type: &str) -> String {
    let mut parts = vec![
        "wpscli".to_string(),
        service.to_string(),
        ep.id.replace('_', "-"),
    ];
    for p in ep.params.path.iter().filter(|p| p.required) {
        parts.push("--path-param".to_string());
        parts.push(format!("{}=<{}>", p.name, p.name));
    }
    for p in ep.params.query.iter().filter(|p| p.required) {
        parts.push("--query".to_string());
        parts.push(format!("{}=<{}>", p.name, p.name));
    }
    if body_supported(&ep.http_method) {
        parts.push("--body".to_string());
        parts.push("'{}'".to_string());
    }
    parts.push("--auth-type".to_string());
    parts.push(auth_type.to_string());
    parts.join(" ")
}

fn invoke_template(ep: &EndpointDescriptor) -> serde_json::Value {
    let mut path_params = HashMap::<String, String>::new();
    let mut query = HashMap::<String, String>::new();
    let mut headers = HashMap::<String, String>::new();
    for p in ep.params.path.iter().filter(|p| p.required) {
        path_params.insert(p.name.clone(), format!("<{}>", p.name));
    }
    for p in ep.params.query.iter().filter(|p| p.required) {
        query.insert(p.name.clone(), format!("<{}>", p.name));
    }
    for p in ep.params.header.iter().filter(|p| p.required) {
        headers.insert(p.name.clone(), format!("<{}>", p.name));
    }
    let body = if body_supported(&ep.http_method) {
        Some(serde_json::json!({}))
    } else {
        None
    };
    serde_json::json!({
        "path_params": path_params,
        "query": query,
        "headers": headers,
        "body": body
    })
}

fn endpoint_invoke_schema(service: &str, base_url: &str, ep: &EndpointDescriptor) -> serde_json::Value {
    let auth_types = supported_auth_types(ep);
    let auth_type = recommended_auth_type(&auth_types);
    serde_json::json!({
        "service": service,
        "endpoint": ep.id,
        "name": ep.name,
        "summary": ep.summary,
        "http": {
            "method": ep.http_method,
            "path": ep.path,
            "base_url": base_url
        },
        "auth": {
            "supported_auth_types": auth_types,
            "recommended_auth_type": auth_type,
            "cookie_only": ep.cookie_only
        },
        "scopes": ep.scopes,
        "params": {
            "path": ep.params.path,
            "query": ep.params.query,
            "header": ep.params.header,
            "required": {
                "path": required_names(&ep.params.path),
                "query": required_names(&ep.params.query),
                "header": required_names(&ep.params.header)
            }
        },
        "command_template": command_template(service, ep, auth_type),
        "invoke_template": invoke_template(ep)
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
            endpoint_invoke_schema(service, &desc.base_url, found)
        } else {
            serde_json::to_value(found)
                .map_err(|e| WpsError::Execution(format!("failed to serialize schema output: {e}")))?
        }
    } else if mode == "invoke" {
        let endpoints = desc
            .endpoints
            .iter()
            .map(|ep| {
                let auth_types = supported_auth_types(ep);
                let auth = recommended_auth_type(&auth_types);
                serde_json::json!({
                    "endpoint": ep.id,
                    "method": ep.http_method,
                    "path": ep.path,
                    "summary": ep.summary,
                    "supported_auth_types": auth_types,
                    "recommended_auth_type": auth,
                    "required_path_params": required_names(&ep.params.path),
                    "required_query_params": required_names(&ep.params.query),
                    "command_template": command_template(service, ep, auth)
                })
            })
            .collect::<Vec<_>>();
        serde_json::json!({
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
        let template = invoke_template(found);
        let text = serde_json::to_string_pretty(&template)
            .map_err(|e| WpsError::Execution(format!("failed to serialize invoke template: {e}")))?;
        std::fs::write(path, text)
            .map_err(|e| WpsError::Execution(format!("failed to write template file {path}: {e}")))?;
    }

    Ok(output)
}
