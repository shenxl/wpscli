mod auth;
mod auth_commands;
mod commands;
mod descriptor;
mod doctor;
mod error;
mod executor;
mod formatter;
mod helpers;
mod link_resolver;
mod schema;
mod secure_store;
mod scope_catalog;
mod skill_runtime;
mod skill_gen;
mod services;
mod ui;
mod validate;

use std::collections::HashMap;
use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

use clap::error::ErrorKind;
use clap::ArgMatches;
use error::{print_error_json, WpsError};
use formatter::{print_value, OutputFormat};

#[tokio::main]
async fn main() {
    let _ = dotenvy::dotenv();
    if let Err(err) = run().await {
        print_error_json(&err);
        std::process::exit(1);
    }
}

async fn run() -> Result<(), WpsError> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 || args[1] == "--help" || args[1] == "-h" || args[1] == "help" {
        let mut root = commands::build_root()
            .subcommand(auth_commands::build_auth_command())
            .subcommand(commands::build_schema_command())
            .subcommand(commands::build_catalog_command())
            .subcommand(commands::build_raw_command())
            .subcommand(commands::build_generate_skills_command())
            .subcommand(commands::build_completions_command())
            .subcommand(commands::build_ui_command())
            .subcommand(commands::build_doctor_command());
        root.print_long_help()
            .map_err(|e| WpsError::Validation(format!("failed to print help: {e}")))?;
        println!();
        return Ok(());
    }

    let first = args[1].as_str();
    if first == "auth" {
        return auth_commands::handle(&args[2..]).await;
    }

    if first == "schema" {
        let root = commands::build_root().subcommand(commands::build_schema_command());
        let Some(m) = parse_matches_or_print(root, args.clone())? else {
            return Ok(());
        };
        let format = OutputFormat::parse(m.get_one::<String>("output"));
        let s = m
            .subcommand_matches("schema")
            .ok_or_else(|| WpsError::Validation("missing schema subcommand".to_string()))?;
        let service = s
            .get_one::<String>("service")
            .ok_or_else(|| WpsError::Validation("missing schema service".to_string()))?;
        let endpoint = s.get_one::<String>("endpoint").map(|s| s.as_str());
        let mode = s.get_one::<String>("mode").map(|v| v.as_str()).unwrap_or("raw");
        let emit_template = s.get_one::<String>("emit-template").map(|v| v.as_str());
        let value = schema::run(service, endpoint, mode, emit_template)?;
        print_value(&value, format);
        return Ok(());
    }
    if first == "catalog" {
        let root = commands::build_root().subcommand(commands::build_catalog_command());
        let Some(m) = parse_matches_or_print(root, args.clone())? else {
            return Ok(());
        };
        let format = OutputFormat::parse(m.get_one::<String>("output"));
        let cm = m
            .subcommand_matches("catalog")
            .ok_or_else(|| WpsError::Validation("missing catalog subcommand".to_string()))?;
        let service = cm.get_one::<String>("service").map(|s| s.as_str());
        let mode = cm
            .get_one::<String>("mode")
            .map(|s| s.as_str())
            .unwrap_or("show");
        let value = run_catalog(service, mode)?;
        print_value(&value, format);
        return Ok(());
    }

    if first == "raw" {
        return run_raw(args).await;
    }
    if first == "generate-skills" {
        let root = commands::build_root().subcommand(commands::build_generate_skills_command());
        let Some(m) = parse_matches_or_print(root, args.clone())? else {
            return Ok(());
        };
        let format = OutputFormat::parse(m.get_one::<String>("output"));
        let gm = m
            .subcommand_matches("generate-skills")
            .ok_or_else(|| WpsError::Validation("missing generate-skills subcommand".to_string()))?;
        let out_dir = gm
            .get_one::<String>("out-dir")
            .ok_or_else(|| WpsError::Validation("missing out-dir".to_string()))?;
        let value = skill_gen::generate(std::path::Path::new(out_dir))?;
        print_value(&value, format);
        return Ok(());
    }
    if first == "completions" {
        let root = commands::build_root().subcommand(commands::build_completions_command());
        let Some(m) = parse_matches_or_print(root, args.clone())? else {
            return Ok(());
        };
        let cm = m
            .subcommand_matches("completions")
            .ok_or_else(|| WpsError::Validation("missing completions subcommand".to_string()))?;
        let shell = cm
            .get_one::<String>("shell")
            .ok_or_else(|| WpsError::Validation("missing shell".to_string()))?;
        print_completion(shell)?;
        return Ok(());
    }
    if first == "ui" || first == "guide" {
        let root = commands::build_root().subcommand(commands::build_ui_command());
        let cmd_args = if first == "guide" {
            let mut v = args.clone();
            v[1] = "ui".to_string();
            if v.len() == 2 {
                v.push("all".to_string());
            }
            v
        } else {
            args.clone()
        };
        let Some(m) = parse_matches_or_print(root, cmd_args)? else {
            return Ok(());
        };
        let um = m
            .subcommand_matches("ui")
            .ok_or_else(|| WpsError::Validation("missing ui subcommand".to_string()))?;
        let scene = um
            .get_one::<String>("scene")
            .ok_or_else(|| WpsError::Validation("missing scene".to_string()))?;
        ui::show(scene)?;
        return Ok(());
    }
    if first == "doctor" {
        let root = commands::build_root().subcommand(commands::build_doctor_command());
        let Some(m) = parse_matches_or_print(root, args.clone())? else {
            return Ok(());
        };
        let format = OutputFormat::parse(m.get_one::<String>("output"));
        let value = doctor::run();
        print_value(&value, format);
        return Ok(());
    }
    if let Some(cmd) = helpers::command(first) {
        if helper_help_requested(&args[2..]) {
            return print_helper_help(cmd, first, &args[2..]);
        }
    }

    if let Some(result) = helpers::dispatch(first, &args[2..]).await {
        let value = result?;
        println!(
            "{}",
            serde_json::to_string_pretty(&value).unwrap_or_else(|_| "{}".to_string())
        );
        return Ok(());
    }

    run_dynamic_service(args).await
}

async fn run_raw(args: Vec<String>) -> Result<(), WpsError> {
    let root = commands::build_root().subcommand(commands::build_raw_command());
    let Some(m) = parse_matches_or_print(root, args)? else {
        return Ok(());
    };
    let format = OutputFormat::parse(m.get_one::<String>("output"));
    let rm = m
        .subcommand_matches("raw")
        .ok_or_else(|| WpsError::Validation("missing raw subcommand".to_string()))?;
    let method = rm.get_one::<String>("method").expect("required");
    let path = rm.get_one::<String>("path").expect("required");
    let query = rm
        .get_many::<String>("query")
        .map(|v| v.cloned().collect::<Vec<_>>())
        .unwrap_or_default();
    let headers = rm
        .get_many::<String>("header")
        .map(|v| v.cloned().collect::<Vec<_>>())
        .unwrap_or_default();
    let body = rm.get_one::<String>("body").cloned();
    let auth_type = rm.get_one::<String>("auth-type").expect("default");
    let auth_type = if rm.get_flag("user-token") {
        "user".to_string()
    } else {
        auth_type.clone()
    };
    let dry_run = rm.get_flag("dry-run");
    let retry = *rm.get_one::<u32>("retry").expect("default");
    let qv = validate::parse_key_value_pairs(&query)?
        .into_iter()
        .collect::<HashMap<_, _>>();
    let hv = validate::parse_key_value_pairs(&headers)?
        .into_iter()
        .collect::<HashMap<_, _>>();
    let value = executor::execute_raw(method, path, qv, hv, body, &auth_type, dry_run, retry).await?;
    print_value(&value, format);
    Ok(())
}

async fn run_dynamic_service(args: Vec<String>) -> Result<(), WpsError> {
    let first = args[1].clone();
    let service = services::resolve_service(&first)?;
    let desc = descriptor::load_service_descriptor(&service)?;
    let root = commands::build_root().subcommand(commands::build_service_command(&desc));
    let Some(m) = parse_matches_or_print(root, args.clone())? else {
        return Ok(());
    };
    let format = OutputFormat::parse(m.get_one::<String>("output"));
    let sm = m
        .subcommand_matches(&service)
        .ok_or_else(|| WpsError::Validation("service subcommand not matched".to_string()))?;
    let (endpoint_cmd, endpoint_m) = endpoint_subcommand(sm)?;
    let endpoint = resolve_endpoint(&desc, &endpoint_cmd)
        .ok_or_else(|| WpsError::Validation(format!("endpoint not found: {}", endpoint_cmd)))?;

    let path_params = endpoint_m
        .get_many::<String>("path-param")
        .map(|v| v.cloned().collect::<Vec<_>>())
        .unwrap_or_default();
    let query_params = endpoint_m
        .get_many::<String>("query")
        .map(|v| v.cloned().collect::<Vec<_>>())
        .unwrap_or_default();
    let headers = endpoint_m
        .get_many::<String>("header")
        .map(|v| v.cloned().collect::<Vec<_>>())
        .unwrap_or_default();
    let opts = executor::ExecOptions {
        path_params: validate::parse_key_value_pairs(&path_params)?
            .into_iter()
            .collect::<HashMap<_, _>>(),
        query_params: validate::parse_key_value_pairs(&query_params)?
            .into_iter()
            .collect::<HashMap<_, _>>(),
        headers: validate::parse_key_value_pairs(&headers)?
            .into_iter()
            .collect::<HashMap<_, _>>(),
        body: endpoint_m.get_one::<String>("body").cloned(),
        auth_type: endpoint_m
            .get_one::<String>("auth-type")
            .cloned()
            .map(|v| if endpoint_m.get_flag("user-token") { "user".to_string() } else { v })
            .unwrap_or_else(|| {
                if endpoint_m.get_flag("user-token") {
                    "user".to_string()
                } else {
                    "app".to_string()
                }
            }),
        dry_run: endpoint_m.get_flag("dry-run"),
        retry: *endpoint_m.get_one::<u32>("retry").unwrap_or(&1),
        paginate: endpoint_m.get_flag("paginate"),
    };
    let value = executor::execute_endpoint(endpoint, opts).await?;
    print_value(&value, format);
    Ok(())
}

fn resolve_endpoint<'a>(
    desc: &'a descriptor::ServiceDescriptor,
    endpoint_cmd: &str,
) -> Option<&'a descriptor::EndpointDescriptor> {
    let cmd_hyphen = endpoint_cmd.replace('_', "-");
    let cmd_underscore = endpoint_cmd.replace('-', "_");
    if let Some(found) = desc.endpoints.iter().find(|ep| {
        let id_hyphen = ep.id.replace('_', "-");
        let id_underscore = ep.id.replace('-', "_");
        ep.id == endpoint_cmd
            || ep.id == cmd_hyphen
            || ep.id == cmd_underscore
            || id_hyphen == cmd_hyphen
            || id_underscore == cmd_underscore
    }) {
        return Some(found);
    }
    desc.endpoints.iter().find(|ep| {
        let id_hyphen = ep.id.replace('_', "-");
        endpoint_aliases(&id_hyphen).iter().any(|a| a == endpoint_cmd)
    })
}

fn endpoint_aliases(endpoint_name: &str) -> Vec<String> {
    let mut aliases = Vec::new();
    if endpoint_name.starts_with("get-") && endpoint_name.ends_with("-list") {
        let inner = endpoint_name
            .trim_start_matches("get-")
            .trim_end_matches("-list");
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

fn endpoint_subcommand(matches: &ArgMatches) -> Result<(String, &ArgMatches), WpsError> {
    matches
        .subcommand()
        .map(|(name, m)| (name.to_string(), m))
        .ok_or_else(|| WpsError::Validation("missing endpoint subcommand".to_string()))
}

fn parse_matches_or_print(
    mut cmd: clap::Command,
    args: Vec<String>,
) -> Result<Option<ArgMatches>, WpsError> {
    match cmd.try_get_matches_from_mut(args) {
        Ok(m) => Ok(Some(m)),
        Err(e) => match e.kind() {
            ErrorKind::DisplayHelp | ErrorKind::DisplayVersion => {
                e.print()
                    .map_err(|pe| WpsError::Validation(format!("failed to print help: {pe}")))?;
                Ok(None)
            }
            _ => Err(WpsError::Validation(e.to_string())),
        },
    }
}

fn run_catalog(service: Option<&str>, mode: &str) -> Result<serde_json::Value, WpsError> {
    if let Some(svc) = service {
        let desc = descriptor::load_service_descriptor(&services::resolve_service(svc)?)?;
        return Ok(serde_json::json!({
            "ok": true,
            "service": desc.service,
            "endpoint_count": desc.endpoints.len(),
            "endpoints": desc.endpoints.iter().map(|e| serde_json::json!({
                "id": e.id,
                "summary": e.summary,
                "http_method": e.http_method,
                "path": e.path
            })).collect::<Vec<_>>()
        }));
    }
    if mode == "service" {
        let manifest = descriptor::load_manifest()?;
        let mut services = Vec::new();
        for entry in manifest.services {
            if let Some(name) = entry.get("service").and_then(|v| v.as_str()) {
                services.push(name.to_string());
            }
        }
        services.sort();
        return Ok(serde_json::json!({
            "ok": true,
            "catalog_mode": "service",
            "total_services": services.len(),
            "services": services
        }));
    }

    let categorized = build_show_grouped_catalog()?;
    Ok(serde_json::json!({
        "ok": true,
        "catalog_mode": "show",
        "groups": categorized
    }))
}

fn helper_help_requested(args: &[String]) -> bool {
    args.iter()
        .any(|a| matches!(a.as_str(), "-h" | "--help" | "help"))
}

fn print_helper_help(mut cmd: clap::Command, helper_name: &str, helper_args: &[String]) -> Result<(), WpsError> {
    let mut argv = vec![helper_name.to_string()];
    argv.extend(helper_args.iter().cloned());
    match cmd.try_get_matches_from_mut(argv) {
        Ok(_) => Ok(()),
        Err(e) => match e.kind() {
            ErrorKind::DisplayHelp | ErrorKind::DisplayVersion => {
                e.print()
                    .map_err(|pe| WpsError::Validation(format!("failed to print helper help: {pe}")))?;
                Ok(())
            }
            _ => Err(WpsError::Validation(e.to_string())),
        },
    }
}

fn show_json_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../wpsskill/wps-openapi-nav/show/show.json")
}

fn api_index_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../wpsskill/wps-openapi-nav/data/api_index.json")
}

fn build_show_grouped_catalog() -> Result<Vec<serde_json::Value>, WpsError> {
    let show_raw = std::fs::read_to_string(show_json_path())
        .map_err(|e| WpsError::Descriptor(format!("failed to read show.json: {e}")))?;
    let show_v: serde_json::Value = serde_json::from_str(&show_raw)
        .map_err(|e| WpsError::Descriptor(format!("failed to parse show.json: {e}")))?;
    let mut breadcrumb_by_doc_id: HashMap<u64, Vec<String>> = HashMap::new();
    collect_show_doc_breadcrumbs(&show_v, &mut breadcrumb_by_doc_id);

    let api_raw = std::fs::read_to_string(api_index_path())
        .map_err(|e| WpsError::Descriptor(format!("failed to read api_index.json: {e}")))?;
    let api_v: serde_json::Value = serde_json::from_str(&api_raw)
        .map_err(|e| WpsError::Descriptor(format!("failed to parse api_index.json: {e}")))?;
    let raw_doc_categories = load_raw_doc_categories()?;
    let endpoints = api_v
        .get("endpoints")
        .and_then(|v| v.as_object())
        .ok_or_else(|| WpsError::Descriptor("api_index.endpoints missing".to_string()))?;

    let mut grouped: BTreeMap<String, BTreeMap<String, (u64, BTreeSet<String>)>> = BTreeMap::new();
    for (_ep_id, ep) in endpoints {
        let doc_id = ep
            .get("doc_id")
            .and_then(|v| v.as_u64().or_else(|| v.as_str().and_then(|s| s.parse::<u64>().ok())));
        let path = ep.get("path").and_then(|v| v.as_str()).unwrap_or("");
        let service = service_from_path(path);

        let nav_category = resolve_nav_category(ep, doc_id, &raw_doc_categories, &breadcrumb_by_doc_id);
        let top = nav_category
            .first()
            .cloned()
            .unwrap_or_else(|| "未归类".to_string());
        let sub = if nav_category.len() > 1 {
            nav_category[1].clone()
        } else {
            "未归类".to_string()
        };
        let entry = grouped
            .entry(top)
            .or_default()
            .entry(sub)
            .or_insert_with(|| (0, BTreeSet::new()));
        entry.0 += 1;
        entry.1.insert(service);
    }

    let mut out = Vec::new();
    for (top, cats) in grouped {
        let mut category_values = Vec::new();
        for (name, (endpoint_count, services)) in cats {
            category_values.push(serde_json::json!({
                "name": name,
                "endpoint_count": endpoint_count,
                "service_count": services.len(),
                "services": services.into_iter().collect::<Vec<_>>(),
            }));
        }
        out.push(serde_json::json!({
            "group": top,
            "categories": category_values
        }));
    }
    Ok(out)
}

fn resolve_nav_category(
    endpoint_obj: &serde_json::Value,
    doc_id: Option<u64>,
    raw_doc_categories: &HashMap<u64, Vec<String>>,
    breadcrumb_by_doc_id: &HashMap<u64, Vec<String>>,
) -> Vec<String> {
    if let Some(id) = doc_id {
        if let Some(raw) = raw_doc_categories.get(&id) {
            if !raw.is_empty() {
                return raw.clone();
            }
        }
    }

    if let Some(cat_arr) = endpoint_obj.get("category").and_then(|v| v.as_array()) {
        let mut cats = cat_arr
            .iter()
            .filter_map(|x| x.as_str().map(|s| s.to_string()))
            .collect::<Vec<_>>();
        cats.retain(|s| !s.trim().is_empty() && s != "文件夹");
        if !cats.is_empty() {
            return cats;
        }
    }

    if let Some(id) = doc_id {
        if let Some(crumb) = breadcrumb_by_doc_id.get(&id) {
            let mut cats = crumb.clone();
            cats.retain(|s| !s.trim().is_empty() && s != "文件夹");
            if !cats.is_empty() {
                return cats;
            }
        }
    }
    vec!["未归类".to_string()]
}

fn collect_show_doc_breadcrumbs(v: &serde_json::Value, map: &mut HashMap<u64, Vec<String>>) {
    if let Some(doc_info) = v.get("docInfo") {
        if let Some(id_str) = doc_info.get("id").and_then(|x| x.as_str()) {
            if let Ok(doc_id) = id_str.parse::<u64>() {
                let breadcrumbs = doc_info
                    .get("breadcrumb")
                    .and_then(|b| b.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|x| x.as_str().map(|s| s.to_string()))
                            .collect::<Vec<_>>()
                    })
                    .unwrap_or_default();
                if !breadcrumbs.is_empty() {
                    map.insert(doc_id, breadcrumbs);
                }
            }
        }
    }
    if let Some(obj) = v.as_object() {
        for (_k, child) in obj {
            collect_show_doc_breadcrumbs(child, map);
        }
    } else if let Some(arr) = v.as_array() {
        for child in arr {
            collect_show_doc_breadcrumbs(child, map);
        }
    }
}

fn service_from_path(raw_path: &str) -> String {
    let mut path = raw_path.trim().trim_end_matches('*').to_string();
    if path.starts_with("https://openapi.wps.cn") {
        path = path.trim_start_matches("https://openapi.wps.cn").to_string();
    }
    if path.starts_with("/oauth2/") {
        return "oauth2".to_string();
    }
    if !path.starts_with("/v7/") {
        return "misc".to_string();
    }
    let rest = &path["/v7/".len()..];
    if rest.is_empty() {
        return "misc".to_string();
    }
    let first = rest.split('/').next().unwrap_or("misc");
    if first == "coop" {
        let mut parts = rest.split('/');
        let _ = parts.next();
        if let Some(second) = parts.next() {
            return format!("coop_{second}");
        }
        return "coop".to_string();
    }
    first.to_string()
}

fn raw_root_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../wpsskill/wps-openapi-nav/data/raw")
}

fn load_raw_doc_categories() -> Result<HashMap<u64, Vec<String>>, WpsError> {
    let root = raw_root_path();
    let mut map = HashMap::new();
    collect_raw_doc_categories(&root, &root, &mut map)?;
    Ok(map)
}

fn collect_raw_doc_categories(
    root: &PathBuf,
    dir: &PathBuf,
    out: &mut HashMap<u64, Vec<String>>,
) -> Result<(), WpsError> {
    let entries = std::fs::read_dir(dir)
        .map_err(|e| WpsError::Descriptor(format!("failed to read raw dir {}: {e}", dir.display())))?;
    for entry in entries {
        let entry = entry
            .map_err(|e| WpsError::Descriptor(format!("failed to read raw dir entry: {e}")))?;
        let path = entry.path();
        if path.is_dir() {
            let pbuf = path.clone();
            collect_raw_doc_categories(root, &pbuf, out)?;
            continue;
        }
        if path.extension().and_then(|s| s.to_str()) != Some("json") {
            continue;
        }
        let raw = match std::fs::read_to_string(&path) {
            Ok(v) => v,
            Err(_) => continue,
        };
        let value = match serde_json::from_str::<serde_json::Value>(&raw) {
            Ok(v) => v,
            Err(_) => continue,
        };
        let doc_id = value
            .get("data")
            .and_then(|d| d.get("meta"))
            .and_then(|m| m.get("doc_id"))
            .and_then(|v| v.as_str().and_then(|s| s.parse::<u64>().ok()).or_else(|| v.as_u64()));
        let Some(doc_id) = doc_id else {
            continue;
        };
        let rel = match path.strip_prefix(root) {
            Ok(r) => r,
            Err(_) => continue,
        };
        let mut comps = rel
            .components()
            .filter_map(|c| c.as_os_str().to_str().map(|s| s.to_string()))
            .collect::<Vec<_>>();
        if !comps.is_empty() {
            let _ = comps.pop();
        }
        if !comps.is_empty() {
            out.insert(doc_id, comps);
        }
    }
    Ok(())
}

fn print_completion(shell: &str) -> Result<(), WpsError> {
    use clap_complete::{generate, shells};
    let mut cmd = commands::build_root();
    let mut out = std::io::stdout();
    match shell {
        "bash" => generate(shells::Bash, &mut cmd, "wpscli", &mut out),
        "zsh" => generate(shells::Zsh, &mut cmd, "wpscli", &mut out),
        "fish" => generate(shells::Fish, &mut cmd, "wpscli", &mut out),
        "powershell" => generate(shells::PowerShell, &mut cmd, "wpscli", &mut out),
        "elvish" => generate(shells::Elvish, &mut cmd, "wpscli", &mut out),
        _ => return Err(WpsError::Validation(format!("unsupported shell: {shell}"))),
    }
    Ok(())
}
