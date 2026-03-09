use std::collections::{BTreeSet, HashMap};
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;

use base64::engine::general_purpose::{URL_SAFE, URL_SAFE_NO_PAD};
use base64::Engine;
use chrono::Utc;
use serde_json::{Map, Value};

use crate::auth::{self, AuthType};
use crate::error::WpsError;

pub const WPS_APP_FILES_SKILL: &str = "wps-app-files";

#[derive(Debug, Clone)]
pub struct ScopePreflight {
    pub required_scopes: Vec<String>,
    pub token_scopes: Vec<String>,
    pub missing_scopes: Vec<String>,
    pub check_mode: String,
    pub reauth_hint: String,
}

impl ScopePreflight {
    pub fn to_json(&self) -> Value {
        serde_json::json!({
            "required_scopes": self.required_scopes,
            "token_scopes": self.token_scopes,
            "missing_scopes": self.missing_scopes,
            "check_mode": self.check_mode,
            "reauth_hint": self.reauth_hint
        })
    }
}

pub async fn ensure_user_scope(
    auth_type: &str,
    required_scopes: &[&str],
    dry_run: bool,
) -> Result<ScopePreflight, WpsError> {
    let required = required_scopes
        .iter()
        .map(|s| s.to_string())
        .collect::<Vec<_>>();
    let reauth_hint = format!(
        "Run `wpscli auth login --user --mode local --scope {}` and retry.",
        required_scopes.join(",")
    );

    if auth_type != "user" {
        return Err(WpsError::Auth(
            "该命令需要用户身份（user token），请添加 `--user-token` 或 `--auth-type user`".to_string(),
        ));
    }

    if dry_run {
        return Ok(ScopePreflight {
            required_scopes: required,
            token_scopes: vec![],
            missing_scopes: vec![],
            check_mode: "skipped_dry_run".to_string(),
            reauth_hint,
        });
    }

    let token = auth::get_access_token(AuthType::User).await?;
    let token_scopes = extract_token_scopes(&token);
    if token_scopes.is_empty() {
        return Ok(ScopePreflight {
            required_scopes: required,
            token_scopes,
            missing_scopes: vec![],
            check_mode: "token_scope_unknown".to_string(),
            reauth_hint,
        });
    }

    let got = token_scopes.iter().cloned().collect::<BTreeSet<_>>();
    let missing = required_scopes
        .iter()
        .filter(|s| !got.contains(**s))
        .map(|s| s.to_string())
        .collect::<Vec<_>>();
    if !missing.is_empty() {
        return Err(WpsError::Auth(format!(
            "user token scope 不足，缺少: {}. {}",
            missing.join(","),
            reauth_hint
        )));
    }

    Ok(ScopePreflight {
        required_scopes: required,
        token_scopes,
        missing_scopes: vec![],
        check_mode: "token_scope_claim".to_string(),
        reauth_hint,
    })
}

pub struct SkillStateStore {
    pub skill_name: String,
    pub base_dir: PathBuf,
}

impl SkillStateStore {
    pub fn new(skill_name: &str) -> Result<Self, WpsError> {
        let base_dir = auth::config_dir().join("skills").join(skill_name);
        std::fs::create_dir_all(&base_dir)
            .map_err(|e| WpsError::Execution(format!("failed to create skill state dir: {e}")))?;
        Ok(Self {
            skill_name: skill_name.to_string(),
            base_dir,
        })
    }

    pub fn registry_path(&self, name: &str) -> PathBuf {
        self.base_dir.join(name)
    }

    pub fn operation_log_path(&self) -> PathBuf {
        self.base_dir.join("operation_log.jsonl")
    }

    pub fn load_registry_map(&self, name: &str) -> Result<Map<String, Value>, WpsError> {
        let path = self.registry_path(name);
        if !path.exists() {
            return Ok(Map::new());
        }
        let raw = std::fs::read_to_string(&path)
            .map_err(|e| WpsError::Execution(format!("failed to read {}: {e}", path.display())))?;
        let value: Value = serde_json::from_str(&raw)
            .map_err(|e| WpsError::Execution(format!("failed to parse {}: {e}", path.display())))?;
        Ok(value.as_object().cloned().unwrap_or_default())
    }

    pub fn save_registry_map(&self, name: &str, map: &Map<String, Value>) -> Result<(), WpsError> {
        let path = self.registry_path(name);
        let payload = serde_json::to_string_pretty(map)
            .map_err(|e| WpsError::Execution(format!("failed to serialize {}: {e}", path.display())))?;
        std::fs::write(&path, payload)
            .map_err(|e| WpsError::Execution(format!("failed to write {}: {e}", path.display())))?;
        Ok(())
    }

    pub fn append_operation_log(&self, action: &str, status: &str, detail: Value) -> Result<(), WpsError> {
        let path = self.operation_log_path();
        let line = serde_json::json!({
            "time": Utc::now().to_rfc3339(),
            "skill": self.skill_name,
            "action": action,
            "status": status,
            "detail": detail,
        });
        let mut f = OpenOptions::new()
            .append(true)
            .create(true)
            .open(&path)
            .map_err(|e| WpsError::Execution(format!("failed to open {}: {e}", path.display())))?;
        writeln!(
            f,
            "{}",
            serde_json::to_string(&line)
                .map_err(|e| WpsError::Execution(format!("failed to serialize operation log: {e}")))?
        )
        .map_err(|e| WpsError::Execution(format!("failed to append {}: {e}", path.display())))?;
        Ok(())
    }

    pub fn read_recent_operations(&self, limit: usize) -> Result<Vec<Value>, WpsError> {
        let path = self.operation_log_path();
        if !path.exists() {
            return Ok(vec![]);
        }
        let raw = std::fs::read_to_string(&path)
            .map_err(|e| WpsError::Execution(format!("failed to read {}: {e}", path.display())))?;
        let mut out = Vec::new();
        for line in raw.lines().filter(|l| !l.trim().is_empty()) {
            if let Ok(v) = serde_json::from_str::<Value>(line) {
                out.push(v);
            }
        }
        if out.len() > limit {
            Ok(out.split_off(out.len() - limit))
        } else {
            Ok(out)
        }
    }
}

pub fn default_state_paths(skill_name: &str) -> HashMap<String, String> {
    let base = auth::config_dir().join("skills").join(skill_name);
    HashMap::from([
        ("base_dir".to_string(), base.display().to_string()),
        (
            "app_registry".to_string(),
            base.join("app_registry.json").display().to_string(),
        ),
        (
            "file_registry".to_string(),
            base.join("file_registry.json").display().to_string(),
        ),
        (
            "operation_log".to_string(),
            base.join("operation_log.jsonl").display().to_string(),
        ),
    ])
}

fn extract_token_scopes(access_token: &str) -> Vec<String> {
    let mut parts = access_token.split('.');
    let _head = parts.next();
    let payload = parts.next().unwrap_or_default();
    if payload.is_empty() {
        return vec![];
    }
    let bytes = URL_SAFE_NO_PAD
        .decode(payload)
        .or_else(|_| URL_SAFE.decode(payload))
        .unwrap_or_default();
    if bytes.is_empty() {
        return vec![];
    }
    let value: Value = serde_json::from_slice(&bytes).unwrap_or(Value::Null);
    let mut scopes = BTreeSet::new();
    if let Some(s) = value.get("scope").and_then(|v| v.as_str()) {
        for part in s.replace(',', " ").split_whitespace() {
            scopes.insert(part.to_string());
        }
    }
    if let Some(arr) = value.get("scopes").and_then(|v| v.as_array()) {
        for v in arr {
            if let Some(s) = v.as_str() {
                scopes.insert(s.to_string());
            }
        }
    }
    scopes.into_iter().collect()
}
