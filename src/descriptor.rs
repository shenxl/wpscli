use std::collections::{BTreeSet, HashMap};
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::error::WpsError;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ParamSpec {
    pub name: String,
    pub location: String,
    #[serde(rename = "ptype")]
    pub param_type: String,
    pub required: bool,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub enum_values: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EndpointParamGroup {
    #[serde(default)]
    pub path: Vec<ParamSpec>,
    #[serde(default)]
    pub query: Vec<ParamSpec>,
    #[serde(default)]
    pub header: Vec<ParamSpec>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EndpointDescriptor {
    pub id: String,
    pub doc_id: Option<u64>,
    pub name: String,
    #[serde(default)]
    pub summary: String,
    pub http_method: String,
    pub path: String,
    #[serde(default)]
    pub signature: String,
    #[serde(default)]
    pub scopes: Vec<String>,
    #[serde(default)]
    pub auth_types: Vec<String>,
    #[serde(default)]
    pub cookie_only: bool,
    #[serde(default)]
    pub params: EndpointParamGroup,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ServiceDescriptor {
    pub service: String,
    pub base_url: String,
    #[serde(default)]
    pub endpoints: Vec<EndpointDescriptor>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DescriptorManifest {
    pub version: String,
    pub generated_from: String,
    pub total_services: u64,
    pub total_endpoints: u64,
    #[serde(default)]
    pub services: Vec<HashMap<String, serde_json::Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DescriptorAuditIssue {
    pub severity: String,
    pub code: String,
    pub message: String,
    #[serde(default)]
    pub service: Option<String>,
    #[serde(default)]
    pub endpoint: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DescriptorAuditReport {
    pub ok: bool,
    pub declared_services: usize,
    pub discovered_descriptor_files: usize,
    pub loaded_services: usize,
    pub endpoint_count: usize,
    pub error_count: usize,
    pub warning_count: usize,
    pub issues: Vec<DescriptorAuditIssue>,
}

pub fn descriptor_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("descriptors")
}

pub fn load_manifest() -> Result<DescriptorManifest, WpsError> {
    let path = descriptor_dir().join("index.json");
    let text = std::fs::read_to_string(&path).map_err(|e| {
        WpsError::Descriptor(format!(
            "failed to read descriptor manifest {}: {e}",
            path.display()
        ))
    })?;
    serde_json::from_str(&text)
        .map_err(|e| WpsError::Descriptor(format!("failed to parse descriptor manifest: {e}")))
}

pub fn load_service_descriptor(service: &str) -> Result<ServiceDescriptor, WpsError> {
    let path = descriptor_dir().join(format!("{service}.json"));
    let text = std::fs::read_to_string(&path).map_err(|e| {
        WpsError::Descriptor(format!(
            "failed to read service descriptor {}: {e}",
            path.display()
        ))
    })?;
    serde_json::from_str(&text)
        .map_err(|e| WpsError::Descriptor(format!("failed to parse service descriptor: {e}")))
}

fn push_issue(
    issues: &mut Vec<DescriptorAuditIssue>,
    severity: &str,
    code: &str,
    message: String,
    service: Option<&str>,
    endpoint: Option<&str>,
) {
    issues.push(DescriptorAuditIssue {
        severity: severity.to_string(),
        code: code.to_string(),
        message,
        service: service.map(|s| s.to_string()),
        endpoint: endpoint.map(|s| s.to_string()),
    });
}

fn normalize_auth_types(auth_types: &[String]) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    for item in auth_types {
        let v = item.trim().to_ascii_lowercase();
        if !v.is_empty() {
            out.insert(v);
        }
    }
    out
}

fn validate_params(
    issues: &mut Vec<DescriptorAuditIssue>,
    service: &str,
    endpoint: &str,
    location: &str,
    params: &[ParamSpec],
) {
    for (idx, p) in params.iter().enumerate() {
        if p.name.trim().is_empty() {
            push_issue(
                issues,
                "error",
                "empty_param_name",
                format!("{service}/{endpoint} {location}[{idx}] has empty param name"),
                Some(service),
                Some(endpoint),
            );
        }
        if p.required && p.name.trim().is_empty() {
            push_issue(
                issues,
                "error",
                "invalid_required_param",
                format!("{service}/{endpoint} required {location}[{idx}] has empty name"),
                Some(service),
                Some(endpoint),
            );
        }
        if !p.location.trim().is_empty() && p.location != location {
            push_issue(
                issues,
                "warning",
                "param_location_mismatch",
                format!(
                    "{service}/{endpoint} param `{}` location mismatch: declared `{}` but grouped in `{}`",
                    p.name, p.location, location
                ),
                Some(service),
                Some(endpoint),
            );
        }
    }
}

pub fn audit_descriptors() -> Result<DescriptorAuditReport, WpsError> {
    let manifest = load_manifest()?;
    let descriptor_root = descriptor_dir();
    let mut descriptor_files = BTreeSet::new();
    let entries = std::fs::read_dir(&descriptor_root).map_err(|e| {
        WpsError::Descriptor(format!(
            "failed to read descriptor dir {}: {e}",
            descriptor_root.display()
        ))
    })?;
    for entry in entries {
        let entry = entry
            .map_err(|e| WpsError::Descriptor(format!("failed to read descriptor entry: {e}")))?;
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("json") {
            continue;
        }
        if path.file_name().and_then(|s| s.to_str()) == Some("index.json") {
            continue;
        }
        if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
            descriptor_files.insert(stem.to_string());
        }
    }

    let mut issues = Vec::new();
    let mut declared = BTreeSet::new();
    let mut endpoint_count = 0usize;
    let mut loaded_services = 0usize;

    for record in manifest.services {
        let svc = record.get("service").and_then(|v| v.as_str()).ok_or_else(|| {
            WpsError::Descriptor("invalid service record in manifest: missing `service`".to_string())
        })?;
        let file = record
            .get("file")
            .and_then(|v| v.as_str())
            .unwrap_or_default();
        declared.insert(svc.to_string());

        if file.trim().is_empty() {
            push_issue(
                &mut issues,
                "warning",
                "missing_file_field",
                format!("manifest service `{svc}` has empty file field"),
                Some(svc),
                None,
            );
        } else if file != format!("{svc}.json") {
            push_issue(
                &mut issues,
                "warning",
                "file_name_mismatch",
                format!("manifest service `{svc}` expects file `{svc}.json` but got `{file}`"),
                Some(svc),
                None,
            );
        }

        if !descriptor_files.contains(svc) {
            push_issue(
                &mut issues,
                "error",
                "missing_descriptor_file",
                format!("descriptor file not found for service `{svc}`"),
                Some(svc),
                None,
            );
            continue;
        }

        let desc = load_service_descriptor(svc)?;
        loaded_services += 1;

        if desc.service != svc {
            push_issue(
                &mut issues,
                "warning",
                "service_name_mismatch",
                format!("descriptor `{svc}` declares service name `{}`", desc.service),
                Some(svc),
                None,
            );
        }
        if desc.base_url.trim().is_empty() {
            push_issue(
                &mut issues,
                "warning",
                "empty_base_url",
                format!("service `{svc}` has empty base_url"),
                Some(svc),
                None,
            );
        }

        for ep in &desc.endpoints {
            endpoint_count += 1;
            if ep.id.trim().is_empty() {
                push_issue(
                    &mut issues,
                    "error",
                    "empty_endpoint_id",
                    format!("service `{svc}` has endpoint with empty id"),
                    Some(svc),
                    None,
                );
                continue;
            }
            if ep.http_method.trim().is_empty() {
                push_issue(
                    &mut issues,
                    "error",
                    "empty_http_method",
                    format!("service `{svc}` endpoint `{}` has empty http_method", ep.id),
                    Some(svc),
                    Some(&ep.id),
                );
            } else if !matches!(
                ep.http_method.to_ascii_uppercase().as_str(),
                "GET" | "POST" | "PUT" | "PATCH" | "DELETE"
            ) {
                push_issue(
                    &mut issues,
                    "error",
                    "invalid_http_method",
                    format!(
                        "service `{svc}` endpoint `{}` has invalid http_method `{}`",
                        ep.id, ep.http_method
                    ),
                    Some(svc),
                    Some(&ep.id),
                );
            }
            if ep.path.trim().is_empty() {
                push_issue(
                    &mut issues,
                    "error",
                    "empty_path",
                    format!("service `{svc}` endpoint `{}` has empty path", ep.id),
                    Some(svc),
                    Some(&ep.id),
                );
            } else if !ep.path.starts_with('/') {
                push_issue(
                    &mut issues,
                    "error",
                    "invalid_path",
                    format!(
                        "service `{svc}` endpoint `{}` path should start with `/`: `{}`",
                        ep.id, ep.path
                    ),
                    Some(svc),
                    Some(&ep.id),
                );
            }

            validate_params(&mut issues, svc, &ep.id, "path", &ep.params.path);
            validate_params(&mut issues, svc, &ep.id, "query", &ep.params.query);
            validate_params(&mut issues, svc, &ep.id, "header", &ep.params.header);

            let auth = normalize_auth_types(&ep.auth_types);
            for item in &auth {
                if !matches!(item.as_str(), "app" | "user" | "cookie") {
                    push_issue(
                        &mut issues,
                        "error",
                        "invalid_auth_type",
                        format!("service `{svc}` endpoint `{}` has invalid auth type `{item}`", ep.id),
                        Some(svc),
                        Some(&ep.id),
                    );
                }
            }

            if ep.cookie_only {
                if auth.len() > 1 || (auth.len() == 1 && !auth.contains("cookie")) {
                    push_issue(
                        &mut issues,
                        "error",
                        "cookie_auth_conflict",
                        format!(
                            "service `{svc}` endpoint `{}` is cookie_only but auth_types={:?}",
                            ep.id, ep.auth_types
                        ),
                        Some(svc),
                        Some(&ep.id),
                    );
                }
            } else if auth.len() == 1 && auth.contains("cookie") {
                push_issue(
                    &mut issues,
                    "warning",
                    "cookie_only_recommended",
                    format!(
                        "service `{svc}` endpoint `{}` auth_types only cookie but cookie_only=false",
                        ep.id
                    ),
                    Some(svc),
                    Some(&ep.id),
                );
            }
        }
    }

    for svc in &descriptor_files {
        if !declared.contains(svc) {
            push_issue(
                &mut issues,
                "warning",
                "orphan_descriptor_file",
                format!("descriptor `{svc}.json` exists but is not declared in index.json"),
                Some(svc),
                None,
            );
        }
    }

    if declared.len() != descriptor_files.len() {
        push_issue(
            &mut issues,
            "warning",
            "service_count_mismatch",
            format!(
                "manifest declares {} services but descriptor directory has {} files",
                declared.len(),
                descriptor_files.len()
            ),
            None,
            None,
        );
    }

    let error_count = issues.iter().filter(|x| x.severity == "error").count();
    let warning_count = issues.iter().filter(|x| x.severity == "warning").count();
    Ok(DescriptorAuditReport {
        ok: error_count == 0,
        declared_services: declared.len(),
        discovered_descriptor_files: descriptor_files.len(),
        loaded_services,
        endpoint_count,
        error_count,
        warning_count,
        issues,
    })
}
