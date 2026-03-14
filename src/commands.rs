use clap::{Arg, ArgAction, Command};

use crate::descriptor::ServiceDescriptor;
use crate::help_schema_contract;

pub fn build_root() -> Command {
    Command::new("wpscli")
        .about("WPS OpenAPI 命令行（面向开发者与 AI Agent）")
        .long_about(
            "WPS OpenAPI 命令行（面向开发者与 AI Agent）。\n\
             \n\
             本 CLI 提供三层调用能力：\n\
             - 业务技能/助手命令（高层语义任务）\n\
             - 描述符驱动动态 API 命令（覆盖面完整）\n\
             - raw 原始 HTTP 调用（任意路径）\n\
             \n\
             可先使用 `wpscli catalog` 快速发现服务与端点。",
        )
        .after_help(
            "示例：\n  \
             wpscli auth guide\n  \
             wpscli auth login --user --mode local\n  \
             wpscli auth status\n  \
             wpscli catalog drives\n  \
             wpscli drives list-files --path-param drive_id=<id> --path-param parent_id=0 --query page_size=5\n  \
             wpscli raw GET /v7/drives --query allotee_type=user --query page_size=5",
        )
        .arg(
            Arg::new("output")
                .long("output")
                .value_parser(["json", "compact", "table"])
                .default_value("json")
                .help("输出格式：json（默认）/compact/table")
                .global(true),
        )
}

pub fn build_service_command(desc: &ServiceDescriptor) -> Command {
    let service_name: &'static str = Box::leak(desc.service.clone().into_boxed_str());
    let mut cmd = Command::new(service_name)
        .about("描述符驱动的动态 API 命令");
    for ep in &desc.endpoints {
        let endpoint_name = help_schema_contract::endpoint_command_name(&ep.id);
        let endpoint_name: &'static str = Box::leak(endpoint_name.into_boxed_str());
        let auth_types = help_schema_contract::supported_auth_types(ep);
        let default_auth_owned = help_schema_contract::recommended_auth_type(&auth_types);
        let default_auth = match default_auth_owned.as_str() {
            "user" => "user",
            "cookie" => "cookie",
            _ => "app",
        };
        let mut ep_cmd = Command::new(endpoint_name)
            .about(ep.summary.clone())
            .arg(
                Arg::new("path-param")
                    .long("path-param")
                    .short('p')
                    .num_args(1)
                    .action(ArgAction::Append)
                    .help("路径参数，格式：key=value（可重复）"),
            )
            .arg(
                Arg::new("query")
                    .long("query")
                    .short('q')
                    .num_args(1)
                    .action(ArgAction::Append)
                    .help("查询参数，格式：key=value（可重复）"),
            )
            .arg(
                Arg::new("header")
                    .long("header")
                    .short('H')
                    .num_args(1)
                    .action(ArgAction::Append)
                    .help("请求头，格式：key=value（可重复）"),
            )
            .arg(Arg::new("body").long("body").num_args(1).help("JSON 字符串请求体"))
            .arg(
                Arg::new("auth-type")
                    .long("auth-type")
                    .value_parser(["app", "user", "cookie"])
                    .default_value(default_auth)
                    .help("鉴权类型：app / user / cookie"),
            )
            .arg(
                Arg::new("user-token")
                    .long("user-token")
                    .action(ArgAction::SetTrue)
                    .help("快捷方式：等价于 --auth-type user"),
            )
            .arg(Arg::new("dry-run").long("dry-run").action(ArgAction::SetTrue))
            .arg(
                Arg::new("retry")
                    .long("retry")
                    .default_value("1")
                    .value_parser(clap::value_parser!(u32))
                    .help("网络错误重试次数"),
            )
            .arg(Arg::new("paginate").long("paginate").action(ArgAction::SetTrue).help("自动翻页抓取完整结果"));
        for alias in endpoint_aliases(endpoint_name) {
            let leaked: &'static str = Box::leak(alias.into_boxed_str());
            ep_cmd = ep_cmd.visible_alias(leaked);
        }
        cmd = cmd.subcommand(ep_cmd);
    }
    cmd
}

fn endpoint_aliases(endpoint_name: &str) -> Vec<String> {
    help_schema_contract::endpoint_aliases(endpoint_name)
}

pub fn build_raw_command() -> Command {
    Command::new("raw")
        .about("直接调用任意 WPS API 路径")
        .after_help(
            "示例：\n  \
             wpscli raw GET /v7/drives --query allotee_type=user --query page_size=5\n  \
             wpscli raw POST /v7/messages/create --user-token --body '{\"chat_id\":\"xxx\",\"text\":\"hello\"}'",
        )
        .arg(Arg::new("method").required(true).value_parser(["GET", "POST", "PUT", "PATCH", "DELETE"]))
        .arg(Arg::new("path").required(true))
        .arg(
            Arg::new("query")
                .long("query")
                .short('q')
                .num_args(1)
                .action(ArgAction::Append),
        )
        .arg(
            Arg::new("header")
                .long("header")
                .short('H')
                .num_args(1)
                .action(ArgAction::Append),
        )
        .arg(Arg::new("body").long("body").num_args(1))
        .arg(
            Arg::new("auth-type")
                .long("auth-type")
                .value_parser(["app", "user", "cookie"])
                .default_value("app"),
        )
        .arg(
            Arg::new("user-token")
                .long("user-token")
                .action(ArgAction::SetTrue)
                .help("快捷方式：等价于 --auth-type user"),
        )
        .arg(Arg::new("dry-run").long("dry-run").action(ArgAction::SetTrue))
        .arg(
            Arg::new("retry")
                .long("retry")
                .default_value("1")
                .value_parser(clap::value_parser!(u32)),
        )
}

pub fn build_schema_command() -> Command {
    Command::new("schema")
        .about("查看服务/端点的参数与结构定义")
        .after_help(
            "示例：\n  \
             wpscli schema drives\n  \
             wpscli schema drives list-files\n  \
             wpscli schema drives list-files --mode invoke\n  \
             wpscli schema drives list-files --mode invoke --emit-template /tmp/list_files_template.json",
        )
        .arg(Arg::new("service").required(true))
        .arg(Arg::new("endpoint").required(false))
        .arg(
            Arg::new("mode")
                .long("mode")
                .value_parser(["raw", "invoke"])
                .default_value("raw")
                .help("输出模式：raw=原始 descriptor，invoke=执行导向 schema（含 command/template）"),
        )
        .arg(
            Arg::new("emit-template")
                .long("emit-template")
                .num_args(1)
                .help("将 invoke_template 写入文件（需同时提供 endpoint）"),
        )
}

pub fn build_catalog_command() -> Command {
    Command::new("catalog")
        .about("按服务或 show 分类列出可用 API")
        .after_help(
            "示例：\n  \
             wpscli catalog\n  \
             wpscli catalog --mode service\n  \
             wpscli catalog --mode ai\n  \
             wpscli catalog drives",
        )
        .arg(
            Arg::new("mode")
                .long("mode")
                .value_parser(["show", "service", "ai"])
                .default_value("show")
                .help("分组模式：show=按 show.json 层级，service=按服务平铺，ai=机器可读索引"),
        )
        .arg(Arg::new("service").required(false))
}

pub fn build_quality_command() -> Command {
    Command::new("quality")
        .about("运行描述符质量门禁（静态/构造/连通抽样）")
        .after_help(
            "示例：\n  \
             wpscli quality\n  \
             wpscli quality --connectivity-sample 10\n  \
             wpscli quality --connectivity-sample 10 --connectivity-auth user",
        )
        .arg(
            Arg::new("connectivity-sample")
                .long("connectivity-sample")
                .default_value("0")
                .value_parser(clap::value_parser!(u32))
                .help("连通抽样数量（仅 GET + 无必填参数端点，0=跳过）"),
        )
        .arg(
            Arg::new("connectivity-auth")
                .long("connectivity-auth")
                .value_parser(["auto", "app", "user", "cookie"])
                .default_value("auto")
                .help("连通抽样鉴权策略"),
        )
}

pub fn build_completions_command() -> Command {
    Command::new("completions")
        .about("生成 shell 自动补全脚本")
        .after_help(
            "示例：\n  \
             wpscli completions zsh > ~/.zfunc/_wpscli",
        )
        .arg(Arg::new("shell").required(true).value_parser(["bash", "zsh", "fish", "powershell", "elvish"]))
}

pub fn build_generate_skills_command() -> Command {
    Command::new("generate-skills")
        .about("根据描述符生成 SKILL.md 文档")
        .after_help(
            "示例：\n  \
             wpscli generate-skills --out-dir skills/generated",
        )
        .arg(Arg::new("out-dir").long("out-dir").required(false).default_value("skills/generated"))
}

pub fn build_ui_command() -> Command {
    Command::new("ui")
        .about("显示交互引导 ASCII 场景")
        .after_help(
            "示例：\n  \
             wpscli ui all\n  \
             wpscli ui framework",
        )
        .arg(
            Arg::new("scene")
                .required(false)
                .default_value("all")
                .help("场景名称")
                .value_parser(["intro", "features", "framework", "setup", "config", "format", "outro", "all"]),
        )
}

pub fn build_doctor_command() -> Command {
    Command::new("doctor")
        .about("执行本地诊断（安装状态/鉴权就绪/安全检查）")
        .after_help(
            "示例：\n  \
             wpscli doctor",
        )
}
