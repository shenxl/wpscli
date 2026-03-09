use std::path::Path;
use std::{collections::BTreeMap, fs};

use crate::descriptor;
use crate::error::WpsError;

fn slug(s: &str) -> String {
    s.chars()
        .map(|c| if c.is_ascii_alphanumeric() || c == '-' || c == '_' { c } else { '-' })
        .collect::<String>()
        .trim_matches('-')
        .to_ascii_lowercase()
}

fn resource_from_path(path: &str, service: &str) -> String {
    let prefix = format!("/{service}/");
    let v7 = if path.starts_with("/v7/") {
        path.trim_start_matches("/v7/")
    } else {
        path.trim_start_matches('/')
    };
    let normalized = v7.trim_start_matches("coop/").trim_start_matches(&prefix);
    let mut parts = normalized.split('/');
    parts
        .next()
        .filter(|p| !p.is_empty() && !p.starts_with('{'))
        .unwrap_or("root")
        .to_string()
}

pub fn generate(out_dir: &Path) -> Result<serde_json::Value, WpsError> {
    fs::create_dir_all(out_dir)
        .map_err(|e| WpsError::Execution(format!("failed to create output directory: {e}")))?;
    // Re-generate cleanly to avoid stale files in previous format.
    if out_dir.exists() {
        for entry in fs::read_dir(out_dir)
            .map_err(|e| WpsError::Execution(format!("failed to read output directory: {e}")))?
        {
            let p = entry
                .map_err(|e| WpsError::Execution(format!("failed to read output entry: {e}")))?
                .path();
            if p.is_dir() {
                let _ = fs::remove_dir_all(&p);
            } else {
                let _ = fs::remove_file(&p);
            }
        }
    }
    let manifest = descriptor::load_manifest()?;
    let mut written = 0u64;

    let shared = r#"---
name: wps-shared
version: 1.0.0
description: "wps CLI: Shared patterns for authentication, global flags, and output formatting."
metadata:
  openclaw:
    category: "productivity"
    requires:
      bins: ["wpscli"]
---

# wpscli — Shared Reference

## Authentication

```bash
# Configure AK/SK and OAuth fields
wpscli auth setup --ak <AK> --sk <SK>

# Step 1: print OAuth consent URL and open it in browser
wpscli auth login --user --print-url-only

# Step 2: exchange authorization code for user token
wpscli auth login --user --code <authorization_code>
```

## Global Flags

| Flag | Description |
|------|-------------|
| `--output <FORMAT>` | Output format: `json` (default), `compact`, `table` |
| `--dry-run` | Print request without sending API call |
| `--auth-type <app|user>` | Select app token or user token |
| `--retry <N>` | Retry count for network failures |

## CLI Syntax

```bash
wpscli <service> <endpoint> [flags]
wpscli raw <METHOD> <PATH|URL> [flags]
wpscli schema <service> [endpoint]
wpscli ui all
wpscli guide
```
"#;
    let shared_dir = out_dir.join("wps-shared");
    fs::create_dir_all(&shared_dir)
        .map_err(|e| WpsError::Execution(format!("failed to create wps-shared dir: {e}")))?;
    fs::write(shared_dir.join("SKILL.md"), shared)
        .map_err(|e| WpsError::Execution(format!("failed to write wps-shared SKILL.md: {e}")))?;
    written += 1;

    for service in manifest.services {
        let service_name = service
            .get("service")
            .and_then(|v| v.as_str())
            .ok_or_else(|| WpsError::Execution("invalid service record in manifest".to_string()))?;
        let desc = descriptor::load_service_descriptor(service_name)?;
        let skill_name = format!("wps-{}", slug(&desc.service));
        let skill_dir = out_dir.join(&skill_name);
        fs::create_dir_all(&skill_dir)
            .map_err(|e| WpsError::Execution(format!("failed to create {skill_name} dir: {e}")))?;

        let mut by_resource: BTreeMap<String, Vec<_>> = BTreeMap::new();
        for ep in desc.endpoints {
            let resource = resource_from_path(&ep.path, &desc.service);
            by_resource.entry(resource).or_default().push(ep);
        }

        let mut md = String::new();
        md.push_str(&format!(
            "---\nname: {skill_name}\nversion: 1.0.0\ndescription: \"WPS OpenAPI service: {}\"\nmetadata:\n  openclaw:\n    category: \"productivity\"\n    requires:\n      bins: [\"wpscli\"]\n    cliHelp: \"wpscli {} --help\"\n---\n\n",
            desc.service, desc.service
        ));
        md.push_str(&format!("# {} service\n\n", desc.service));
        md.push_str(
            "> **PREREQUISITE:** Read `../wps-shared/SKILL.md` for auth, global flags, and security rules.\n\n",
        );
        md.push_str(&format!("```bash\nwpscli {} <endpoint> [flags]\n```\n\n", desc.service));
        md.push_str("## API Resources\n\n");
        for (resource, mut eps) in by_resource {
            eps.sort_by(|a, b| a.id.cmp(&b.id));
            md.push_str(&format!("### {}\n\n", resource));
            for ep in eps {
                let scopes = if ep.scopes.is_empty() {
                    "-".to_string()
                } else {
                    ep.scopes.join(", ")
                };
                let summary = if ep.summary.is_empty() {
                    ep.name.clone()
                } else {
                    ep.summary.clone()
                };
                md.push_str(&format!(
                    "  - `{}` — {} (`{}` `{}`; scopes: `{}`)\n",
                    ep.id, summary, ep.http_method, ep.path, scopes
                ));
            }
            md.push('\n');
        }
        md.push_str("## Discovering Commands\n\n");
        md.push_str("```bash\n");
        md.push_str(&format!("wpscli {} --help\n", desc.service));
        md.push_str(&format!("wpscli schema {}\n", desc.service));
        md.push_str("```\n");

        fs::write(skill_dir.join("SKILL.md"), md)
            .map_err(|e| WpsError::Execution(format!("failed to write skill file: {e}")))?;
        written += 1;
    }

    Ok(serde_json::json!({
        "ok": true,
        "generated": written,
        "output_dir": out_dir
    }))
}
