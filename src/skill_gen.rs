use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::{fs, vec};

use clap::{Arg, Command};
use serde::Deserialize;

use crate::descriptor::{self, EndpointDescriptor};
use crate::error::WpsError;
use crate::{helpers, scope_catalog};

#[derive(Debug, Clone)]
struct HelperSpec {
    cli_name: &'static str,
    skill_slug: &'static str,
    recommended_auth: &'static str,
    description: &'static str,
}

#[derive(Debug, Clone, Deserialize, Default)]
struct RecipeRegistry {
    #[serde(default)]
    recipes: Vec<RecipeDef>,
}

#[derive(Debug, Clone, Deserialize)]
struct RecipeDef {
    name: String,
    title: String,
    description: String,
    #[serde(default)]
    category: String,
    #[serde(default)]
    services: Vec<String>,
    #[serde(default)]
    auth_sequence: Vec<String>,
    #[serde(default)]
    steps: Vec<String>,
    #[serde(default)]
    caution: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Default)]
struct PersonaRegistry {
    #[serde(default)]
    personas: Vec<PersonaDef>,
}

#[derive(Debug, Clone, Deserialize)]
struct PersonaDef {
    name: String,
    title: String,
    description: String,
    #[serde(default)]
    services: Vec<String>,
    #[serde(default)]
    workflows: Vec<String>,
    #[serde(default)]
    instructions: Vec<String>,
    #[serde(default)]
    tips: Vec<String>,
}

fn slug(s: &str) -> String {
    s.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '-'
            }
        })
        .collect::<String>()
        .trim_matches('-')
        .to_ascii_lowercase()
}

fn helper_specs() -> Vec<HelperSpec> {
    vec![
        HelperSpec {
            cli_name: "doc",
            skill_slug: "doc",
            recommended_auth: "user",
            description: "文档统一读写（分享链接解析、读取、写入、搜索）",
        },
        HelperSpec {
            cli_name: "files",
            skill_slug: "files",
            recommended_auth: "user",
            description: "应用目录与文件编排（创建、上传、下载、状态管理）",
        },
        HelperSpec {
            cli_name: "app-files",
            skill_slug: "app-files",
            recommended_auth: "user",
            description: "files 助手别名（兼容旧调用方式）",
        },
        HelperSpec {
            cli_name: "users",
            skill_slug: "users",
            recommended_auth: "app",
            description: "组织通讯录同步与查询（强制 app token）",
        },
        HelperSpec {
            cli_name: "dbsheet",
            skill_slug: "dbsheet",
            recommended_auth: "user",
            description: "多维表场景命令（schema/select/insert/update/delete）",
        },
        HelperSpec {
            cli_name: "dbt",
            skill_slug: "dbt",
            recommended_auth: "user",
            description: "多维表批量与结构化写入工具",
        },
        HelperSpec {
            cli_name: "chat",
            skill_slug: "chat",
            recommended_auth: "user",
            description: "会话与消息助手命令",
        },
        HelperSpec {
            cli_name: "calendar",
            skill_slug: "calendar",
            recommended_auth: "user",
            description: "日历查询、创建与忙闲分析",
        },
        HelperSpec {
            cli_name: "meeting",
            skill_slug: "meeting",
            recommended_auth: "user",
            description: "会议创建与参会人编排",
        },
        HelperSpec {
            cli_name: "airpage",
            skill_slug: "airpage",
            recommended_auth: "user",
            description: "智能文档块读取与写入",
        },
    ]
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
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

fn normalize_auth_types(auth_types: &[String]) -> Vec<String> {
    let mut out = BTreeSet::new();
    for t in auth_types {
        let n = t.trim().to_ascii_lowercase();
        if !n.is_empty() {
            out.insert(n);
        }
    }
    out.into_iter().collect()
}

fn endpoint_auth_types(ep: &EndpointDescriptor) -> Vec<String> {
    if ep.cookie_only {
        return vec!["cookie".to_string()];
    }
    let explicit = normalize_auth_types(&ep.auth_types);
    if !explicit.is_empty() {
        return explicit;
    }

    if ep.scopes.is_empty() {
        return vec!["app".to_string(), "user".to_string()];
    }

    let mut out = BTreeSet::new();
    let app_scopes = scope_catalog::filter_supported(&ep.scopes, scope_catalog::ScopeType::AppRole);
    let delegated_scopes = scope_catalog::filter_supported(&ep.scopes, scope_catalog::ScopeType::Delegated);
    if !app_scopes.is_empty() {
        out.insert("app".to_string());
    }
    if !delegated_scopes.is_empty() {
        out.insert("user".to_string());
    }

    if out.is_empty() {
        vec!["app".to_string(), "user".to_string()]
    } else {
        out.into_iter().collect()
    }
}

fn auth_label(auth_types: &[String]) -> String {
    let normalized = normalize_auth_types(auth_types);
    if normalized.len() == 1 {
        if normalized[0] == "cookie" {
            return "cookie-only".to_string();
        }
        return normalized[0].clone();
    }
    if normalized.len() == 2 && normalized.iter().any(|v| v == "app") && normalized.iter().any(|v| v == "user")
    {
        return "both".to_string();
    }
    normalized.join(",")
}

fn yaml_array(values: &[String]) -> String {
    if values.is_empty() {
        return "[]".to_string();
    }
    format!(
        "[{}]",
        values
            .iter()
            .map(|v| format!("\"{}\"", v.replace('"', "\\\"")))
            .collect::<Vec<_>>()
            .join(", ")
    )
}

fn shared_skill_markdown() -> String {
    let md = r#"---
name: wps-shared
version: 1.1.0
description: "wps CLI: shared auth model, global flags, scope preflight and security rules."
metadata:
  openclaw:
    category: "productivity"
    requires:
      bins: ["wpscli"]
    auth_types: ["app", "user", "cookie"]
---

# wpscli Shared Reference

## Authentication

### app auth (OpenAPI, preferred for org/application data)

```bash
wpscli auth setup --ak <AK> --sk <SK>
wpscli drives list-drives --auth-type app
```

### user auth (OpenAPI, preferred for user-owned files)

```bash
wpscli auth setup --ak <AK> --sk <SK>
wpscli auth login --user --print-url-only
wpscli auth login --user --code <authorization_code>
wpscli files search-files --auth-type user --query keyword=周报
```

### oauth daemon mode (recommended for long-running agents)

```bash
# start extracted daemon under tools/oauth-server
cd tools/oauth-server && ./start.sh --daemon

# point cli to oauth server
wpscli auth setup --oauth-server http://127.0.0.1:8089
wpscli auth status
```

### cookie auth (private V7, supplemental only)

Cookie mode is for private interfaces under `https://api.wps.cn` and is not production-stable.

```bash
# 1) full cookie
export WPS_CLI_COOKIE='wps_sid=...; csrf=...'

# 2) sid only
export WPS_CLI_WPS_SID='...'

# 3) cookie file (json or plain text)
export WPS_CLI_COOKIE_FILE=~/.cursor/skills/wpsv7-skills/.wps_sid_cache.json

# execute private endpoint
wpscli raw GET /v7/recent_chats --auth-type cookie
```

## Auth Selection Matrix

| Scenario | Recommended Auth | Why |
|----------|------------------|-----|
| Org directory / dept / member sync | `app` | Usually app-role scopes only |
| User-owned files and personal docs | `user` | Requires delegated user consent |
| Private V7 interfaces (IM search, recent chats) | `cookie` | Non-OpenAPI endpoints |
| Unknown endpoint with declared scopes | Follow endpoint `auth` tag in generated skill | Uses scope catalog matrix |

## Scope Preflight Recovery

When execution fails with scope/auth mismatch:

1. Run `wpscli auth status` to inspect token and auto-refresh readiness.
2. Switch auth type according to endpoint `auth` tag (`app`, `user`, `both`, `cookie-only`).
3. Re-login user token if delegated scope changed: `wpscli auth login --user`.
4. For cookie-only endpoints, refresh cookie source and retry with `--auth-type cookie`.

## Security Rules

1. Prefer `wpscli auth setup/login` encrypted local storage over long-lived env tokens.
2. Use `wpscli auth harden --apply` regularly.
3. Treat cookie credentials as high-risk ephemeral secrets; do not commit cookie files.
4. For destructive operations, use `--dry-run` first.
5. For bulk dbsheet writes, always use batch mode to avoid timeout and consistency issues.

## Global Flags

| Flag | Description |
|------|-------------|
| `--output <FORMAT>` | Output format: `json` (default), `compact`, `table` |
| `--dry-run` | Print request without sending API call |
| `--auth-type <app|user|cookie>` | Select auth mode |
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
    md.to_string()
}

fn to_text<T: ToString>(opt: Option<T>) -> String {
    opt.map(|v| v.to_string()).unwrap_or_default()
}

fn render_arg_flag(arg: &Arg) -> String {
    if let Some(long) = arg.get_long() {
        format!("--{long}")
    } else if let Some(short) = arg.get_short() {
        format!("-{short}")
    } else {
        arg.get_id().to_string()
    }
}

fn render_arg_rows(command: &Command) -> Vec<(String, String, String)> {
    let mut rows = Vec::new();
    for arg in command.get_arguments() {
        let flag = render_arg_flag(arg);
        let required = if arg.is_required_set() { "yes" } else { "no" }.to_string();
        let mut desc = to_text(arg.get_help()).trim().to_string();
        if desc.is_empty() {
            desc = to_text(arg.get_long_help()).trim().to_string();
        }
        if desc.is_empty() {
            desc = "-".to_string();
        }
        rows.push((flag, required, desc));
    }
    rows
}

fn markdown_escape(s: &str) -> String {
    s.replace('|', "\\|")
}

fn render_helper_skill(spec: &HelperSpec, cmd: &Command) -> String {
    let skill_name = format!("wps-helper-{}", slug(spec.skill_slug));
    let auth_types = if spec.recommended_auth == "app" {
        vec!["app".to_string()]
    } else if spec.recommended_auth == "cookie" {
        vec!["cookie".to_string()]
    } else {
        vec!["user".to_string(), "app".to_string()]
    };
    let mut md = String::new();
    md.push_str(&format!(
        "---\nname: {skill_name}\nversion: 1.0.0\ndescription: \"WPS helper command: {}\"\nmetadata:\n  openclaw:\n    category: \"productivity\"\n    requires:\n      bins: [\"wpscli\"]\n    cliHelp: \"wpscli {} --help\"\n    auth_types: {}\n---\n\n",
        spec.cli_name,
        spec.cli_name,
        yaml_array(&auth_types)
    ));
    md.push_str(&format!("# {} helper\n\n", spec.cli_name));
    md.push_str("> **PREREQUISITE:** Read `../wps-shared/SKILL.md` for auth and security rules.\n\n");
    md.push_str(&format!("- Recommended auth: `{}`\n", spec.recommended_auth));
    md.push_str(&format!("- Scope: {}\n\n", spec.description));
    md.push_str(&format!("```bash\nwpscli {} <command> [flags]\n```\n\n", spec.cli_name));

    let mut subs = cmd.get_subcommands().collect::<Vec<_>>();
    subs.sort_by(|a, b| a.get_name().cmp(b.get_name()));
    if subs.is_empty() {
        md.push_str("## Arguments\n\n");
        let rows = render_arg_rows(cmd);
        if rows.is_empty() {
            md.push_str("- No explicit arguments.\n\n");
        } else {
            md.push_str("| Arg | Required | Description |\n|-----|----------|-------------|\n");
            for (flag, required, desc) in rows {
                md.push_str(&format!(
                    "| `{}` | {} | {} |\n",
                    markdown_escape(&flag),
                    required,
                    markdown_escape(&desc)
                ));
            }
            md.push('\n');
        }
    } else {
        md.push_str("## Commands\n\n");
        for sub in subs {
            md.push_str(&format!("### {}\n\n", sub.get_name()));
            let about = to_text(sub.get_about()).trim().to_string();
            if !about.is_empty() {
                md.push_str(&format!("{about}\n\n"));
            }
            md.push_str(&format!("```bash\nwpscli {} {}\n```\n\n", spec.cli_name, sub.get_name()));
            let rows = render_arg_rows(sub);
            if !rows.is_empty() {
                md.push_str("| Arg | Required | Description |\n|-----|----------|-------------|\n");
                for (flag, required, desc) in rows {
                    md.push_str(&format!(
                        "| `{}` | {} | {} |\n",
                        markdown_escape(&flag),
                        required,
                        markdown_escape(&desc)
                    ));
                }
                md.push('\n');
            }
        }
    }

    let after_help = to_text(cmd.get_after_help()).trim().to_string();
    if !after_help.is_empty() {
        md.push_str("## Examples\n\n```bash\n");
        md.push_str(&after_help.replace("\\\n", "\n").replace("\\n", "\n"));
        md.push_str("\n```\n");
    } else {
        md.push_str("## Discovering Commands\n\n```bash\n");
        md.push_str(&format!("wpscli {} --help\n", spec.cli_name));
        md.push_str("```\n");
    }
    md
}

fn golden_case_template(spec: &HelperSpec) -> String {
    format!(
        "# GOLDEN CASES - {}\n\n## Case 1: happy path\n\n```bash\nwpscli {} --help\n```\n\n## Case 2: dry run validation\n\n```bash\nwpscli raw GET /v7/drives --auth-type {} --dry-run\n```\n\n## Case 3: auth fallback\n\n- Validate auth mode with `wpscli auth status`\n- Retry with `--auth-type {}` if scope/auth mismatch happens.\n",
        spec.cli_name, spec.cli_name, spec.recommended_auth, spec.recommended_auth
    )
}

fn maybe_copy_manual_golden_case(spec: &HelperSpec, target_file: &Path) -> Result<(), WpsError> {
    let source = match spec.skill_slug {
        "files" | "app-files" => repo_root().join("skills/wps-app-files/GOLDEN_CASES.md"),
        _ => PathBuf::new(),
    };
    if source.exists() {
        let content = fs::read_to_string(&source)
            .map_err(|e| WpsError::Execution(format!("failed to read {}: {e}", source.display())))?;
        fs::write(target_file, content)
            .map_err(|e| WpsError::Execution(format!("failed to write {}: {e}", target_file.display())))?;
        return Ok(());
    }
    fs::write(target_file, golden_case_template(spec))
        .map_err(|e| WpsError::Execution(format!("failed to write {}: {e}", target_file.display())))
}

fn load_recipes_registry() -> Result<RecipeRegistry, WpsError> {
    let path = repo_root().join("registry/recipes.yaml");
    let text = fs::read_to_string(&path)
        .map_err(|e| WpsError::Execution(format!("failed to read {}: {e}", path.display())))?;
    serde_yaml::from_str::<RecipeRegistry>(&text)
        .map_err(|e| WpsError::Execution(format!("failed to parse {}: {e}", path.display())))
}

fn load_personas_registry() -> Result<PersonaRegistry, WpsError> {
    let path = repo_root().join("registry/personas.yaml");
    let text = fs::read_to_string(&path)
        .map_err(|e| WpsError::Execution(format!("failed to read {}: {e}", path.display())))?;
    serde_yaml::from_str::<PersonaRegistry>(&text)
        .map_err(|e| WpsError::Execution(format!("failed to parse {}: {e}", path.display())))
}

fn write_skill_file(dir: &Path, markdown: &str) -> Result<(), WpsError> {
    fs::create_dir_all(dir).map_err(|e| WpsError::Execution(format!("failed to create {}: {e}", dir.display())))?;
    fs::write(dir.join("SKILL.md"), markdown)
        .map_err(|e| WpsError::Execution(format!("failed to write {}: {e}", dir.join("SKILL.md").display())))
}

fn write_index(
    out_dir: &Path,
    services: &[String],
    helpers_list: &[String],
    recipes: &[String],
    personas: &[String],
) -> Result<(), WpsError> {
    let mut md = String::new();
    md.push_str("# WPS Skills Index\n\n");
    md.push_str(&format!("- Layer 1 services: {}\n", services.len()));
    md.push_str(&format!("- Layer 2 helpers: {}\n", helpers_list.len()));
    md.push_str(&format!("- Layer 3 recipes: {}\n", recipes.len()));
    md.push_str(&format!("- Layer 4 personas: {}\n\n", personas.len()));

    md.push_str("## Layer 1 - Service Skills\n\n");
    for name in services {
        md.push_str(&format!("- `{}` -> `{}/SKILL.md`\n", name, name));
    }
    md.push_str("\n## Layer 2 - Helper Skills\n\n");
    for name in helpers_list {
        md.push_str(&format!("- `{}` -> `{}/SKILL.md`\n", name, name));
    }
    md.push_str("\n## Layer 3 - Recipe Skills\n\n");
    for name in recipes {
        md.push_str(&format!("- `{}` -> `{}/SKILL.md`\n", name, name));
    }
    md.push_str("\n## Layer 4 - Persona Skills\n\n");
    for name in personas {
        md.push_str(&format!("- `{}` -> `{}/SKILL.md`\n", name, name));
    }

    fs::write(out_dir.join("INDEX.md"), md)
        .map_err(|e| WpsError::Execution(format!("failed to write INDEX.md: {e}")))
}

pub fn generate(out_dir: &Path) -> Result<serde_json::Value, WpsError> {
    fs::create_dir_all(out_dir).map_err(|e| WpsError::Execution(format!("failed to create output directory: {e}")))?;
    if out_dir.exists() {
        for entry in fs::read_dir(out_dir).map_err(|e| WpsError::Execution(format!("failed to read output directory: {e}")))? {
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

    let mut generated = 0u64;
    let mut service_skills = Vec::<String>::new();
    let mut helper_skills = Vec::<String>::new();
    let mut recipe_skills = Vec::<String>::new();
    let mut persona_skills = Vec::<String>::new();

    let shared_dir = out_dir.join("wps-shared");
    write_skill_file(&shared_dir, &shared_skill_markdown())?;
    generated += 1;

    let manifest = descriptor::load_manifest()?;
    for service in manifest.services {
        let service_name = service
            .get("service")
            .and_then(|v| v.as_str())
            .ok_or_else(|| WpsError::Execution("invalid service record in manifest".to_string()))?;
        let desc = descriptor::load_service_descriptor(service_name)?;
        let skill_name = format!("wps-{}", slug(&desc.service));
        let skill_dir = out_dir.join(&skill_name);

        let mut by_resource: BTreeMap<String, Vec<_>> = BTreeMap::new();
        let mut service_auth = BTreeSet::new();
        for ep in &desc.endpoints {
            let resource = resource_from_path(&ep.path, &desc.service);
            let auth_types = endpoint_auth_types(ep);
            for at in &auth_types {
                service_auth.insert(at.clone());
            }
            by_resource.entry(resource).or_default().push(ep.clone());
        }

        let service_auth_list = service_auth.into_iter().collect::<Vec<_>>();
        let mut md = String::new();
        md.push_str(&format!(
            "---\nname: {skill_name}\nversion: 1.0.0\ndescription: \"WPS OpenAPI service: {}\"\nmetadata:\n  openclaw:\n    category: \"productivity\"\n    requires:\n      bins: [\"wpscli\"]\n    cliHelp: \"wpscli {} --help\"\n    auth_types: {}\n---\n\n",
            desc.service,
            desc.service,
            yaml_array(&service_auth_list)
        ));
        md.push_str(&format!("# {} service\n\n", desc.service));
        md.push_str("> **PREREQUISITE:** Read `../wps-shared/SKILL.md` for auth, global flags, and security rules.\n\n");
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
                let summary = if ep.summary.is_empty() { ep.name.clone() } else { ep.summary.clone() };
                let ep_auth = endpoint_auth_types(&ep);
                md.push_str(&format!(
                    "  - `{}` — {} (`{}` `{}`; scopes: `{}`; auth: `{}`)\n",
                    ep.id,
                    summary,
                    ep.http_method,
                    ep.path,
                    scopes,
                    auth_label(&ep_auth)
                ));
            }
            md.push('\n');
        }
        md.push_str("## Discovering Commands\n\n```bash\n");
        md.push_str(&format!("wpscli {} --help\n", desc.service));
        md.push_str(&format!("wpscli schema {}\n", desc.service));
        md.push_str("```\n");
        write_skill_file(&skill_dir, &md)?;
        generated += 1;
        service_skills.push(skill_name);
    }
    service_skills.sort();

    for spec in helper_specs() {
        let Some(cmd) = helpers::command(spec.cli_name) else {
            continue;
        };
        let helper_skill_name = format!("wps-helper-{}", slug(spec.skill_slug));
        let helper_dir = out_dir.join(&helper_skill_name);
        write_skill_file(&helper_dir, &render_helper_skill(&spec, &cmd))?;
        maybe_copy_manual_golden_case(&spec, &helper_dir.join("GOLDEN_CASES.md"))?;
        generated += 1;
        helper_skills.push(helper_skill_name);
    }
    helper_skills.sort();
    helper_skills.dedup();

    let recipes_registry = load_recipes_registry()?;
    for recipe in &recipes_registry.recipes {
        let skill_name = format!("recipe-{}", slug(&recipe.name));
        let mut md = String::new();
        md.push_str(&format!(
            "---\nname: {skill_name}\nversion: 1.0.0\ndescription: \"{}\"\nmetadata:\n  openclaw:\n    category: \"recipe\"\n    requires:\n      bins: [\"wpscli\"]\n      skills: [\"wps-shared\"]\n    domain: \"{}\"\n---\n\n",
            recipe.description.replace('"', "\\\""),
            if recipe.category.trim().is_empty() {
                "general"
            } else {
                recipe.category.trim()
            }
        ));
        md.push_str(&format!("# {}\n\n{}\n\n", recipe.title, recipe.description));
        if !recipe.services.is_empty() {
            md.push_str(&format!("- Services: `{}`\n", recipe.services.join("`, `")));
        }
        if !recipe.auth_sequence.is_empty() {
            md.push_str(&format!("- Auth sequence: `{}`\n", recipe.auth_sequence.join(" -> ")));
        }
        md.push('\n');
        md.push_str("## Steps\n\n");
        for (idx, step) in recipe.steps.iter().enumerate() {
            md.push_str(&format!("{}. {}\n", idx + 1, step));
        }
        if let Some(caution) = &recipe.caution {
            if !caution.trim().is_empty() {
                md.push_str(&format!("\n## Caution\n\n{}\n", caution.trim()));
            }
        }
        write_skill_file(&out_dir.join(&skill_name), &md)?;
        generated += 1;
        recipe_skills.push(skill_name);
    }
    recipe_skills.sort();

    let personas_registry = load_personas_registry()?;
    for persona in &personas_registry.personas {
        let skill_name = format!("persona-{}", slug(&persona.name));
        let mut md = String::new();
        md.push_str(&format!(
            "---\nname: {skill_name}\nversion: 1.0.0\ndescription: \"{}\"\nmetadata:\n  openclaw:\n    category: \"persona\"\n    requires:\n      bins: [\"wpscli\"]\n      skills: [\"wps-shared\"]\n---\n\n",
            persona.description.replace('"', "\\\"")
        ));
        md.push_str(&format!("# {}\n\n{}\n\n", persona.title, persona.description));
        if !persona.services.is_empty() {
            md.push_str(&format!("## Service Focus\n\n- `{}`\n\n", persona.services.join("`, `")));
        }
        if !persona.workflows.is_empty() {
            md.push_str("## Workflows\n\n");
            for wf in &persona.workflows {
                md.push_str(&format!("- `recipe-{}`\n", slug(wf)));
            }
            md.push('\n');
        }
        if !persona.instructions.is_empty() {
            md.push_str("## Instructions\n\n");
            for i in &persona.instructions {
                md.push_str(&format!("- {}\n", i));
            }
            md.push('\n');
        }
        if !persona.tips.is_empty() {
            md.push_str("## Tips\n\n");
            for t in &persona.tips {
                md.push_str(&format!("- {}\n", t));
            }
            md.push('\n');
        }
        write_skill_file(&out_dir.join(&skill_name), &md)?;
        generated += 1;
        persona_skills.push(skill_name);
    }
    persona_skills.sort();

    write_index(
        out_dir,
        &service_skills,
        &helper_skills,
        &recipe_skills,
        &persona_skills,
    )?;

    Ok(serde_json::json!({
        "ok": true,
        "generated": generated,
        "layers": {
            "service": service_skills.len(),
            "helper": helper_skills.len(),
            "recipe": recipe_skills.len(),
            "persona": persona_skills.len()
        },
        "output_dir": out_dir
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn helper_specs_are_resolvable() {
        for spec in helper_specs() {
            assert!(
                helpers::command(spec.cli_name).is_some(),
                "helper command missing: {}",
                spec.cli_name
            );
        }
    }

    #[test]
    fn recipes_reference_existing_services() {
        let manifest = descriptor::load_manifest().expect("manifest");
        let mut svc_set = BTreeSet::new();
        for service in manifest.services {
            if let Some(name) = service.get("service").and_then(|v| v.as_str()) {
                svc_set.insert(name.to_string());
            }
        }
        let registry = load_recipes_registry().expect("recipes registry");
        for recipe in registry.recipes {
            assert!(!recipe.name.trim().is_empty(), "recipe.name should not be empty");
            for svc in recipe.services {
                assert!(
                    svc_set.contains(&svc),
                    "recipe `{}` references missing service `{}`",
                    recipe.name,
                    svc
                );
            }
        }
    }

    #[test]
    fn personas_reference_existing_recipes() {
        let recipes = load_recipes_registry().expect("recipes registry");
        let recipe_set = recipes
            .recipes
            .into_iter()
            .map(|r| slug(&r.name))
            .collect::<BTreeSet<_>>();
        let personas = load_personas_registry().expect("personas registry");
        for persona in personas.personas {
            for wf in persona.workflows {
                assert!(
                    recipe_set.contains(&slug(&wf)),
                    "persona `{}` references missing workflow `{}`",
                    persona.name,
                    wf
                );
            }
        }
    }

    #[test]
    fn cookie_route_coverage_is_defined() {
        let calendars = descriptor::load_service_descriptor("calendars").expect("calendars");
        assert!(calendars.endpoints.iter().any(|e| e.path.starts_with("/v7/calendars")));

        let meetings = descriptor::load_service_descriptor("meetings").expect("meetings");
        assert!(meetings.endpoints.iter().any(|e| e.path.starts_with("/v7/meetings")));

        let drives = descriptor::load_service_descriptor("drives").expect("drives");
        assert!(drives.endpoints.iter().any(|e| e.path.starts_with("/v7/drives")));

        let free_busy = descriptor::load_service_descriptor("free_busy_list").expect("free_busy_list");
        assert!(free_busy.endpoints.iter().any(|e| e.path == "/v7/free_busy_list"));

        let users = descriptor::load_service_descriptor("users").expect("users");
        assert!(users.endpoints.iter().any(|e| e.path == "/v7/users/current"));

        let cookie_im = descriptor::load_service_descriptor("cookie_im").expect("cookie_im");
        let cookie_im_paths = cookie_im.endpoints.iter().map(|e| e.path.as_str()).collect::<BTreeSet<_>>();
        for expected in [
            "/v7/recent_chats",
            "/v7/chats/search",
            "/v7/messages/search",
            "/v7/chats/{chat_id}/messages/send_cloud_file",
        ] {
            assert!(cookie_im_paths.contains(expected), "missing cookie path: {}", expected);
        }

        let cookie_contacts = descriptor::load_service_descriptor("cookie_contacts").expect("cookie_contacts");
        assert!(cookie_contacts.endpoints.iter().any(|e| e.path == "/v7/users/search"));
    }
}
