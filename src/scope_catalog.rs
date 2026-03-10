use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::OnceLock;

use serde::Deserialize;

const EMBEDDED_SCOPE_CATALOG_JSON: &str = include_str!("../config/scope_catalog.json");

#[derive(Debug, Clone, Copy, Default)]
pub struct ScopeSupport {
    pub known: bool,
    pub app_role: bool,
    pub delegated: bool,
}

#[derive(Debug, Clone, Copy)]
pub enum ScopeType {
    AppRole,
    Delegated,
}

#[derive(Debug, Clone)]
pub struct ScopeAvailability {
    pub source: String,
    pub available: Vec<String>,
    pub missing: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct ScopeCatalogDoc {
    #[serde(default)]
    scopes: Vec<ScopeCatalogItem>,
}

#[derive(Debug, Deserialize)]
struct ScopeCatalogItem {
    scope_name: String,
    #[serde(default)]
    app_role: bool,
    #[serde(default)]
    delegated: bool,
}

#[derive(Debug, Default)]
struct ScopeCatalogState {
    source: String,
    index: HashMap<String, ScopeSupport>,
}

static SCOPE_CATALOG: OnceLock<ScopeCatalogState> = OnceLock::new();

fn build_index(doc: ScopeCatalogDoc) -> HashMap<String, ScopeSupport> {
    let mut index = HashMap::new();
    for it in doc.scopes {
        if it.scope_name.trim().is_empty() {
            continue;
        }
        index.insert(
            it.scope_name,
            ScopeSupport {
                known: true,
                app_role: it.app_role,
                delegated: it.delegated,
            },
        );
    }
    index
}

fn parse_catalog(text: &str) -> Option<ScopeCatalogDoc> {
    serde_json::from_str::<ScopeCatalogDoc>(text).ok()
}

fn load_catalog_from_path(path: &PathBuf) -> Option<ScopeCatalogState> {
    let raw = std::fs::read_to_string(path).ok()?;
    let doc = parse_catalog(&raw)?;
    Some(ScopeCatalogState {
        source: path.display().to_string(),
        index: build_index(doc),
    })
}

fn init_catalog() -> ScopeCatalogState {
    if let Ok(v) = std::env::var("WPSCLI_SCOPE_CATALOG_PATH") {
        let path = PathBuf::from(v);
        if path.exists() {
            if let Some(state) = load_catalog_from_path(&path) {
                return state;
            }
        }
    }
    if let Some(doc) = parse_catalog(EMBEDDED_SCOPE_CATALOG_JSON) {
        return ScopeCatalogState {
            source: "embedded:config/scope_catalog.json".to_string(),
            index: build_index(doc),
        };
    }
    ScopeCatalogState {
        source: "empty".to_string(),
        index: HashMap::new(),
    }
}

fn state() -> &'static ScopeCatalogState {
    SCOPE_CATALOG.get_or_init(init_catalog)
}

pub fn source_label() -> String {
    state().source.clone()
}

pub fn support(scope_name: &str) -> ScopeSupport {
    state()
        .index
        .get(scope_name)
        .copied()
        .unwrap_or(ScopeSupport {
            known: false,
            app_role: false,
            delegated: false,
        })
}

pub fn analyze_required(required: &[String], scope_type: ScopeType) -> ScopeAvailability {
    let mut available = Vec::new();
    let mut missing = Vec::new();
    for scope_name in required {
        let s = support(scope_name);
        let ok = match scope_type {
            ScopeType::AppRole => s.app_role,
            ScopeType::Delegated => s.delegated,
        };
        if ok {
            available.push(scope_name.clone());
        } else {
            missing.push(scope_name.clone());
        }
    }
    ScopeAvailability {
        source: source_label(),
        available,
        missing,
    }
}

pub fn filter_supported(scopes: &[String], scope_type: ScopeType) -> Vec<String> {
    let mut out = Vec::new();
    for scope_name in scopes {
        let s = support(scope_name);
        let ok = match scope_type {
            ScopeType::AppRole => s.app_role,
            ScopeType::Delegated => s.delegated,
        };
        if ok {
            out.push(scope_name.clone());
        }
    }
    out
}

pub fn filter_unknown(scopes: &[String]) -> Vec<String> {
    let mut out = Vec::new();
    for scope_name in scopes {
        if !support(scope_name).known {
            out.push(scope_name.clone());
        }
    }
    out
}
