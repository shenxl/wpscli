use std::collections::{BTreeMap, BTreeSet, HashMap};

use serde::Serialize;

use crate::capability_domains;
use crate::commands;
use crate::descriptor::{self, EndpointDescriptor, ServiceDescriptor};
use crate::error::WpsError;
use crate::executor::{self, ExecOptions};
use crate::help_schema_contract;

#[derive(Debug, Clone, Serialize)]
struct GateFailure {
    service: String,
    endpoint: String,
    reason: String,
}

#[derive(Debug, Clone, Serialize)]
struct GateSummary {
    name: String,
    status: String,
    total: usize,
    passed: usize,
    failed: usize,
    skipped: usize,
    failures: Vec<GateFailure>,
}

#[derive(Debug, Clone)]
struct EndpointRef {
    service: String,
    base_url: String,
    endpoint: EndpointDescriptor,
}

#[derive(Debug, Clone, Serialize)]
struct HelpSchemaConsistencySummary {
    total_endpoints: usize,
    consistent_endpoints: usize,
    inconsistent_endpoints: usize,
    failures: Vec<GateFailure>,
}

fn load_all_descriptors() -> Result<Vec<ServiceDescriptor>, WpsError> {
    let manifest = descriptor::load_manifest()?;
    let mut out = Vec::new();
    for record in manifest.services {
        let Some(service) = record.get("service").and_then(|v| v.as_str()) else {
            continue;
        };
        out.push(descriptor::load_service_descriptor(service)?);
    }
    Ok(out)
}

fn list_all_endpoints(descriptors: &[ServiceDescriptor]) -> Vec<EndpointRef> {
    let mut out = Vec::new();
    for desc in descriptors {
        for ep in &desc.endpoints {
            out.push(EndpointRef {
                service: desc.service.clone(),
                base_url: desc.base_url.clone(),
                endpoint: ep.clone(),
            });
        }
    }
    out
}

fn command_arg_ids(cmd: &clap::Command) -> BTreeSet<String> {
    cmd.get_arguments().map(|arg| arg.get_id().to_string()).collect()
}

fn expected_dynamic_flags(ep: &EndpointDescriptor) -> BTreeSet<String> {
    let mut expected = BTreeSet::from([
        "path-param".to_string(),
        "query".to_string(),
        "header".to_string(),
        "auth-type".to_string(),
        "user-token".to_string(),
        "dry-run".to_string(),
        "retry".to_string(),
        "paginate".to_string(),
    ]);
    if help_schema_contract::body_supported(&ep.http_method) {
        expected.insert("body".to_string());
    }
    expected
}

fn run_help_schema_consistency(descriptors: &[ServiceDescriptor]) -> HelpSchemaConsistencySummary {
    let mut total = 0usize;
    let mut failures = Vec::new();
    for desc in descriptors {
        let service_cmd = commands::build_service_command(desc);
        for ep in &desc.endpoints {
            total += 1;
            let endpoint_name = help_schema_contract::endpoint_command_name(&ep.id);
            let Some(ep_cmd) = service_cmd.get_subcommands().find(|s| s.get_name() == endpoint_name) else {
                failures.push(GateFailure {
                    service: desc.service.clone(),
                    endpoint: ep.id.clone(),
                    reason: "missing endpoint subcommand in dynamic -h".to_string(),
                });
                continue;
            };

            let args = command_arg_ids(ep_cmd);
            let expected = expected_dynamic_flags(ep);
            let missing = expected
                .difference(&args)
                .cloned()
                .collect::<Vec<_>>();
            if !missing.is_empty() {
                failures.push(GateFailure {
                    service: desc.service.clone(),
                    endpoint: ep.id.clone(),
                    reason: format!("dynamic -h missing flags: {}", missing.join(",")),
                });
                continue;
            }

            let contract = help_schema_contract::endpoint_contract(&desc.service, &desc.base_url, ep);
            if contract.http.path.trim().is_empty()
                || contract.http.method.trim().is_empty()
                || contract.command_template.trim().is_empty()
            {
                failures.push(GateFailure {
                    service: desc.service.clone(),
                    endpoint: ep.id.clone(),
                    reason: "schema contract missing required fields".to_string(),
                });
                continue;
            }

            if !contract
                .auth
                .supported_auth_types
                .iter()
                .any(|v| v == &contract.auth.recommended_auth_type)
            {
                failures.push(GateFailure {
                    service: desc.service.clone(),
                    endpoint: ep.id.clone(),
                    reason: format!(
                        "recommended auth `{}` not in supported auth types",
                        contract.auth.recommended_auth_type
                    ),
                });
                continue;
            }
        }
    }
    let inconsistent = failures.len();
    HelpSchemaConsistencySummary {
        total_endpoints: total,
        consistent_endpoints: total.saturating_sub(inconsistent),
        inconsistent_endpoints: inconsistent,
        failures,
    }
}

fn minimal_exec_options(ep: &EndpointDescriptor, auth_type: &str, dry_run: bool) -> ExecOptions {
    let mut path_params = HashMap::new();
    let mut query_params = HashMap::new();
    let mut headers = HashMap::new();

    for p in ep.params.path.iter().filter(|p| p.required) {
        path_params.insert(p.name.clone(), "x".to_string());
    }
    for p in ep.params.query.iter().filter(|p| p.required) {
        query_params.insert(p.name.clone(), "x".to_string());
    }
    for p in ep.params.header.iter().filter(|p| p.required) {
        headers.insert(p.name.clone(), "x".to_string());
    }

    let body = if help_schema_contract::body_supported(&ep.http_method) {
        Some("{}".to_string())
    } else {
        None
    };

    ExecOptions {
        path_params,
        query_params,
        headers,
        body,
        auth_type: auth_type.to_string(),
        dry_run,
        retry: 0,
        paginate: false,
    }
}

async fn run_dry_run_gate(endpoints: &[EndpointRef]) -> GateSummary {
    let mut failures = Vec::new();
    let total = endpoints.len();
    let mut passed = 0usize;
    for item in endpoints {
        let auth = help_schema_contract::recommended_auth_type(
            &help_schema_contract::supported_auth_types(&item.endpoint),
        );
        let opts = minimal_exec_options(&item.endpoint, &auth, true);
        match executor::execute_endpoint(&item.endpoint, opts).await {
            Ok(v) => {
                if v.get("dry_run").and_then(|x| x.as_bool()) == Some(true) {
                    passed += 1;
                } else {
                    failures.push(GateFailure {
                        service: item.service.clone(),
                        endpoint: item.endpoint.id.clone(),
                        reason: "dry-run response missing `dry_run=true`".to_string(),
                    });
                }
            }
            Err(e) => failures.push(GateFailure {
                service: item.service.clone(),
                endpoint: item.endpoint.id.clone(),
                reason: e.to_string(),
            }),
        }
    }
    let failed = total.saturating_sub(passed);
    GateSummary {
        name: "dry_run_gate".to_string(),
        status: if failed == 0 { "pass".to_string() } else { "warn".to_string() },
        total,
        passed,
        failed,
        skipped: 0,
        failures,
    }
}

fn resolve_connectivity_auth(endpoint: &EndpointDescriptor, mode: &str) -> Option<String> {
    let supported = help_schema_contract::supported_auth_types(endpoint);
    if mode == "auto" {
        return Some(help_schema_contract::recommended_auth_type(&supported));
    }
    let selected = mode.to_string();
    if supported.iter().any(|s| s == &selected) {
        Some(selected)
    } else {
        None
    }
}

async fn run_connectivity_gate(endpoints: &[EndpointRef], sample: usize, mode: &str) -> GateSummary {
    if sample == 0 {
        return GateSummary {
            name: "connectivity_gate".to_string(),
            status: "skipped".to_string(),
            total: 0,
            passed: 0,
            failed: 0,
            skipped: endpoints.len(),
            failures: vec![],
        };
    }

    let mut candidates = Vec::new();
    for item in endpoints {
        if !item.endpoint.http_method.eq_ignore_ascii_case("GET") {
            continue;
        }
        if item.endpoint.params.path.iter().any(|p| p.required)
            || item.endpoint.params.query.iter().any(|p| p.required)
            || item.endpoint.params.header.iter().any(|p| p.required)
        {
            continue;
        }
        if resolve_connectivity_auth(&item.endpoint, mode).is_none() {
            continue;
        }
        candidates.push(item.clone());
    }
    candidates.sort_by(|a, b| {
        a.service
            .cmp(&b.service)
            .then(a.endpoint.id.cmp(&b.endpoint.id))
            .then(a.base_url.cmp(&b.base_url))
    });

    let selected = candidates.into_iter().take(sample).collect::<Vec<_>>();
    let mut failures = Vec::new();
    let mut passed = 0usize;
    let total = selected.len();

    for item in selected {
        let Some(auth) = resolve_connectivity_auth(&item.endpoint, mode) else {
            continue;
        };
        let opts = minimal_exec_options(&item.endpoint, &auth, false);
        match executor::execute_endpoint(&item.endpoint, opts).await {
            Ok(v) => {
                if v.get("ok").and_then(|x| x.as_bool()) == Some(true) {
                    passed += 1;
                } else {
                    let status = v.get("status").and_then(|x| x.as_u64()).unwrap_or_default();
                    failures.push(GateFailure {
                        service: item.service.clone(),
                        endpoint: item.endpoint.id.clone(),
                        reason: format!("http status {}", status),
                    });
                }
            }
            Err(e) => failures.push(GateFailure {
                service: item.service.clone(),
                endpoint: item.endpoint.id.clone(),
                reason: e.to_string(),
            }),
        }
    }

    let failed = total.saturating_sub(passed);
    let status = if total == 0 {
        "skipped"
    } else if failed == 0 {
        "pass"
    } else {
        "warn"
    };
    GateSummary {
        name: "connectivity_gate".to_string(),
        status: status.to_string(),
        total,
        passed,
        failed,
        skipped: endpoints.len().saturating_sub(total),
        failures,
    }
}

fn top_failure_reasons(gates: &[&GateSummary], limit: usize) -> Vec<serde_json::Value> {
    let mut bucket = BTreeMap::<String, usize>::new();
    for gate in gates {
        for f in &gate.failures {
            *bucket.entry(f.reason.clone()).or_insert(0) += 1;
        }
    }
    bucket
        .into_iter()
        .map(|(reason, count)| serde_json::json!({ "reason": reason, "count": count }))
        .take(limit)
        .collect::<Vec<_>>()
}

fn score_report(
    descriptor_report: &descriptor::DescriptorAuditReport,
    consistency: &HelpSchemaConsistencySummary,
    dry_run: &GateSummary,
    connectivity: &GateSummary,
    domain_audit: &capability_domains::TopDomainAudit,
) -> i64 {
    let mut score = 100i64;
    score -= (descriptor_report.error_count as i64) * 8;
    score -= (descriptor_report.warning_count as i64) * 2;
    score -= (consistency.inconsistent_endpoints as i64) * 2;
    score -= (dry_run.failed as i64) * 2;
    score -= (connectivity.failed as i64) * 1;
    if !domain_audit.ok {
        score -= 15;
    }
    score.clamp(0, 100)
}

pub fn baseline_snapshot() -> Result<serde_json::Value, WpsError> {
    let descriptor_report = descriptor::audit_descriptors()?;
    let descriptors = load_all_descriptors()?;
    let consistency = run_help_schema_consistency(&descriptors);
    let domain_audit = capability_domains::audit_against_manifest()?;
    let red_lines = vec![
        serde_json::json!({
            "name": "descriptor_static_errors",
            "triggered": descriptor_report.error_count > 0,
            "message": "descriptor 结构存在阻断错误"
        }),
        serde_json::json!({
            "name": "help_schema_inconsistency",
            "triggered": consistency.inconsistent_endpoints > 0,
            "message": "-h 与 schema 合同不一致"
        }),
        serde_json::json!({
            "name": "top_domain_coverage",
            "triggered": !domain_audit.ok,
            "message": "顶层能力域映射未完整覆盖 manifest"
        }),
    ];
    Ok(serde_json::json!({
        "contract_version": help_schema_contract::HELP_SCHEMA_CONTRACT_VERSION,
        "descriptor_static": descriptor_report,
        "help_schema_consistency": consistency,
        "top_domains": domain_audit,
        "red_lines": red_lines
    }))
}

pub async fn run(connectivity_sample: usize, connectivity_auth: &str) -> Result<serde_json::Value, WpsError> {
    let descriptor_report = descriptor::audit_descriptors()?;
    let descriptors = load_all_descriptors()?;
    let endpoints = list_all_endpoints(&descriptors);
    let consistency = run_help_schema_consistency(&descriptors);
    let dry_run_gate = run_dry_run_gate(&endpoints).await;
    let connectivity_gate = run_connectivity_gate(&endpoints, connectivity_sample, connectivity_auth).await;
    let domain_audit = capability_domains::audit_against_manifest()?;

    let score = score_report(
        &descriptor_report,
        &consistency,
        &dry_run_gate,
        &connectivity_gate,
        &domain_audit,
    );
    let top_failures = top_failure_reasons(&[&dry_run_gate, &connectivity_gate], 10);

    Ok(serde_json::json!({
        "ok": descriptor_report.ok && consistency.inconsistent_endpoints == 0 && dry_run_gate.failed == 0,
        "quality_contract_version": help_schema_contract::HELP_SCHEMA_CONTRACT_VERSION,
        "score": score,
        "metrics": {
            "service_coverage": {
                "covered": descriptor_report.loaded_services,
                "declared": descriptor_report.declared_services,
            },
            "endpoint_coverage": {
                "total": descriptor_report.endpoint_count,
                "help_schema_consistent": consistency.consistent_endpoints,
                "dry_run_passed": dry_run_gate.passed,
                "dry_run_failed": dry_run_gate.failed,
            },
            "auth_recommendation": {
                "checked": consistency.total_endpoints,
                "failed": consistency.inconsistent_endpoints
            },
            "top_domains": domain_audit,
        },
        "gates": {
            "descriptor_static": descriptor_report,
            "help_schema_consistency": consistency,
            "dry_run": dry_run_gate,
            "connectivity": connectivity_gate,
        },
        "top_failures": top_failures
    }))
}
