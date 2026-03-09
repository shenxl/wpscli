use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::json;

use crate::auth;

fn ts_secs(st: SystemTime) -> u64 {
    st.duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn metadata_mtime_secs(path: &Path) -> Option<u64> {
    let md = std::fs::metadata(path).ok()?;
    let mt = md.modified().ok()?;
    Some(ts_secs(mt))
}

fn find_wps_cli_repo_root(start: &Path) -> Option<PathBuf> {
    let mut cur = start.to_path_buf();
    loop {
        let cargo_toml = cur.join("Cargo.toml");
        if cargo_toml.exists() {
            let raw = std::fs::read_to_string(&cargo_toml).unwrap_or_default();
            if raw.contains("name = \"wps-cli\"") {
                return Some(cur);
            }
        }
        if !cur.pop() {
            break;
        }
    }
    None
}

fn source_probe(repo_root: &Path, exe_mtime: Option<u64>) -> serde_json::Value {
    let watch_files = [
        "Cargo.toml",
        "src/main.rs",
        "src/helpers/mod.rs",
        "src/helpers/dbsheet.rs",
    ];
    let mut latest_src_mtime: Option<u64> = None;
    for rel in watch_files {
        let p = repo_root.join(rel);
        if let Some(mt) = metadata_mtime_secs(&p) {
            latest_src_mtime = Some(latest_src_mtime.map(|v| v.max(mt)).unwrap_or(mt));
        }
    }
    let stale = match (latest_src_mtime, exe_mtime) {
        (Some(src), Some(bin)) => src > bin,
        _ => false,
    };
    let hint = if stale {
        Some("检测到源码修改时间晚于当前可执行文件，建议执行 `cargo install --path . --bin wpscli --force`".to_string())
    } else {
        None
    };
    json!({
        "repo_root": repo_root.display().to_string(),
        "watch_files": watch_files,
        "latest_source_mtime": latest_src_mtime,
        "binary_mtime": exe_mtime,
        "binary_older_than_source": stale,
        "hint": hint
    })
}

pub fn run() -> serde_json::Value {
    let exe = std::env::current_exe().ok();
    let exe_path = exe
        .as_ref()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|| "<unknown>".to_string());
    let exe_mtime = exe
        .as_ref()
        .and_then(|p| metadata_mtime_secs(p.as_path()));
    let is_cargo_target_binary = exe_path.contains("/target/debug/") || exe_path.contains("/target/release/");

    let cwd = std::env::current_dir().ok();
    let repo_root = cwd
        .as_ref()
        .and_then(|d| find_wps_cli_repo_root(d));
    let source = repo_root
        .as_ref()
        .map(|r| source_probe(r, exe_mtime))
        .unwrap_or_else(|| {
            json!({
                "repo_root": null,
                "binary_older_than_source": null,
                "hint": "当前目录未检测到 wps-cli 源码仓库，跳过源码新旧对比"
            })
        });

    let creds = auth::load_credentials();
    let cache = auth::load_token_cache();
    let now = ts_secs(SystemTime::now());
    let user_expired = cache.user_expires_at.map(|exp| exp <= now).unwrap_or(true);
    let has_ak_effective = std::env::var("WPS_CLI_AK").ok().filter(|v| !v.is_empty()).is_some()
        || creds.ak.as_ref().filter(|v| !v.is_empty()).is_some();
    let has_sk_effective = std::env::var("WPS_CLI_SK").ok().filter(|v| !v.is_empty()).is_some()
        || creds.sk.as_ref().filter(|v| !v.is_empty()).is_some();

    let supports_dbsheet = crate::helpers::command("dbsheet").is_some();
    let mut checks = vec![
        json!({
            "name": "dbsheet_command_registered",
            "status": if supports_dbsheet { "pass" } else { "fail" },
            "message": if supports_dbsheet { "binary 包含 `dbsheet` 子命令" } else { "binary 未包含 `dbsheet` 子命令" },
            "suggested_action": if supports_dbsheet { serde_json::Value::Null } else { json!("执行 `cargo install --path . --bin wpscli --force`") }
        }),
        json!({
            "name": "auth_ak_sk_effective",
            "status": if has_ak_effective && has_sk_effective { "pass" } else { "warn" },
            "message": if has_ak_effective && has_sk_effective { "AK/SK 已就绪" } else { "AK/SK 缺失，app token 与 user refresh 可能失败" },
            "suggested_action": if has_ak_effective && has_sk_effective { serde_json::Value::Null } else { json!("运行 `wpscli auth setup` 或设置环境变量 WPS_CLI_AK / WPS_CLI_SK") }
        }),
    ];
    if source.get("binary_older_than_source").and_then(|v| v.as_bool()) == Some(true) {
        checks.push(json!({
            "name": "binary_freshness",
            "status": "warn",
            "message": "源码时间晚于当前二进制，可能出现“文档有命令但命令不可用”",
            "suggested_action": "执行 `cargo install --path . --bin wpscli --force`"
        }));
    } else {
        checks.push(json!({
            "name": "binary_freshness",
            "status": "pass",
            "message": "未发现明显的二进制落后源码风险",
            "suggested_action": serde_json::Value::Null
        }));
    }

    json!({
        "ok": true,
        "doctor": {
            "version": env!("CARGO_PKG_VERSION"),
            "binary": {
                "path": exe_path,
                "mtime": exe_mtime,
                "is_cargo_target_binary": is_cargo_target_binary,
                "supports_dbsheet": supports_dbsheet
            },
            "workspace": {
                "cwd": cwd.map(|p| p.display().to_string()),
                "source_probe": source
            },
            "auth_probe": {
                "config_dir": auth::config_dir().display().to_string(),
                "credentials_exists": auth::credentials_path().exists(),
                "token_cache_exists": auth::token_cache_path().exists(),
                "legacy_plaintext_credentials_exists": auth::legacy_credentials_path().exists(),
                "legacy_plaintext_token_cache_exists": auth::legacy_token_cache_path().exists(),
                "secure_storage": "aes256_gcm_with_os_keyring_fallback",
                "has_ak_effective": has_ak_effective,
                "has_sk_effective": has_sk_effective,
                "has_user_refresh_token": cache.user_refresh_token.is_some(),
                "has_user_access_token": cache.user_access_token.is_some(),
                "user_expires_at": cache.user_expires_at,
                "user_token_expired": user_expired
            },
            "checks": checks
        }
    })
}
