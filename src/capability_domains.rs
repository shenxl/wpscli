use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::descriptor;
use crate::error::WpsError;

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct TopDomainRegistry {
    #[serde(default)]
    pub domains: Vec<TopDomain>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TopDomain {
    pub id: String,
    pub title: String,
    pub description: String,
    #[serde(default)]
    pub services: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TopDomainAudit {
    pub ok: bool,
    pub domain_count: usize,
    pub declared_services: usize,
    pub manifest_services: usize,
    pub coverage_ratio: f64,
    pub missing_services: Vec<String>,
    pub duplicate_services: Vec<String>,
    pub unknown_services: Vec<String>,
    pub domain_service_counts: BTreeMap<String, usize>,
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

pub fn registry_path() -> PathBuf {
    repo_root().join("registry/top_domains.yaml")
}

pub fn load_registry() -> Result<TopDomainRegistry, WpsError> {
    let path = registry_path();
    let text = std::fs::read_to_string(&path)
        .map_err(|e| WpsError::Execution(format!("failed to read {}: {e}", path.display())))?;
    serde_yaml::from_str::<TopDomainRegistry>(&text)
        .map_err(|e| WpsError::Execution(format!("failed to parse {}: {e}", path.display())))
}

pub fn audit_against_manifest() -> Result<TopDomainAudit, WpsError> {
    let registry = load_registry()?;
    let manifest = descriptor::load_manifest()?;
    let mut manifest_set = BTreeSet::new();
    for entry in manifest.services {
        if let Some(service) = entry.get("service").and_then(|v| v.as_str()) {
            manifest_set.insert(service.to_string());
        }
    }

    let mut seen = BTreeSet::new();
    let mut duplicates = BTreeSet::new();
    let mut unknown = BTreeSet::new();
    let mut per_domain = BTreeMap::new();
    for domain in &registry.domains {
        per_domain.insert(domain.id.clone(), domain.services.len());
        for service in &domain.services {
            if !manifest_set.contains(service) {
                unknown.insert(service.clone());
            }
            if !seen.insert(service.clone()) {
                duplicates.insert(service.clone());
            }
        }
    }

    let missing = manifest_set
        .iter()
        .filter(|svc| !seen.contains(*svc))
        .cloned()
        .collect::<Vec<_>>();

    let manifest_services = manifest_set.len();
    let declared_services = seen.len();
    let coverage_ratio = if manifest_services == 0 {
        0.0
    } else {
        declared_services as f64 / manifest_services as f64
    };
    let ok = missing.is_empty() && duplicates.is_empty() && unknown.is_empty();
    Ok(TopDomainAudit {
        ok,
        domain_count: registry.domains.len(),
        declared_services,
        manifest_services,
        coverage_ratio,
        missing_services: missing,
        duplicate_services: duplicates.into_iter().collect(),
        unknown_services: unknown.into_iter().collect(),
        domain_service_counts: per_domain,
    })
}
