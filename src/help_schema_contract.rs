use std::collections::{BTreeSet, HashMap};

use serde::Serialize;

use crate::descriptor::{EndpointDescriptor, ParamSpec};
use crate::scope_catalog::{self, ScopeType};

pub const HELP_SCHEMA_CONTRACT_VERSION: &str = "1.0.0";

#[derive(Debug, Clone, Serialize)]
pub struct EndpointHelpSchemaContract {
    pub contract_version: String,
    pub service: String,
    pub endpoint: String,
    pub name: String,
    pub summary: String,
    pub http: HttpContract,
    pub auth: AuthContract,
    pub scopes: Vec<String>,
    pub params: ParamContract,
    pub command_template: String,
    pub invoke_template: serde_json::Value,
}

#[derive(Debug, Clone, Serialize)]
pub struct HttpContract {
    pub method: String,
    pub path: String,
    pub base_url: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct AuthContract {
    pub supported_auth_types: Vec<String>,
    pub recommended_auth_type: String,
    pub cookie_only: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct ParamContract {
    pub path: Vec<ParamSpec>,
    pub query: Vec<ParamSpec>,
    pub header: Vec<ParamSpec>,
    pub required: RequiredParamContract,
}

#[derive(Debug, Clone, Serialize)]
pub struct RequiredParamContract {
    pub path: Vec<String>,
    pub query: Vec<String>,
    pub header: Vec<String>,
}

pub fn endpoint_command_name(endpoint_id: &str) -> String {
    endpoint_id.replace('_', "-")
}

pub fn endpoint_aliases(endpoint_name: &str) -> Vec<String> {
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

pub fn body_supported(method: &str) -> bool {
    matches!(method.to_ascii_uppercase().as_str(), "POST" | "PUT" | "PATCH" | "DELETE")
}

pub fn required_names(params: &[ParamSpec]) -> Vec<String> {
    params
        .iter()
        .filter(|p| p.required)
        .map(|p| p.name.clone())
        .collect::<Vec<_>>()
}

pub fn supported_auth_types(ep: &EndpointDescriptor) -> Vec<String> {
    if ep.cookie_only {
        return vec!["cookie".to_string()];
    }
    if !ep.auth_types.is_empty() {
        let mut seen = BTreeSet::new();
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

pub fn recommended_auth_type(supported: &[String]) -> String {
    if supported.iter().any(|v| v == "cookie") && supported.len() == 1 {
        return "cookie".to_string();
    }
    if supported.iter().any(|v| v == "app") {
        return "app".to_string();
    }
    if supported.iter().any(|v| v == "user") {
        return "user".to_string();
    }
    "app".to_string()
}

pub fn command_template(service: &str, ep: &EndpointDescriptor, auth_type: &str) -> String {
    let mut parts = vec![
        "wpscli".to_string(),
        service.to_string(),
        endpoint_command_name(&ep.id),
    ];
    for p in ep.params.path.iter().filter(|p| p.required) {
        parts.push("--path-param".to_string());
        parts.push(format!("{}=<{}>", p.name, p.name));
    }
    for p in ep.params.query.iter().filter(|p| p.required) {
        parts.push("--query".to_string());
        parts.push(format!("{}=<{}>", p.name, p.name));
    }
    for p in ep.params.header.iter().filter(|p| p.required) {
        parts.push("--header".to_string());
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

pub fn invoke_template(ep: &EndpointDescriptor) -> serde_json::Value {
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

pub fn endpoint_contract(service: &str, base_url: &str, ep: &EndpointDescriptor) -> EndpointHelpSchemaContract {
    let auth_types = supported_auth_types(ep);
    let auth_type = recommended_auth_type(&auth_types);
    EndpointHelpSchemaContract {
        contract_version: HELP_SCHEMA_CONTRACT_VERSION.to_string(),
        service: service.to_string(),
        endpoint: ep.id.clone(),
        name: ep.name.clone(),
        summary: ep.summary.clone(),
        http: HttpContract {
            method: ep.http_method.clone(),
            path: ep.path.clone(),
            base_url: base_url.to_string(),
        },
        auth: AuthContract {
            supported_auth_types: auth_types,
            recommended_auth_type: auth_type.clone(),
            cookie_only: ep.cookie_only,
        },
        scopes: ep.scopes.clone(),
        params: ParamContract {
            path: ep.params.path.clone(),
            query: ep.params.query.clone(),
            header: ep.params.header.clone(),
            required: RequiredParamContract {
                path: required_names(&ep.params.path),
                query: required_names(&ep.params.query),
                header: required_names(&ep.params.header),
            },
        },
        command_template: command_template(service, ep, &auth_type),
        invoke_template: invoke_template(ep),
    }
}
