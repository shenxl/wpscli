use thiserror::Error;

#[derive(Debug, Error)]
pub enum WpsError {
    #[error("validation error: {0}")]
    Validation(String),
    #[error("auth error: {0}")]
    Auth(String),
    #[error("network error: {0}")]
    Network(String),
    #[error("descriptor error: {0}")]
    Descriptor(String),
    #[error("execution error: {0}")]
    Execution(String),
}

impl WpsError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::Validation(_) => "validation_error",
            Self::Auth(_) => "auth_error",
            Self::Network(_) => "network_error",
            Self::Descriptor(_) => "descriptor_error",
            Self::Execution(_) => "execution_error",
        }
    }
}

pub fn print_error_json(err: &WpsError) {
    let semantic = classify_semantic(err);
    let payload = serde_json::json!({
        "ok": false,
        "error": {
            "code": err.code(),
            "message": err.to_string(),
            "category": semantic.category,
            "retryable": semantic.retryable,
            "suggested_action": semantic.suggested_action
        }
    });
    println!("{}", serde_json::to_string_pretty(&payload).unwrap_or_else(|_| "{\"ok\":false}".to_string()));
}

struct Semantic<'a> {
    category: &'a str,
    retryable: bool,
    suggested_action: &'a str,
}

fn classify_semantic(err: &WpsError) -> Semantic<'static> {
    match err {
        WpsError::Validation(_) => Semantic {
            category: "parameter",
            retryable: false,
            suggested_action: "检查必填参数、枚举值和 JSON 格式后重试",
        },
        WpsError::Auth(_) => Semantic {
            category: "auth",
            retryable: false,
            suggested_action: "执行 `wpscli auth status` 与 `wpscli auth login --user` 重新授权",
        },
        WpsError::Descriptor(_) => Semantic {
            category: "parameter",
            retryable: false,
            suggested_action: "检查 service/endpoint 名称是否存在，先运行 `wpscli catalog`",
        },
        WpsError::Execution(_) => Semantic {
            category: "execution",
            retryable: false,
            suggested_action: "检查本地配置和输入数据，必要时开启 --dry-run 验证请求",
        },
        WpsError::Network(msg) => {
            let lower = msg.to_ascii_lowercase();
            if contains_any(&lower, &["403", "permission", "forbidden", "无权限"]) {
                Semantic {
                    category: "permission",
                    retryable: false,
                    suggested_action: "确认调用账号对目标资源有权限，必要时改用 --user-token",
                }
            } else if contains_any(&lower, &["401", "invalid_token", "unauthorized", "token"]) {
                Semantic {
                    category: "auth",
                    retryable: false,
                    suggested_action: "token 可能过期，执行 `wpscli auth login --user` 重新获取",
                }
            } else if contains_any(
                &lower,
                &[
                    "429",
                    "timeout",
                    "timed out",
                    "connection reset",
                    "temporarily",
                    "5xx",
                    "502",
                    "503",
                    "504",
                    "request failed after retries",
                ],
            ) {
                Semantic {
                    category: "retryable",
                    retryable: true,
                    suggested_action: "这是可重试错误，建议提升 --retry 或稍后重试",
                }
            } else {
                Semantic {
                    category: "network",
                    retryable: false,
                    suggested_action: "检查网络连通性、API 参数和服务端返回体",
                }
            }
        }
    }
}

fn contains_any(haystack: &str, needles: &[&str]) -> bool {
    needles.iter().any(|n| haystack.contains(n))
}
