use clap::{error::ErrorKind, Arg, ArgAction, ArgMatches, Command};

use crate::auth::{self, Credentials};
use crate::error::WpsError;

pub fn build_auth_command() -> Command {
    Command::new("auth")
        .about("管理 WPS 授权（支持引导模式）")
        .after_help(
            "示例：\n  \
             wpscli auth guide\n  \
             wpscli auth setup --ak <AK> --sk <SK>\n  \
             wpscli auth login --user --mode local\n  \
             wpscli auth status",
        )
        .subcommand(Command::new("guide").about("中文引导式授权（推荐新手）"))
        .subcommand(
            Command::new("setup")
                .about("保存 AK/SK 与 OAuth 相关配置")
                .after_help(
                    "示例：\n  \
                     wpscli auth setup --ak <AK> --sk <SK>\n  \
                     wpscli auth setup --redirect-uri http://localhost:53682/callback --scope kso.user_base.read",
                )
                .arg(Arg::new("ak").long("ak").num_args(1).help("应用 AK（client_id）"))
                .arg(Arg::new("sk").long("sk").num_args(1).help("应用 SK（client_secret）"))
                .arg(Arg::new("redirect-uri").long("redirect-uri").num_args(1).help("OAuth 回调地址"))
                .arg(Arg::new("scope").long("scope").num_args(1).help("OAuth scope，多个 scope 用空格分隔"))
                .arg(Arg::new("oauth-server").long("oauth-server").num_args(1).help("可选：外部 OAuth 代理服务地址"))
                .arg(Arg::new("oauth-device-endpoint").long("oauth-device-endpoint").num_args(1).help("可选：设备码授权端点"))
                .arg(Arg::new("oauth-token-endpoint").long("oauth-token-endpoint").num_args(1).help("可选：Token 交换端点")),
        )
        .subcommand(
            Command::new("login")
                .about("获取并保存用户 token，或手动写入 token")
                .after_help(
                    "示例：\n  \
                     wpscli auth login --user --mode local\n  \
                     wpscli auth login --user --mode remote\n  \
                     wpscli auth login --user --mode remote-device\n  \
                     wpscli auth login --user-token <ACCESS_TOKEN> --refresh-token <REFRESH_TOKEN>",
                )
                .arg(Arg::new("user").long("user").action(ArgAction::SetTrue).help("启用 OAuth 用户登录流程"))
                .arg(Arg::new("ak").long("ak").num_args(1).help("覆盖使用的 AK"))
                .arg(Arg::new("sk").long("sk").num_args(1).help("覆盖使用的 SK"))
                .arg(Arg::new("oauth-server").long("oauth-server").num_args(1).help("可选：OAuth 代理服务地址"))
                .arg(Arg::new("code").long("code").num_args(1).help("授权码（适用于手动流程）"))
                .arg(Arg::new("state").long("state").num_args(1).default_value("wpscli"))
                .arg(
                    Arg::new("mode")
                        .long("mode")
                        .num_args(1)
                        .default_value("local")
                        .value_parser(["local", "remote", "remote-device"])
                        .help("登录模式：local=本地回调，remote=粘贴回调URL，remote-device=设备码轮询"),
                )
                .arg(Arg::new("scope").long("scope").num_args(1).help("本次登录使用的 scope"))
                .arg(Arg::new("redirect-uri").long("redirect-uri").num_args(1).help("本次登录使用的回调地址"))
                .arg(
                    Arg::new("callback-url")
                        .long("callback-url")
                        .num_args(1)
                        .help("远程模式下粘贴完整回调 URL"),
                )
                .arg(Arg::new("print-url-only").long("print-url-only").action(ArgAction::SetTrue).help("仅打印授权链接，不继续交换 token"))
                .arg(
                    Arg::new("no-open")
                        .long("no-open")
                        .help("不自动打开浏览器")
                        .action(ArgAction::SetTrue),
                )
                .arg(
                    Arg::new("no-local-server")
                        .long("no-local-server")
                        .help("关闭本地回调监听，改用 --code 手动输入")
                        .action(ArgAction::SetTrue),
                )
                .arg(
                    Arg::new("timeout-seconds")
                        .long("timeout-seconds")
                        .default_value("180")
                        .value_parser(clap::value_parser!(u64))
                        .help("等待本地回调超时（秒）"),
                )
                .arg(
                    Arg::new("poll-interval-seconds")
                        .long("poll-interval-seconds")
                        .default_value("5")
                        .value_parser(clap::value_parser!(u64))
                        .help("设备码模式轮询间隔（秒）"),
                )
                .arg(Arg::new("user-token").long("user-token").num_args(1).help("手动写入 user access_token"))
                .arg(Arg::new("refresh-token").long("refresh-token").num_args(1).help("手动写入 refresh_token")),
        )
        .subcommand(
            Command::new("refresh-user")
                .about("使用 refresh_token 立即刷新 user token")
                .after_help("示例：\n  wpscli auth refresh-user"),
        )
        .subcommand(
            Command::new("harden")
                .about("巡检并加固本地敏感信息存储")
                .after_help(
                    "示例：\n  \
                     wpscli auth harden\n  \
                     wpscli auth harden --apply",
                )
                .arg(
                    Arg::new("apply")
                        .long("apply")
                        .action(ArgAction::SetTrue)
                        .help("执行推荐加固动作（删除旧明文、收紧文件权限）"),
                ),
        )
        .subcommand(
            Command::new("status")
                .about("查看授权状态、token 有效期与自动刷新就绪度")
                .after_help("示例：\n  wpscli auth status"),
        )
        .subcommand(
            Command::new("logout")
                .about("清空本地保存的凭据与 token 缓存")
                .after_help("示例：\n  wpscli auth logout"),
        )
}

pub async fn handle(args: &[String]) -> Result<(), WpsError> {
    let cmd = build_auth_command();
    let mut argv = vec!["auth".to_string()];
    argv.extend_from_slice(args);
    let matches = match cmd.try_get_matches_from(argv) {
        Ok(m) => m,
        Err(e) => {
            if matches!(e.kind(), ErrorKind::DisplayHelp | ErrorKind::DisplayVersion) {
                print!("{e}");
                return Ok(());
            }
            return Err(WpsError::Validation(e.to_string()));
        }
    };

    match matches.subcommand() {
        Some(("guide", _)) => run_auth_guide().await,
        Some(("setup", m)) => {
            let mut creds: Credentials = auth::load_credentials();
            let interactive = m.get_one::<String>("ak").is_none()
                && m.get_one::<String>("sk").is_none()
                && m.get_one::<String>("redirect-uri").is_none()
                && m.get_one::<String>("scope").is_none()
                && m.get_one::<String>("oauth-server").is_none()
                && m.get_one::<String>("oauth-device-endpoint").is_none()
                && m.get_one::<String>("oauth-token-endpoint").is_none();

            if interactive {
                print!("\x1B[2J\x1B[1;1H");
                show_art(include_str!("../art/setup.txt"));
            }

            if let Some(ak) = m.get_one::<String>("ak").cloned().or_else(|| {
                if interactive {
                    prompt("AK", creds.ak.clone())
                } else {
                    None
                }
            }) {
                if !ak.trim().is_empty() {
                    creds.ak = Some(ak);
                }
            }
            if let Some(sk) = m.get_one::<String>("sk").cloned().or_else(|| {
                if interactive {
                    prompt("SK", creds.sk.clone())
                } else {
                    None
                }
            }) {
                if !sk.trim().is_empty() {
                    creds.sk = Some(sk);
                }
            }
            if let Some(v) = m.get_one::<String>("redirect-uri").cloned().or_else(|| {
                if interactive {
                    prompt(
                        "OAuth Redirect URI",
                        creds.oauth_redirect_uri
                            .clone()
                            .or_else(|| Some("http://localhost:53682/callback".to_string())),
                    )
                } else {
                    None
                }
            }) {
                if !v.trim().is_empty() {
                    creds.oauth_redirect_uri = Some(v);
                }
            }
            if let Some(v) = m.get_one::<String>("scope").cloned().or_else(|| {
                if interactive {
                    prompt(
                        "OAuth Scope",
                        creds.oauth_scope
                            .clone()
                            .or_else(|| Some("kso.user_base.read".to_string())),
                    )
                } else {
                    None
                }
            }) {
                if !v.trim().is_empty() {
                    creds.oauth_scope = Some(v);
                }
            }
            if let Some(v) = m.get_one::<String>("oauth-server").cloned().or_else(|| {
                if interactive {
                    prompt("OAuth Server URL (optional)", creds.oauth_server.clone())
                } else {
                    None
                }
            }) {
                if !v.trim().is_empty() {
                    creds.oauth_server = Some(v);
                }
            }
            if let Some(v) = m.get_one::<String>("oauth-device-endpoint").cloned().or_else(|| {
                if interactive {
                    prompt("OAuth Device Endpoint (optional)", creds.oauth_device_endpoint.clone())
                } else {
                    None
                }
            }) {
                if !v.trim().is_empty() {
                    creds.oauth_device_endpoint = Some(v);
                }
            }
            if let Some(v) = m.get_one::<String>("oauth-token-endpoint").cloned().or_else(|| {
                if interactive {
                    prompt("OAuth Token Endpoint (optional)", creds.oauth_token_endpoint.clone())
                } else {
                    None
                }
            }) {
                if !v.trim().is_empty() {
                    creds.oauth_token_endpoint = Some(v);
                }
            }
            auth::save_credentials(&creds)?;
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "ok": true,
                    "message": "auth setup saved",
                    "next": ["wpscli auth login --user --print-url-only", "wpscli auth status"]
                }))
                .unwrap_or_else(|_| "{\"ok\":true}".to_string())
            );
            Ok(())
        }
        Some(("login", m)) => {
            let mut creds: Credentials = auth::load_credentials();
            if let Some(ak) = m.get_one::<String>("ak") {
                creds.ak = Some(ak.clone());
            }
            if let Some(sk) = m.get_one::<String>("sk") {
                creds.sk = Some(sk.clone());
            }
            if let Some(server) = m.get_one::<String>("oauth-server") {
                creds.oauth_server = Some(server.clone());
            }
            if let Some(v) = m.get_one::<String>("oauth-device-endpoint") {
                creds.oauth_device_endpoint = Some(v.clone());
            }
            if let Some(v) = m.get_one::<String>("oauth-token-endpoint") {
                creds.oauth_token_endpoint = Some(v.clone());
            }
            if let Some(v) = m.get_one::<String>("redirect-uri") {
                creds.oauth_redirect_uri = Some(v.clone());
            }
            if let Some(v) = m.get_one::<String>("scope") {
                creds.oauth_scope = Some(v.clone());
            }
            auth::save_credentials(&creds)?;

            let mut cache = auth::load_token_cache();
            let user_oauth = m.get_flag("user");
            if user_oauth {
                let client_id = creds
                    .ak
                    .clone()
                    .ok_or_else(|| {
                        WpsError::Auth("missing AK; run `wpscli auth setup --ak <AK> --sk <SK>`".to_string())
                    })?;
                let client_secret = creds
                    .sk
                    .clone()
                    .ok_or_else(|| {
                        WpsError::Auth("missing SK; run `wpscli auth setup --ak <AK> --sk <SK>`".to_string())
                    })?;
                let redirect_uri = m
                    .get_one::<String>("redirect-uri")
                    .cloned()
                    .or_else(|| std::env::var("WPS_CLI_OAUTH_REDIRECT_URI").ok())
                    .or_else(|| creds.oauth_redirect_uri.clone())
                    .unwrap_or_else(|| "http://localhost:53682/callback".to_string());
                let scope = m
                    .get_one::<String>("scope")
                    .cloned()
                    .or_else(|| std::env::var("WPS_CLI_OAUTH_SCOPE").ok())
                    .or_else(|| creds.oauth_scope.clone())
                    .unwrap_or_else(|| "kso.user_base.read".to_string());
                let state = m
                    .get_one::<String>("state")
                    .cloned()
                    .unwrap_or_else(|| "wpscli".to_string());
                let mode = m
                    .get_one::<String>("mode")
                    .cloned()
                    .unwrap_or_else(|| "local".to_string());
                let poll_interval_seconds = *m.get_one::<u64>("poll-interval-seconds").unwrap_or(&5);

                let auth_url = auth::build_oauth_authorize_url(&client_id, &redirect_uri, &scope, &state)?;
                let print_only = m.get_flag("print-url-only");
                let no_open = m.get_flag("no-open");
                let no_local_server = m.get_flag("no-local-server");
                let timeout_seconds = *m.get_one::<u64>("timeout-seconds").unwrap_or(&180);
                if mode != "remote-device" && m.get_one::<String>("code").is_none() && print_only {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&serde_json::json!({
                            "ok": true,
                            "mode": mode,
                            "authorize_url": auth_url,
                            "next": "After browser consent, run: wpscli auth login --user --code <authorization_code>",
                            "remote_hint": "For cross-machine login, run with --mode remote and then pass --callback-url '<redirected_full_url>'"
                        }))
                        .unwrap_or_else(|_| "{\"ok\":true}".to_string())
                    );
                    return Ok(());
                }

                let code = if mode == "remote-device" {
                    let session = auth::start_device_authorization(&client_id, &scope).await?;
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&serde_json::json!({
                            "ok": true,
                            "mode": "remote-device",
                            "verification_uri": session.verification_uri,
                            "verification_uri_complete": session.verification_uri_complete,
                            "user_code": session.user_code,
                            "expires_in": session.expires_in,
                            "poll_interval_seconds": poll_interval_seconds,
                            "next": "Open verification_uri on any browser device, authorize with user_code, CLI will poll automatically."
                        }))
                        .unwrap_or_else(|_| "{\"ok\":true}".to_string())
                    );
                    let new_cache = auth::poll_device_token(
                        &client_id,
                        &client_secret,
                        &session.device_code,
                        poll_interval_seconds.max(session.interval),
                        timeout_seconds.max(session.expires_in),
                    )
                    .await?;
                    cache.user_access_token = new_cache.user_access_token;
                    cache.user_refresh_token = new_cache.user_refresh_token;
                    cache.user_expires_at = new_cache.user_expires_at;
                    auth::save_token_cache(&cache)?;
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&serde_json::json!({"ok": true, "mode":"remote-device", "message": "user token acquired via device flow", "bin": "wpscli"}))
                            .unwrap_or_else(|_| "{\"ok\":true}".to_string())
                    );
                    return Ok(());
                } else if let Some(code) = m.get_one::<String>("code").cloned() {
                    code
                } else if let Some(callback_url) = m.get_one::<String>("callback-url") {
                    auth::extract_code_from_callback_url(callback_url, Some(&state))?
                } else if mode == "remote" {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&serde_json::json!({
                            "ok": true,
                            "mode": "remote",
                            "authorize_url": auth_url,
                            "redirect_uri": redirect_uri,
                            "next": [
                                "Open authorize_url in any browser machine and authorize",
                                "Copy the FULL redirected URL from browser address bar",
                                "Paste it back to this terminal"
                            ]
                        }))
                        .unwrap_or_else(|_| "{\"ok\":true}".to_string())
                    );
                    let pasted = prompt("请粘贴完整回调 URL", None)
                        .ok_or_else(|| WpsError::Auth("missing callback url input".to_string()))?;
                    auth::extract_code_from_callback_url(&pasted, Some(&state))?
                } else if no_local_server {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&serde_json::json!({
                            "ok": true,
                            "mode": "local",
                            "authorize_url": auth_url,
                            "next": "Open URL, then run: wpscli auth login --user --code <authorization_code>"
                        }))
                        .unwrap_or_else(|_| "{\"ok\":true}".to_string())
                    );
                    return Ok(());
                } else {
                    if !no_open {
                        let _ = auth::try_open_browser(&auth_url);
                    }
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&serde_json::json!({
                            "ok": true,
                            "mode": "local",
                            "authorize_url": auth_url,
                            "waiting_callback": redirect_uri,
                            "timeout_seconds": timeout_seconds,
                            "hint": "If browser does not open, copy authorize_url manually."
                        }))
                        .unwrap_or_else(|_| "{\"ok\":true}".to_string())
                    );
                    auth::wait_for_oauth_code(&redirect_uri, Some(&state), timeout_seconds)?
                };
                let new_cache = auth::exchange_user_token_with_code(
                    &client_id,
                    &client_secret,
                    &redirect_uri,
                    &code,
                )
                .await?;
                cache.user_access_token = new_cache.user_access_token;
                cache.user_refresh_token = new_cache.user_refresh_token;
                cache.user_expires_at = new_cache.user_expires_at;
            }

            if let Some(token) = m.get_one::<String>("user-token") {
                cache.user_access_token = Some(token.clone());
                cache.user_expires_at = Some(
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs()
                        + 7000,
                );
            }
            if let Some(rt) = m.get_one::<String>("refresh-token") {
                cache.user_refresh_token = Some(rt.clone());
            }
            auth::save_token_cache(&cache)?;
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({"ok": true, "message": "credentials saved", "bin": "wpscli"}))
                    .unwrap_or_else(|_| "{\"ok\":true}".to_string())
            );
            Ok(())
        }
        Some(("refresh-user", _)) => {
            let creds = auth::load_credentials();
            let client_id = creds
                .ak
                .clone()
                .ok_or_else(|| WpsError::Auth("missing AK".to_string()))?;
            let client_secret = creds
                .sk
                .clone()
                .ok_or_else(|| WpsError::Auth("missing SK".to_string()))?;
            let mut cache = auth::load_token_cache();
            let rt = cache
                .user_refresh_token
                .clone()
                .ok_or_else(|| WpsError::Auth("missing refresh_token; login first".to_string()))?;
            let refreshed = auth::refresh_user_token(&client_id, &client_secret, &rt).await?;
            cache.user_access_token = refreshed.user_access_token;
            cache.user_refresh_token = refreshed.user_refresh_token;
            cache.user_expires_at = refreshed.user_expires_at;
            auth::save_token_cache(&cache)?;
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({"ok": true, "message": "user token refreshed"}))
                    .unwrap_or_else(|_| "{\"ok\":true}".to_string())
            );
            Ok(())
        }
        Some(("harden", m)) => run_auth_harden(m),
        Some(("status", _)) => {
            let creds = auth::load_credentials();
            let cache = auth::load_token_cache();
            let has_ak = creds.ak.as_ref().map(|s| !s.is_empty()).unwrap_or(false);
            let has_sk = creds.sk.as_ref().map(|s| !s.is_empty()).unwrap_or(false);
            let now_ts = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            let payload = serde_json::json!({
                "ok": true,
                "config_dir": auth::config_dir(),
                "credentials_path": auth::credentials_path(),
                "token_cache_path": auth::token_cache_path(),
                "secure_storage": "aes256_gcm_with_os_keyring_fallback",
                "legacy_plaintext_credentials_exists": auth::legacy_credentials_path().exists(),
                "legacy_plaintext_token_cache_exists": auth::legacy_token_cache_path().exists(),
                "has_ak": has_ak,
                "has_sk": has_sk,
                "oauth_client_mode": "reuse_ak_sk",
                "oauth_server": creds.oauth_server,
                "oauth_device_endpoint": creds.oauth_device_endpoint,
                "oauth_token_endpoint": creds.oauth_token_endpoint,
                "has_app_token": cache.app_access_token.is_some(),
                "has_user_token": cache.user_access_token.is_some(),
                "has_refresh_token": cache.user_refresh_token.as_ref().map(|v| !v.is_empty()).unwrap_or(false),
                "user_expires_at": cache.user_expires_at,
                "now_ts": now_ts,
                "user_token_expired": cache.user_expires_at.map(|exp| exp <= now_ts),
                "auto_refresh_ready": has_ak
                    && has_sk
                    && cache.user_refresh_token.as_ref().map(|v| !v.is_empty()).unwrap_or(false),
                "tips": {
                    "quick_start": [
                      "wpscli auth setup --ak <AK> --sk <SK>",
                      "wpscli auth login --user --mode local",
                      "wpscli auth login --user --mode remote",
                      "wpscli auth login --user --mode remote-device",
                      "wpscli auth status"
                    ],
                    "set_ak_sk": "wpscli auth login --ak <AK> --sk <SK>",
                    "login_user_token_manual": "wpscli auth login --user-token <token>",
                    "force_refresh_now": "wpscli auth refresh-user",
                    "remote_oauth": "wpscli auth login --user --mode remote  # one command, then paste callback url",
                    "remote_device_oauth": "wpscli auth login --user --mode remote-device",
                    "set_device_endpoint": "wpscli auth setup --oauth-device-endpoint <url> --oauth-token-endpoint <url>  # for custom OAuth broker"
                }
            });
            println!(
                "{}",
                serde_json::to_string_pretty(&payload).unwrap_or_else(|_| "{\"ok\":true}".to_string())
            );
            Ok(())
        }
        Some(("logout", _)) => {
            let _ = std::fs::remove_file(auth::credentials_path());
            let _ = std::fs::remove_file(auth::token_cache_path());
            let _ = std::fs::remove_file(auth::legacy_credentials_path());
            let _ = std::fs::remove_file(auth::legacy_token_cache_path());
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({"ok": true, "message": "logged out"}))
                    .unwrap_or_else(|_| "{\"ok\":true}".to_string())
            );
            Ok(())
        }
        _ => Err(WpsError::Validation("unknown auth subcommand".to_string())),
    }
}

fn run_auth_harden(m: &ArgMatches) -> Result<(), WpsError> {
    let apply = m.get_flag("apply");
    let config_dir = auth::config_dir();
    let credentials_enc = auth::credentials_path();
    let token_cache_enc = auth::token_cache_path();
    let legacy_credentials = auth::legacy_credentials_path();
    let legacy_token_cache = auth::legacy_token_cache_path();
    let key_file = config_dir.join(".encryption_key");

    let mut checks: Vec<serde_json::Value> = Vec::new();
    let mut actions_applied: Vec<String> = Vec::new();

    let legacy_creds_exists = legacy_credentials.exists();
    let legacy_cache_exists = legacy_token_cache.exists();
    checks.push(serde_json::json!({
        "name": "legacy_plaintext_files",
        "status": if legacy_creds_exists || legacy_cache_exists { "warn" } else { "pass" },
        "details": {
            "credentials_json_exists": legacy_creds_exists,
            "token_cache_json_exists": legacy_cache_exists
        },
        "suggested_action": "执行 `wpscli auth harden --apply` 清理旧明文文件"
    }));

    if apply {
        if legacy_creds_exists {
            let _ = std::fs::remove_file(&legacy_credentials);
            actions_applied.push(format!("removed {}", legacy_credentials.display()));
        }
        if legacy_cache_exists {
            let _ = std::fs::remove_file(&legacy_token_cache);
            actions_applied.push(format!("removed {}", legacy_token_cache.display()));
        }
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perm_results = Vec::new();
        let targets: Vec<(&std::path::PathBuf, u32)> = vec![
            (&config_dir, 0o700),
            (&credentials_enc, 0o600),
            (&token_cache_enc, 0o600),
            (&key_file, 0o600),
        ];
        for (path, mode) in targets {
            if !path.exists() {
                continue;
            }
            let before = std::fs::metadata(path)
                .ok()
                .map(|m| m.permissions().mode() & 0o777);
            let mut changed = false;
            let mut ok = true;
            let mut err = None::<String>;
            if apply {
                if let Err(e) = std::fs::set_permissions(path, std::fs::Permissions::from_mode(mode)) {
                    ok = false;
                    err = Some(e.to_string());
                } else {
                    changed = before != Some(mode);
                    if changed {
                        actions_applied.push(format!("chmod {:o} {}", mode, path.display()));
                    }
                }
            }
            let after = std::fs::metadata(path)
                .ok()
                .map(|m| m.permissions().mode() & 0o777);
            perm_results.push(serde_json::json!({
                "path": path.display().to_string(),
                "recommended_mode": format!("{:o}", mode),
                "before_mode": before.map(|v| format!("{:o}", v)),
                "after_mode": after.map(|v| format!("{:o}", v)),
                "changed": changed,
                "ok": ok,
                "error": err
            }));
        }
        checks.push(serde_json::json!({
            "name": "filesystem_permissions",
            "status": "info",
            "details": perm_results,
            "suggested_action": "Unix/macOS 建议目录 700、敏感文件 600"
        }));
    }

    #[cfg(not(unix))]
    {
        checks.push(serde_json::json!({
            "name": "filesystem_permissions",
            "status": "info",
            "details": "non-unix platform, skip chmod guidance",
            "suggested_action": "使用系统 ACL/权限面板限制配置目录访问"
        }));
    }

    let mut env_risks = Vec::new();
    for k in [
        "WPS_CLI_AK",
        "WPS_CLI_SK",
        "WPS_CLI_TOKEN",
        "WPS_CLI_APP_TOKEN",
        "WPS_CLI_USER_TOKEN",
    ] {
        if std::env::var(k).ok().filter(|v| !v.is_empty()).is_some() {
            env_risks.push(k.to_string());
        }
    }
    checks.push(serde_json::json!({
        "name": "shell_env_exposure",
        "status": if env_risks.is_empty() { "pass" } else { "warn" },
        "details": {
            "exposed_vars": env_risks
        },
        "suggested_action": "避免在长期环境变量中保存高敏 token；优先使用 `wpscli auth setup/login` 本地加密存储"
    }));

    let cwd = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
    let env_file = cwd.join(".env");
    if env_file.exists() {
        let raw = std::fs::read_to_string(&env_file).unwrap_or_default();
        let mut hits = Vec::new();
        for key in [
            "WPS_CLI_AK",
            "WPS_CLI_SK",
            "WPS_CLI_TOKEN",
            "WPS_CLI_APP_TOKEN",
            "WPS_CLI_USER_TOKEN",
            "refresh_token",
        ] {
            if raw.contains(key) {
                hits.push(key.to_string());
            }
        }
        checks.push(serde_json::json!({
            "name": "dotenv_secret_check",
            "status": if hits.is_empty() { "pass" } else { "warn" },
            "details": {
                "dotenv_path": env_file.display().to_string(),
                "matched_keys": hits
            },
            "suggested_action": "确保 `.env` 不入库，并删除高敏项（尤其 access token）"
        }));
    } else {
        checks.push(serde_json::json!({
            "name": "dotenv_secret_check",
            "status": "info",
            "details": {
                "dotenv_path": env_file.display().to_string(),
                "exists": false
            },
            "suggested_action": "如使用 .env，请确保加入 .gitignore"
        }));
    }

    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({
            "ok": true,
            "command": "auth harden",
            "apply_mode": apply,
            "config_dir": config_dir,
            "checks": checks,
            "actions_applied": actions_applied,
            "next": [
                "wpscli auth status",
                "wpscli doctor",
                "定期执行: wpscli auth harden --apply"
            ]
        }))
        .unwrap_or_else(|_| "{\"ok\":true}".to_string())
    );
    Ok(())
}

fn prompt(label: &str, default: Option<String>) -> Option<String> {
    use std::io::{self, Write};
    let suffix = default
        .as_ref()
        .map(|d| format!(" [{d}]"))
        .unwrap_or_default();
    print!("{label}{suffix}: ");
    let _ = io::stdout().flush();
    let mut buf = String::new();
    if io::stdin().read_line(&mut buf).is_err() {
        return None;
    }
    let input = buf.trim().to_string();
    if input.is_empty() {
        default
    } else {
        Some(input)
    }
}

fn blue(text: &str) -> String {
    format!("\x1b[38;5;33m{text}\x1b[0m")
}

fn bold(text: &str) -> String {
    format!("\x1b[1m{text}\x1b[0m")
}

fn show_art(content: &str) {
    for line in content.lines() {
        println!("{}", blue(line));
    }
}

async fn run_auth_guide() -> Result<(), WpsError> {
    let intro = include_str!("../art/intro.txt");
    let setup = include_str!("../art/setup.txt");
    let config = include_str!("../art/config.txt");
    print!("\x1B[2J\x1B[1;1H");
    show_art(intro);
    show_art(setup);

    let mut creds = auth::load_credentials();
    if creds.ak.as_ref().map(|s| s.is_empty()).unwrap_or(true) {
        creds.ak = prompt("请输入 AK", creds.ak.clone());
    }
    if creds.sk.as_ref().map(|s| s.is_empty()).unwrap_or(true) {
        creds.sk = prompt("请输入 SK", creds.sk.clone());
    }
    if creds.ak.as_ref().map(|s| s.is_empty()).unwrap_or(true)
        || creds.sk.as_ref().map(|s| s.is_empty()).unwrap_or(true)
    {
        return Err(WpsError::Auth("AK/SK 不能为空".to_string()));
    }
    auth::save_credentials(&creds)?;

    let default_mode = if std::env::var("SSH_CONNECTION").is_ok() {
        "2".to_string()
    } else {
        "1".to_string()
    };
    println!("{}", blue("选择登录方式（回车使用默认）："));
    println!("  1) 本地浏览器自动回调（最省事）");
    println!("  2) 远程模式（同一命令粘贴回调 URL）");
    println!("  3) 设备码轮询（需配置设备端点）");
    let choice = prompt("输入序号", Some(default_mode)).unwrap_or_else(|| "1".to_string());

    let client_id = creds.ak.clone().unwrap_or_default();
    let client_secret = creds.sk.clone().unwrap_or_default();
    let redirect_uri = std::env::var("WPS_CLI_OAUTH_REDIRECT_URI")
        .ok()
        .or_else(|| creds.oauth_redirect_uri.clone())
        .unwrap_or_else(|| "http://localhost:53682/callback".to_string());
    let scope = std::env::var("WPS_CLI_OAUTH_SCOPE")
        .ok()
        .or_else(|| creds.oauth_scope.clone())
        .unwrap_or_else(|| "kso.user_base.read".to_string());
    let state = "wpscli";

    let new_cache = match choice.trim() {
        "1" => {
            let auth_url = auth::build_oauth_authorize_url(&client_id, &redirect_uri, &scope, state)?;
            let _ = auth::try_open_browser(&auth_url);
            println!("{}", blue("已打开浏览器授权，如未打开请手动访问："));
            println!("{}", auth_url);
            let code = auth::wait_for_oauth_code(&redirect_uri, Some(state), 300)?;
            auth::exchange_user_token_with_code(&client_id, &client_secret, &redirect_uri, &code).await?
        }
        "2" => {
            let auth_url = auth::build_oauth_authorize_url(&client_id, &redirect_uri, &scope, state)?;
            println!("{}", blue("请在任意浏览器打开以下链接完成授权："));
            println!("{}", auth_url);
            let pasted = prompt("请粘贴完整回调 URL", None)
                .ok_or_else(|| WpsError::Auth("缺少回调 URL".to_string()))?;
            let code = auth::extract_code_from_callback_url(&pasted, Some(state))?;
            auth::exchange_user_token_with_code(&client_id, &client_secret, &redirect_uri, &code).await?
        }
        "3" => {
            let session = auth::start_device_authorization(&client_id, &scope).await?;
            show_art(config);
            println!("{}", blue("请在浏览器完成设备码授权："));
            if let Some(uri) = session.verification_uri_complete.as_ref() {
                println!("  链接: {uri}");
                let _ = auth::try_open_browser(uri);
            } else if let Some(uri) = session.verification_uri.as_ref() {
                println!("  链接: {uri}");
                let _ = auth::try_open_browser(uri);
            }
            if let Some(code) = session.user_code.as_ref() {
                println!("  验证码: {}", bold(code));
            }
            auth::poll_device_token(
                &client_id,
                &client_secret,
                &session.device_code,
                session.interval.max(5),
                session.expires_in.max(300),
            )
            .await?
        }
        _ => return Err(WpsError::Validation("无效选项，请输入 1/2/3".to_string())),
    };

    let mut cache = auth::load_token_cache();
    cache.user_access_token = new_cache.user_access_token;
    cache.user_refresh_token = new_cache.user_refresh_token;
    cache.user_expires_at = new_cache.user_expires_at;
    auth::save_token_cache(&cache)?;

    let outro = include_str!("../art/outro.txt");
    show_art(outro);
    println!("{}", blue("✅ 授权完成：已保存 user token（支持自动刷新）"));
    println!("下一步可执行：wpscli auth status");
    Ok(())
}

