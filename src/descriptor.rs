use std::collections::HashMap;
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

pub fn descriptor_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("descriptors")
}

pub fn load_manifest() -> Result<DescriptorManifest, WpsError> {
    let path = descriptor_dir().join("index.json");
    let text = std::fs::read_to_string(&path).map_err(|e| {
        WpsError::Descriptor(format!("failed to read descriptor manifest {}: {e}", path.display()))
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

