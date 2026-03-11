use std::collections::{BTreeSet, HashMap};
use std::time::Duration;

use clap::{Arg, ArgAction, Command};
use serde_json::Value;

use crate::error::WpsError;
use crate::executor;
use crate::link_resolver;
use super::{airpage, dbsheet, sheets};

pub fn command() -> Command {
    Command::new("doc")
        .about("文档读写助手（链接解析/读取/写入/检索）")
        .after_help(
            "示例：\n  \
             wpscli doc read-doc --url \"https://365.kdocs.cn/l/xxxx\" --user-token\n  \
             wpscli doc write-doc --url \"https://365.kdocs.cn/l/xxxx\" --target-format otl --content \"# 标题\" --user-token\n  \
             wpscli doc write-doc --url \"https://365.kdocs.cn/l/xxxx\" --target-format xlsx --xlsx-values-json '[[\"标题\",\"值\"],[\"A\",1]]' --user-token",
        )
        .subcommand(
            Command::new("resolve-link")
                .about("解析分享链接，获取 drive_id/file_id")
                .after_help("示例：\n  wpscli doc resolve-link \"https://365.kdocs.cn/l/xxxx\" --user-token")
                .arg(Arg::new("url").required(true))
                .arg(
                    Arg::new("auth-type")
                        .long("auth-type")
                        .value_parser(["app", "user", "cookie"])
                        .default_value("user")
                        .help("鉴权类型：app / user / cookie"),
                )
                .arg(Arg::new("user-token").long("user-token").action(ArgAction::SetTrue).help("快捷方式：等价于 --auth-type user"))
                .arg(Arg::new("dry-run").long("dry-run").action(ArgAction::SetTrue))
                .arg(
                    Arg::new("retry")
                        .long("retry")
                        .default_value("1")
                        .value_parser(clap::value_parser!(u32)),
                ),
        )
        .subcommand(
            Command::new("read-doc")
                .about("读取文档内容（支持 URL 或 drive/file_id）")
                .after_help(
                    "示例：\n  \
                     wpscli doc read-doc --url \"https://365.kdocs.cn/l/xxxx\" --format markdown --mode auto --user-token\n  \
                     wpscli doc read-doc --drive-id <drive_id> --file-id <file_id> --user-token\n  \
                     wpscli doc read-doc --url \"https://365.kdocs.cn/l/xxxx\" --xlsx-row-offset 500 --xlsx-row-head 200 --output-file /tmp/xlsx_page_3.json --user-token\n  \
                     wpscli doc read-doc --url \"https://365.kdocs.cn/l/xxxx\" --xlsx-row-head 1000 --export-parquet /tmp/xlsx.parquet --user-token",
                )
                .arg(Arg::new("url").long("url").num_args(1).help("文档分享链接"))
                .arg(Arg::new("drive-id").long("drive-id").num_args(1).help("网盘 ID（与 --file-id 搭配）"))
                .arg(Arg::new("file-id").long("file-id").num_args(1).help("文件 ID"))
                .arg(
                    Arg::new("format")
                        .long("format")
                        .default_value("markdown")
                        .value_parser(["markdown", "plain", "kdc"])
                        .help("输出格式"),
                )
                .arg(
                    Arg::new("mode")
                        .long("mode")
                        .default_value("sync")
                        .value_parser(["sync", "async", "auto"])
                        .help("读取模式：sync/async/auto"),
                )
                .arg(Arg::new("task-id").long("task-id").num_args(1).help("异步任务 ID（续跑时使用）"))
                .arg(
                    Arg::new("include-elements")
                        .long("include-elements")
                        .num_args(1)
                        .help("元素过滤，逗号分隔：para,table,component,textbox,all"),
                )
                .arg(
                    Arg::new("auth-type")
                        .long("auth-type")
                        .value_parser(["app", "user", "cookie"])
                        .default_value("user")
                        .help("鉴权类型：app / user / cookie"),
                )
                .arg(Arg::new("user-token").long("user-token").action(ArgAction::SetTrue).help("快捷方式：等价于 --auth-type user"))
                .arg(Arg::new("dry-run").long("dry-run").action(ArgAction::SetTrue))
                .arg(
                    Arg::new("retry")
                        .long("retry")
                        .default_value("1")
                        .value_parser(clap::value_parser!(u32)),
                )
                .arg(
                    Arg::new("poll-interval-ms")
                        .long("poll-interval-ms")
                        .default_value("1500")
                        .value_parser(clap::value_parser!(u64))
                        .help("异步任务轮询间隔（毫秒）"),
                )
                .arg(
                    Arg::new("max-wait-seconds")
                        .long("max-wait-seconds")
                        .default_value("120")
                        .value_parser(clap::value_parser!(u64))
                        .help("异步任务最大等待时长（秒）"),
                )
                .arg(
                    Arg::new("xlsx-max-sheets")
                        .long("xlsx-max-sheets")
                        .default_value("10")
                        .value_parser(clap::value_parser!(u32))
                        .help("xlsx 最多读取工作表数量（兼容参数，等价于 --xlsx-sheet-head）"),
                )
                .arg(
                    Arg::new("xlsx-max-rows")
                        .long("xlsx-max-rows")
                        .default_value("200")
                        .value_parser(clap::value_parser!(u32))
                        .help("xlsx 每个工作表最多读取行数（兼容参数，等价于 --xlsx-row-head）"),
                )
                .arg(
                    Arg::new("xlsx-max-cols")
                        .long("xlsx-max-cols")
                        .default_value("50")
                        .value_parser(clap::value_parser!(u32))
                        .help("xlsx 每个工作表最多读取列数（兼容参数，等价于 --xlsx-col-head）"),
                )
                .arg(
                    Arg::new("xlsx-sheet-offset")
                        .long("xlsx-sheet-offset")
                        .default_value("0")
                        .value_parser(clap::value_parser!(u32))
                        .help("xlsx 读取工作表偏移量（从第几个工作表开始）"),
                )
                .arg(
                    Arg::new("xlsx-sheet-head")
                        .long("xlsx-sheet-head")
                        .value_parser(clap::value_parser!(u32))
                        .help("xlsx 读取工作表数量上限（优先于 --xlsx-max-sheets）"),
                )
                .arg(
                    Arg::new("xlsx-row-offset")
                        .long("xlsx-row-offset")
                        .default_value("0")
                        .value_parser(clap::value_parser!(u32))
                        .help("xlsx 每个工作表行偏移量（从 active_area 的第几行开始）"),
                )
                .arg(
                    Arg::new("xlsx-row-head")
                        .long("xlsx-row-head")
                        .value_parser(clap::value_parser!(u32))
                        .help("xlsx 每个工作表读取行数上限（优先于 --xlsx-max-rows）"),
                )
                .arg(
                    Arg::new("xlsx-col-offset")
                        .long("xlsx-col-offset")
                        .default_value("0")
                        .value_parser(clap::value_parser!(u32))
                        .help("xlsx 每个工作表列偏移量（从 active_area 的第几列开始）"),
                )
                .arg(
                    Arg::new("xlsx-col-head")
                        .long("xlsx-col-head")
                        .value_parser(clap::value_parser!(u32))
                        .help("xlsx 每个工作表读取列数上限（优先于 --xlsx-max-cols）"),
                )
                .arg(
                    Arg::new("output-file")
                        .long("output-file")
                        .num_args(1)
                        .help("将 read-doc 结果写入指定 JSON 文件（大数据推荐）"),
                )
                .arg(
                    Arg::new("export-parquet")
                        .long("export-parquet")
                        .num_args(0..=1)
                        .default_missing_value("auto")
                        .help("将表格结果导出为 Parquet（可选路径；不传值则写入系统临时目录）"),
                )
                .arg(
                    Arg::new("output-stdout")
                        .long("output-stdout")
                        .action(ArgAction::SetTrue)
                        .help("配合 --output-file 使用：写文件后仍输出完整结果到 stdout"),
                )
                .arg(Arg::new("dbsheet-sheet-id").long("dbsheet-sheet-id").num_args(1).help("dbt 读取时指定 sheet_id（可选）"))
                .arg(Arg::new("dbsheet-where").long("dbsheet-where").num_args(1).help("dbt 读取过滤条件（SQL-like）"))
                .arg(Arg::new("dbsheet-fields").long("dbsheet-fields").num_args(1).help("dbt 读取字段投影，逗号分隔"))
                .arg(
                    Arg::new("dbsheet-limit")
                        .long("dbsheet-limit")
                        .default_value("100")
                        .value_parser(clap::value_parser!(usize))
                        .help("dbt 读取返回条数上限"),
                )
                .arg(
                    Arg::new("dbsheet-offset")
                        .long("dbsheet-offset")
                        .default_value("0")
                        .value_parser(clap::value_parser!(usize))
                        .help("dbt 读取偏移量"),
                ),
        )
        .subcommand(
            Command::new("write-doc")
                .about("按文件类型写入内容（otl/dbt/xlsx）")
                .after_help(
                    "示例：\n  \
                     wpscli doc write-doc --url \"https://365.kdocs.cn/l/xxxx\" --target-format otl --content \"# 周报\" --user-token\n  \
                     wpscli doc write-doc --file-id <file_id> --target-format dbt --db-action create --sheet-id 2 --records-json '[{\"热点标题\":\"A\"}]' --user-token\n  \
                     wpscli doc write-doc --file-id <file_id> --target-format xlsx --xlsx-sheet-id 0 --xlsx-values-json '[[\"项目\",\"状态\"],[\"M1\",\"完成\"]]' --user-token",
                )
                .arg(Arg::new("url").long("url").num_args(1).help("文档分享链接"))
                .arg(Arg::new("drive-id").long("drive-id").num_args(1).help("网盘 ID（与 --file-id 搭配）"))
                .arg(Arg::new("file-id").long("file-id").num_args(1).help("文件 ID"))
                .arg(
                    Arg::new("target-format")
                        .long("target-format")
                        .default_value("auto")
                        .value_parser(["auto", "otl", "dbt", "xlsx"])
                        .help("目标格式（默认自动识别）"),
                )
                .arg(
                    Arg::new("write-mode")
                        .long("write-mode")
                        .default_value("replace")
                        .value_parser(["replace", "append"])
                        .help("OTL 写入模式：replace 或 append"),
                )
                .arg(Arg::new("content").long("content").num_args(1).help("OTL 写入内容（Markdown）"))
                .arg(Arg::new("content-file").long("content-file").num_args(1).help("从文件读取 OTL Markdown 内容"))
                .arg(
                    Arg::new("db-action")
                        .long("db-action")
                        .default_value("create")
                        .value_parser(["create", "update", "delete"])
                        .help("DBT 操作类型"),
                )
                .arg(Arg::new("sheet-id").long("sheet-id").num_args(1).help("DBT 必填：工作表 ID"))
                .arg(Arg::new("records-json").long("records-json").num_args(1).help("DBT 记录 JSON 字符串"))
                .arg(Arg::new("records-file").long("records-file").num_args(1).help("从文件读取 DBT 记录 JSON"))
                .arg(Arg::new("record-id").long("record-id").num_args(1).help("DBT update 单条记录 ID"))
                .arg(
                    Arg::new("db-batch-size")
                        .long("db-batch-size")
                        .default_value("100")
                        .value_parser(clap::value_parser!(usize))
                        .help("DBT 批量写入大小"),
                )
                .arg(Arg::new("db-prefer-id").long("db-prefer-id").action(ArgAction::SetTrue).help("DBT 写入优先使用 id 字段映射"))
                .arg(Arg::new("xlsx-sheet-id").long("xlsx-sheet-id").num_args(1).help("XLSX 写入目标 worksheet_id（可选）"))
                .arg(Arg::new("xlsx-row-from").long("xlsx-row-from").num_args(1).help("XLSX 写入起始行（默认 0；append 模式自动追加）"))
                .arg(Arg::new("xlsx-col-from").long("xlsx-col-from").num_args(1).help("XLSX 写入起始列（默认取工作表 active_area.col_from）"))
                .arg(
                    Arg::new("xlsx-write-mode")
                        .long("xlsx-write-mode")
                        .default_value("replace")
                        .value_parser(["replace", "append"])
                        .help("XLSX 写入模式：replace/append"),
                )
                .arg(Arg::new("xlsx-values-json").long("xlsx-values-json").num_args(1).help("XLSX 写入二维数组 JSON"))
                .arg(Arg::new("xlsx-values-file").long("xlsx-values-file").num_args(1).help("从文件读取 XLSX 写入 JSON"))
                .arg(
                    Arg::new("auth-type")
                        .long("auth-type")
                        .value_parser(["app", "user", "cookie"])
                        .default_value("user")
                        .help("鉴权类型：app / user / cookie"),
                )
                .arg(Arg::new("user-token").long("user-token").action(ArgAction::SetTrue).help("快捷方式：等价于 --auth-type user"))
                .arg(Arg::new("dry-run").long("dry-run").action(ArgAction::SetTrue))
                .arg(
                    Arg::new("retry")
                        .long("retry")
                        .default_value("1")
                        .value_parser(clap::value_parser!(u32)),
                ),
        )
        .subcommand(
            Command::new("file-info")
                .about("读取文件元信息")
                .after_help("示例：\n  wpscli doc file-info --url \"https://365.kdocs.cn/l/xxxx\" --user-token")
                .arg(Arg::new("url").long("url").num_args(1).help("文档分享链接"))
                .arg(Arg::new("drive-id").long("drive-id").num_args(1).help("网盘 ID"))
                .arg(Arg::new("file-id").long("file-id").num_args(1).help("文件 ID"))
                .arg(
                    Arg::new("auth-type")
                        .long("auth-type")
                        .value_parser(["app", "user", "cookie"])
                        .default_value("user")
                        .help("鉴权类型：app / user / cookie"),
                )
                .arg(Arg::new("user-token").long("user-token").action(ArgAction::SetTrue).help("快捷方式：等价于 --auth-type user"))
                .arg(Arg::new("dry-run").long("dry-run").action(ArgAction::SetTrue))
                .arg(
                    Arg::new("retry")
                        .long("retry")
                        .default_value("1")
                        .value_parser(clap::value_parser!(u32)),
                ),
        )
        .subcommand(
            Command::new("list-files")
                .about("列出子文件/子目录")
                .after_help(
                    "示例：\n  \
                     wpscli doc list-files --url \"https://365.kdocs.cn/l/xxxx\" --user-token\n  \
                     wpscli doc list-files --drive-id <drive_id> --parent-id <folder_id> --page-size 100 --user-token",
                )
                .arg(Arg::new("url").long("url").num_args(1).help("分享链接（默认 parent=file_id）"))
                .arg(Arg::new("drive-id").long("drive-id").num_args(1).help("网盘 ID"))
                .arg(Arg::new("parent-id").long("parent-id").num_args(1).help("父目录 ID"))
                .arg(
                    Arg::new("page-size")
                        .long("page-size")
                        .default_value("50")
                        .value_parser(clap::value_parser!(u32))
                        .help("每页数量"),
                )
                .arg(Arg::new("page-token").long("page-token").num_args(1).help("翻页游标"))
                .arg(
                    Arg::new("auth-type")
                        .long("auth-type")
                        .value_parser(["app", "user", "cookie"])
                        .default_value("user")
                        .help("鉴权类型：app / user / cookie"),
                )
                .arg(Arg::new("user-token").long("user-token").action(ArgAction::SetTrue).help("快捷方式：等价于 --auth-type user"))
                .arg(Arg::new("dry-run").long("dry-run").action(ArgAction::SetTrue))
                .arg(
                    Arg::new("retry")
                        .long("retry")
                        .default_value("1")
                        .value_parser(clap::value_parser!(u32)),
                ),
        )
        .subcommand(
            Command::new("search")
                .about("按关键字搜索文件")
                .after_help("示例：\n  wpscli doc search --keyword \"周报\" --user-token")
                .arg(Arg::new("keyword").long("keyword").short('k').required(true).num_args(1))
                .arg(Arg::new("drive-id").long("drive-id").num_args(1))
                .arg(
                    Arg::new("auth-type")
                        .long("auth-type")
                        .value_parser(["app", "user", "cookie"])
                        .default_value("user")
                        .help("鉴权类型：app / user / cookie"),
                )
                .arg(Arg::new("user-token").long("user-token").action(ArgAction::SetTrue).help("快捷方式：等价于 --auth-type user"))
                .arg(Arg::new("dry-run").long("dry-run").action(ArgAction::SetTrue))
                .arg(
                    Arg::new("retry")
                        .long("retry")
                        .default_value("1")
                        .value_parser(clap::value_parser!(u32)),
                ),
        )
}

pub async fn handle(args: &[String]) -> Result<serde_json::Value, WpsError> {
    let mut argv = vec!["doc".to_string()];
    argv.extend_from_slice(args);
    let m = command()
        .try_get_matches_from(argv)
        .map_err(|e| WpsError::Validation(e.to_string()))?;
    match m.subcommand() {
        Some(("resolve-link", s)) => {
            let auth = effective_auth_type(s);
            let dry = s.get_flag("dry-run");
            let retry = *s.get_one::<u32>("retry").unwrap_or(&1);
            link_resolver::resolve_share_link(
                s.get_one::<String>("url").expect("required"),
                &auth,
                dry,
                retry,
            )
            .await
        }
        Some(("read-doc", s)) => {
            let auth = effective_auth_type(s);
            let dry = s.get_flag("dry-run");
            let retry = *s.get_one::<u32>("retry").unwrap_or(&1);
            let poll_interval_ms = *s.get_one::<u64>("poll-interval-ms").unwrap_or(&1500);
            let max_wait_seconds = *s.get_one::<u64>("max-wait-seconds").unwrap_or(&120);
            let xlsx_sheet_offset = *s.get_one::<u32>("xlsx-sheet-offset").unwrap_or(&0);
            let xlsx_sheet_head = s
                .get_one::<u32>("xlsx-sheet-head")
                .copied()
                .unwrap_or_else(|| *s.get_one::<u32>("xlsx-max-sheets").unwrap_or(&10));
            let xlsx_row_offset = *s.get_one::<u32>("xlsx-row-offset").unwrap_or(&0);
            let xlsx_row_head = s
                .get_one::<u32>("xlsx-row-head")
                .copied()
                .unwrap_or_else(|| *s.get_one::<u32>("xlsx-max-rows").unwrap_or(&200));
            let xlsx_col_offset = *s.get_one::<u32>("xlsx-col-offset").unwrap_or(&0);
            let xlsx_col_head = s
                .get_one::<u32>("xlsx-col-head")
                .copied()
                .unwrap_or_else(|| *s.get_one::<u32>("xlsx-max-cols").unwrap_or(&50));
            let (drive_id, file_id) = resolve_doc_ids(s, &auth, dry, retry).await?;
            let file_meta = if dry {
                None
            } else {
                Some(fetch_file_meta(&drive_id, &file_id, &auth, dry, retry).await?)
            };
            let ext = file_meta
                .as_ref()
                .and_then(file_extension_from_meta)
                .unwrap_or_default();

            if let Some(meta) = &file_meta {
                if is_xlsx_file(meta) {
                    let resp = sheets::read_xlsx_via_sheets(
                        &drive_id,
                        &file_id,
                        &auth,
                        dry,
                        retry,
                        xlsx_sheet_offset,
                        xlsx_sheet_head,
                        xlsx_row_offset,
                        xlsx_row_head,
                        xlsx_col_offset,
                        xlsx_col_head,
                        meta.clone(),
                        "sheets_range_data_primary",
                        None,
                    )
                    .await?;
                    return finalize_read_output(s, resp);
                }
                if is_ppt_extension(&ext) {
                    let aidocs_resp = read_via_aidocs_extract(
                        meta,
                        &auth,
                        dry,
                        retry,
                        poll_interval_ms,
                        max_wait_seconds,
                        "aidocs_extract_primary",
                        None,
                    )
                    .await?;
                    // PPT family should never go through drives /content path.
                    return finalize_read_output(s, aidocs_resp);
                }
                if ext == "dbt" {
                    let sheet_id = if let Some(v) = s.get_one::<String>("dbsheet-sheet-id") {
                        v.clone()
                    } else {
                        dbsheet::resolve_first_sheet_id(&file_id, &auth, dry, retry).await?
                    };
                    let selected_fields = parse_csv_fields(s.get_one::<String>("dbsheet-fields"));
                    let selected = dbsheet::sql_like_select_records(
                        &file_id,
                        &sheet_id,
                        s.get_one::<String>("dbsheet-where").cloned(),
                        selected_fields.clone(),
                        *s.get_one::<usize>("dbsheet-limit").unwrap_or(&100),
                        *s.get_one::<usize>("dbsheet-offset").unwrap_or(&0),
                        100,
                        &auth,
                        dry,
                        retry,
                    )
                    .await?;
                    let dbt_resp = serde_json::json!({
                        "ok": true,
                        "status": 200,
                        "data": {
                            "code": 0,
                            "msg": "ok",
                            "data": {
                                "src_format": "dbt",
                                "file_id": file_id,
                                "sheet_id": sheet_id,
                                "where": s.get_one::<String>("dbsheet-where").cloned(),
                                "fields": selected_fields,
                                "total": selected.total,
                                "records": selected.records
                            }
                        }
                    });
                    return finalize_read_output(s, attach_read_mode(dbt_resp, "dbsheet_sql_like_primary"));
                }
            }

            let format = s
                .get_one::<String>("format")
                .cloned()
                .unwrap_or_else(|| "markdown".to_string());
            let input_mode = s
                .get_one::<String>("mode")
                .cloned()
                .unwrap_or_else(|| "sync".to_string());
            let mut effective_mode = input_mode.clone();
            if effective_mode == "sync" && prefer_auto_mode_for_extension(&ext) {
                effective_mode = "auto".to_string();
            }
            let task_id = s.get_one::<String>("task-id").cloned();
            let include_elements = s.get_one::<String>("include-elements").cloned();

            let content_resp = read_content_with_async(
                &drive_id,
                &file_id,
                &auth,
                dry,
                retry,
                &format,
                &effective_mode,
                task_id,
                include_elements,
                poll_interval_ms,
                max_wait_seconds,
            )
            .await?;
            if let Some(fallback) = maybe_fallback_read_xlsx(
                &content_resp,
                &drive_id,
                &file_id,
                &auth,
                dry,
                retry,
                xlsx_sheet_offset,
                xlsx_sheet_head,
                xlsx_row_offset,
                xlsx_row_head,
                xlsx_col_offset,
                xlsx_col_head,
                file_meta.as_ref(),
            )
            .await?
            {
                return finalize_read_output(s, fallback);
            }
            finalize_read_output(s, attach_read_mode(content_resp, "drives_content"))
        }
        Some(("write-doc", s)) => {
            let auth = effective_auth_type(s);
            let dry = s.get_flag("dry-run");
            let retry = *s.get_one::<u32>("retry").unwrap_or(&1);
            let (drive_id, file_id) = resolve_doc_ids(s, &auth, dry, retry).await?;
            let target_format = s
                .get_one::<String>("target-format")
                .cloned()
                .unwrap_or_else(|| "auto".to_string());
            let file_meta = if dry {
                let fake_name = match target_format.as_str() {
                    "dbt" => "dry_run.dbt",
                    "xlsx" => "dry_run.xlsx",
                    _ => "dry_run.otl",
                };
                serde_json::json!({"data":{"data":{"name": fake_name}}})
            } else {
                fetch_file_meta(&drive_id, &file_id, &auth, dry, retry).await?
            };
            let mut ext = file_extension_from_meta(&file_meta).unwrap_or_default();
            if target_format != "auto" {
                ext = target_format;
            }
            match ext.as_str() {
                "otl" => write_otl_markdown(s, &file_id, &auth, dry, retry).await,
                "dbt" => write_dbt_records(s, &file_id, &auth, dry, retry).await,
                "xlsx" | "xls" | "et" => write_xlsx_values(s, &file_id, &auth, dry, retry).await,
                _ => Err(WpsError::Validation(format!(
                    "write-doc 当前仅支持 otl/dbt/xlsx，检测到: {ext}"
                ))),
            }
        }
        Some(("file-info", s)) => {
            let auth = effective_auth_type(s);
            let dry = s.get_flag("dry-run");
            let retry = *s.get_one::<u32>("retry").unwrap_or(&1);
            let (drive_id, file_id) = resolve_doc_ids(s, &auth, dry, retry).await?;
            executor::execute_raw(
                "GET",
                &format!("/v7/drives/{drive_id}/files/{file_id}/meta"),
                HashMap::new(),
                HashMap::new(),
                None,
                &auth,
                dry,
                retry,
            )
            .await
        }
        Some(("list-files", s)) => {
            let auth = effective_auth_type(s);
            let dry = s.get_flag("dry-run");
            let retry = *s.get_one::<u32>("retry").unwrap_or(&1);
            let (drive_id, file_id_from_url) = resolve_doc_ids(s, &auth, dry, retry).await?;
            let parent_id = s
                .get_one::<String>("parent-id")
                .cloned()
                .unwrap_or(file_id_from_url);
            let mut q = HashMap::new();
            q.insert(
                "page_size".to_string(),
                s.get_one::<u32>("page-size").copied().unwrap_or(50).to_string(),
            );
            if let Some(token) = s.get_one::<String>("page-token") {
                q.insert("page_token".to_string(), token.clone());
            }
            executor::execute_raw(
                "GET",
                &format!("/v7/drives/{drive_id}/files/{parent_id}/children"),
                q,
                HashMap::new(),
                None,
                &auth,
                dry,
                retry,
            )
            .await
        }
        Some(("search", s)) => {
            let mut q = HashMap::new();
            q.insert("keyword".to_string(), s.get_one::<String>("keyword").expect("required").clone());
            if let Some(d) = s.get_one::<String>("drive-id") {
                q.insert("drive_id".to_string(), d.clone());
            }
            let auth = effective_auth_type(s);
            let dry = s.get_flag("dry-run");
            let retry = *s.get_one::<u32>("retry").unwrap_or(&1);
            executor::execute_raw(
                "GET",
                "/v7/files/search",
                q,
                HashMap::new(),
                None,
                &auth,
                dry,
                retry,
            )
            .await
        }
        _ => Err(WpsError::Validation("unknown doc subcommand".to_string())),
    }
}

fn api_status_ok(v: &Value) -> bool {
    v.get("ok").and_then(|x| x.as_bool()).unwrap_or(false)
}

fn api_code(v: &Value) -> Option<i64> {
    v.get("data").and_then(|x| x.get("code")).and_then(|x| x.as_i64())
}

fn attach_read_mode(mut resp: Value, read_mode: &str) -> Value {
    if let Some(data_obj) = resp
        .get_mut("data")
        .and_then(|v| v.get_mut("data"))
        .and_then(|v| v.as_object_mut())
    {
        if !data_obj.contains_key("read_mode") {
            data_obj.insert("read_mode".to_string(), Value::String(read_mode.to_string()));
        }
    }
    resp
}

fn file_extension_from_meta(file_meta_resp: &Value) -> Option<String> {
    let name = file_meta_resp
        .get("data")
        .and_then(|v| v.get("data"))
        .and_then(|v| v.get("name"))
        .and_then(|v| v.as_str())?;
    let ext = name.rsplit('.').next()?.to_lowercase();
    if ext == name.to_lowercase() {
        return None;
    }
    Some(ext)
}

fn prefer_auto_mode_for_extension(ext: &str) -> bool {
    matches!(ext, "pdf" | "ppt" | "pptx" | "doc" | "docx")
}

fn is_ppt_extension(ext: &str) -> bool {
    matches!(ext, "ppt" | "pptx")
}

fn content_task_running(resp: &Value) -> bool {
    resp.get("data")
        .and_then(|v| v.get("data"))
        .and_then(|v| v.get("task_status"))
        .and_then(|v| v.as_str())
        .map(|s| s.eq_ignore_ascii_case("running"))
        .unwrap_or(false)
}

fn content_task_id(resp: &Value) -> Option<String> {
    resp.get("data")
        .and_then(|v| v.get("data"))
        .and_then(|v| v.get("task_id"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
}

async fn request_file_content(
    drive_id: &str,
    file_id: &str,
    auth_type: &str,
    dry_run: bool,
    retry: u32,
    format: &str,
    mode: &str,
    task_id: Option<&str>,
    include_elements: Option<&str>,
) -> Result<Value, WpsError> {
    let mut q = HashMap::new();
    q.insert("format".to_string(), format.to_string());
    q.insert("mode".to_string(), mode.to_string());
    if let Some(t) = task_id {
        q.insert("task_id".to_string(), t.to_string());
    }
    if let Some(include) = include_elements {
        q.insert("include_elements".to_string(), include.to_string());
    }
    executor::execute_raw(
        "GET",
        &format!("/v7/drives/{drive_id}/files/{file_id}/content"),
        q,
        HashMap::new(),
        None,
        auth_type,
        dry_run,
        retry,
    )
    .await
}

async fn read_content_with_async(
    drive_id: &str,
    file_id: &str,
    auth_type: &str,
    dry_run: bool,
    retry: u32,
    format: &str,
    mode: &str,
    task_id: Option<String>,
    include_elements: Option<String>,
    poll_interval_ms: u64,
    max_wait_seconds: u64,
) -> Result<Value, WpsError> {
    let mut resp = request_file_content(
        drive_id,
        file_id,
        auth_type,
        dry_run,
        retry,
        format,
        mode,
        task_id.as_deref(),
        include_elements.as_deref(),
    )
    .await?;
    if dry_run || !api_status_ok(&resp) {
        return Ok(resp);
    }
    let Some(task_id) = content_task_id(&resp) else {
        return Ok(resp);
    };
    if poll_interval_ms == 0 || max_wait_seconds == 0 {
        return Ok(resp);
    }
    let max_polls = std::cmp::max(1, (max_wait_seconds * 1000) / poll_interval_ms);
    let mut best_resp = resp.clone();
    let mut best_len = markdown_len(&resp);
    for _ in 0..max_polls {
        tokio::time::sleep(Duration::from_millis(poll_interval_ms)).await;
        resp = request_file_content(
            drive_id,
            file_id,
            auth_type,
            dry_run,
            retry,
            format,
            "auto",
            Some(&task_id),
            include_elements.as_deref(),
        )
        .await?;
        let cur_len = markdown_len(&resp);
        if cur_len > best_len {
            best_len = cur_len;
            best_resp = resp.clone();
        }
        if !content_task_running(&resp) && cur_len > 0 {
            // one extra stabilization poll to mitigate eventual consistency
            tokio::time::sleep(Duration::from_millis(poll_interval_ms)).await;
            let final_try = request_file_content(
                drive_id,
                file_id,
                auth_type,
                dry_run,
                retry,
                format,
                "auto",
                Some(&task_id),
                include_elements.as_deref(),
            )
            .await?;
            let final_len = markdown_len(&final_try);
            if final_len > best_len {
                best_resp = final_try;
            }
            return Ok(best_resp);
        }
    }
    Ok(best_resp)
}

fn markdown_len(resp: &Value) -> usize {
    resp.get("data")
        .and_then(|v| v.get("data"))
        .and_then(|v| v.get("markdown"))
        .and_then(|v| v.as_str())
        .map(|s| s.len())
        .unwrap_or(0)
}

async fn read_via_aidocs_extract(
    file_meta_resp: &Value,
    auth_type: &str,
    dry_run: bool,
    retry: u32,
    poll_interval_ms: u64,
    max_wait_seconds: u64,
    read_mode: &str,
    trigger_response: Option<Value>,
) -> Result<Value, WpsError> {
    let meta = file_meta_resp
        .get("data")
        .and_then(|v| v.get("data"))
        .cloned()
        .unwrap_or(Value::Null);
    let file_id = meta.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let drive_id = meta
        .get("drive_id")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let link_id = meta
        .get("link_id")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .unwrap_or_default();
    if link_id.is_empty() {
        return Ok(serde_json::json!({
            "ok": false,
            "status": 400,
            "data": {"code": 400, "msg": "missing link_id for aidocs extract"},
            "read_mode": read_mode
        }));
    }

    let commit_body = serde_json::json!({
        "file_id": link_id,
        "item_name": {
            "type": "single",
            "fields": [
                {
                    "name": "全文",
                    "desc": "提取文档全部正文内容，按原文顺序输出，不要遗漏任意页面或章节"
                }
            ]
        }
    });
    let commit_resp = executor::execute_raw(
        "POST",
        "/v7/aidocs/extract/commit",
        HashMap::new(),
        HashMap::new(),
        Some(commit_body.to_string()),
        auth_type,
        dry_run,
        retry,
    )
    .await?;
    if dry_run || !api_status_ok(&commit_resp) {
        return Ok(commit_resp);
    }
    let task_id = commit_resp
        .get("data")
        .and_then(|v| v.get("data"))
        .and_then(|v| v.get("task_id"))
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    if task_id.is_empty() {
        return Ok(commit_resp);
    }

    let mut res = Value::Null;
    let max_polls = if poll_interval_ms == 0 || max_wait_seconds == 0 {
        1
    } else {
        std::cmp::max(1, (max_wait_seconds * 1000) / poll_interval_ms)
    };
    for i in 0..max_polls {
        if i > 0 && poll_interval_ms > 0 {
            tokio::time::sleep(Duration::from_millis(poll_interval_ms)).await;
        }
        let body = serde_json::json!({ "task_id": task_id });
        res = executor::execute_raw(
            "POST",
            "/v7/aidocs/extract/res",
            HashMap::new(),
            HashMap::new(),
            Some(body.to_string()),
            auth_type,
            dry_run,
            retry,
        )
        .await?;
        if !api_status_ok(&res) {
            break;
        }
        let status = res
            .get("data")
            .and_then(|v| v.get("data"))
            .and_then(|v| v.get("status"))
            .and_then(|v| v.as_str())
            .unwrap_or("");
        if !status.eq_ignore_ascii_case("running") {
            break;
        }
    }

    if !api_status_ok(&res) {
        return Ok(res);
    }
    let item_info = res
        .get("data")
        .and_then(|v| v.get("data"))
        .and_then(|v| v.get("item_info"))
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    let mut chunks = Vec::new();
    for item in &item_info {
        let values = item
            .get("value")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();
        for v in values {
            if let Some(s) = v.as_str() {
                if !s.trim().is_empty() {
                    chunks.push(s.to_string());
                }
            }
        }
    }
    let markdown = chunks.join("\n\n");

    let mut out = serde_json::json!({
        "ok": true,
        "status": 200,
        "data": {
            "code": 0,
            "msg": "ok",
            "data": {
                "read_mode": read_mode,
                "src_format": file_extension_from_meta(file_meta_resp).unwrap_or_default(),
                "drive_id": drive_id,
                "file_id": file_id,
                "task_id": task_id,
                "task_status": res.get("data").and_then(|v| v.get("data")).and_then(|v| v.get("status")).cloned().unwrap_or(Value::Null),
                "markdown": markdown,
                "item_info": item_info,
                "file_meta": meta
            }
        },
        "raw": {
            "commit": commit_resp,
            "result": res
        }
    });
    if let Some(trigger) = trigger_response {
        out["fallback"] = serde_json::json!({
            "reason": "content_extract_incomplete_or_failed",
            "trigger_response": trigger
        });
    }
    Ok(out)
}

async fn maybe_fallback_read_xlsx(
    content_resp: &Value,
    drive_id: &str,
    file_id: &str,
    auth_type: &str,
    dry_run: bool,
    retry: u32,
    xlsx_sheet_offset: u32,
    xlsx_sheet_head: u32,
    xlsx_row_offset: u32,
    xlsx_row_head: u32,
    xlsx_col_offset: u32,
    xlsx_col_head: u32,
    file_meta_cached: Option<&Value>,
) -> Result<Option<Value>, WpsError> {
    if api_status_ok(content_resp) {
        return Ok(None);
    }
    if api_code(content_resp) != Some(400008018) {
        return Ok(None);
    }
    if let Some(meta) = file_meta_cached {
        if !is_xlsx_file(meta) {
            return Ok(None);
        }
    }

    let file_meta = if let Some(v) = file_meta_cached {
        v.clone()
    } else {
        fetch_file_meta(drive_id, file_id, auth_type, dry_run, retry).await?
    };
    let resp = sheets::read_xlsx_via_sheets(
        drive_id,
        file_id,
        auth_type,
        dry_run,
        retry,
        xlsx_sheet_offset,
        xlsx_sheet_head,
        xlsx_row_offset,
        xlsx_row_head,
        xlsx_col_offset,
        xlsx_col_head,
        file_meta,
        "sheets_range_data_fallback",
        Some(content_resp.clone()),
    )
    .await?;
    Ok(Some(resp))
}

async fn fetch_file_meta(
    drive_id: &str,
    file_id: &str,
    auth_type: &str,
    dry_run: bool,
    retry: u32,
) -> Result<Value, WpsError> {
    executor::execute_raw(
        "GET",
        &format!("/v7/drives/{drive_id}/files/{file_id}/meta"),
        HashMap::new(),
        HashMap::new(),
        None,
        auth_type,
        dry_run,
        retry,
    )
    .await
}

fn is_xlsx_file(file_meta_resp: &Value) -> bool {
    let data = file_meta_resp
        .get("data")
        .and_then(|v| v.get("data"))
        .unwrap_or(&Value::Null);
    let name = data
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_lowercase();
    name.ends_with(".xlsx") || name.ends_with(".xls") || name.ends_with(".et")
}

fn effective_auth_type(m: &clap::ArgMatches) -> String {
    if m.get_flag("user-token") {
        "user".to_string()
    } else {
        m.get_one::<String>("auth-type")
            .cloned()
            .unwrap_or_else(|| "user".to_string())
    }
}

fn finalize_read_output(m: &clap::ArgMatches, mut value: Value) -> Result<Value, WpsError> {
    let parquet_meta = maybe_export_parquet(m, &value)?;
    if let Some(meta) = parquet_meta.clone() {
        if let Some(obj) = value
            .get_mut("data")
            .and_then(|v| v.get_mut("data"))
            .and_then(|v| v.as_object_mut())
        {
            obj.insert("parquet_export".to_string(), meta);
        }
    }

    let Some(path) = m.get_one::<String>("output-file") else {
        return Ok(value);
    };

    let output_path = std::path::Path::new(path);
    if let Some(parent) = output_path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent).map_err(|e| {
                WpsError::Validation(format!(
                    "failed to create output directory {}: {e}",
                    parent.display()
                ))
            })?;
        }
    }
    let bytes = serde_json::to_vec_pretty(&value)
        .map_err(|e| WpsError::Validation(format!("failed to serialize read result: {e}")))?;
    std::fs::write(output_path, &bytes).map_err(|e| {
        WpsError::Validation(format!(
            "failed to write output file {}: {e}",
            output_path.display()
        ))
    })?;

    if m.get_flag("output-stdout") {
        let mut with_meta = value;
        if let Some(obj) = with_meta
            .get_mut("data")
            .and_then(|v| v.get_mut("data"))
            .and_then(|v| v.as_object_mut())
        {
            obj.insert(
                "persisted".to_string(),
                serde_json::json!({
                    "output_file": output_path.display().to_string(),
                    "bytes": bytes.len()
                }),
            );
        }
        return Ok(with_meta);
    }

    Ok(serde_json::json!({
        "ok": true,
        "status": 200,
        "data": {
            "code": 0,
            "msg": "ok",
            "data": {
                "persisted_only": true,
                "output_file": output_path.display().to_string(),
                "bytes": bytes.len(),
                "parquet_export": parquet_meta,
                "hint": "使用 --output-stdout 可在写文件同时输出完整结果"
            }
        }
    }))
}

fn maybe_export_parquet(m: &clap::ArgMatches, value: &Value) -> Result<Option<Value>, WpsError> {
    let Some(export_arg) = m.get_one::<String>("export-parquet") else {
        return Ok(None);
    };
    let parquet_path = if export_arg == "auto" {
        let ts = chrono::Local::now().format("%Y%m%d_%H%M%S");
        std::env::temp_dir().join(format!("wpscli_read_{ts}.parquet"))
    } else {
        std::path::PathBuf::from(export_arg)
    };
    let stats = export_read_result_to_parquet(value, &parquet_path)?;
    Ok(Some(serde_json::json!({
        "output_file": parquet_path.display().to_string(),
        "rows": stats.0,
        "columns": stats.1,
        "engine": "polars"
    })))
}

fn export_read_result_to_parquet(value: &Value, output_path: &std::path::Path) -> Result<(usize, usize), WpsError> {
    use polars::prelude::{DataFrame, NamedFrom, ParquetWriter, Series};

    let rows = collect_tabular_rows_for_parquet(value)?;
    if rows.is_empty() {
        return Err(WpsError::Validation(
            "当前结果没有可导出的表格数据".to_string(),
        ));
    }

    let mut keys = BTreeSet::new();
    for row in &rows {
        for key in row.keys() {
            keys.insert(key.clone());
        }
    }
    if keys.is_empty() {
        return Err(WpsError::Validation(
            "当前结果没有可导出的列".to_string(),
        ));
    }

    let mut columns = Vec::new();
    for key in &keys {
        let vals: Vec<Option<String>> = rows
            .iter()
            .map(|row| row.get(key).and_then(json_value_to_string))
            .collect();
        let series = Series::new(key.as_str().into(), vals);
        columns.push(series.into());
    }
    let mut df = DataFrame::new(columns)
        .map_err(|e| WpsError::Execution(format!("polars build dataframe failed: {e}")))?;
    let row_count = df.height();
    let col_count = df.width();

    if let Some(parent) = output_path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent).map_err(|e| {
                WpsError::Validation(format!(
                    "failed to create parquet directory {}: {e}",
                    parent.display()
                ))
            })?;
        }
    }

    let mut file = std::fs::File::create(output_path).map_err(|e| {
        WpsError::Validation(format!(
            "failed to create parquet file {}: {e}",
            output_path.display()
        ))
    })?;
    ParquetWriter::new(&mut file)
        .finish(&mut df)
        .map_err(|e| WpsError::Execution(format!("polars write parquet failed: {e}")))?;
    Ok((row_count, col_count))
}

fn collect_tabular_rows_for_parquet(value: &Value) -> Result<Vec<serde_json::Map<String, Value>>, WpsError> {
    let data = value
        .get("data")
        .and_then(|v| v.get("data"))
        .unwrap_or(&Value::Null);
    let src_format = data
        .get("src_format")
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .to_ascii_lowercase();

    let mut out: Vec<serde_json::Map<String, Value>> = Vec::new();
    if matches!(src_format.as_str(), "xlsx" | "xls" | "et") {
        let sheets = data
            .get("sheets")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();
        for sheet in sheets {
            let sheet_id = sheet.get("sheet_id").cloned().unwrap_or(Value::Null);
            let sheet_name = sheet.get("name").cloned().unwrap_or(Value::Null);
            let table = sheet.get("table").cloned().unwrap_or(Value::Null);
            let header_inferred = table
                .get("header_inferred")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            let header_records = table
                .get("records_with_header")
                .and_then(|v| v.as_array())
                .cloned()
                .unwrap_or_default();
            let records = table
                .get("records")
                .and_then(|v| v.as_array())
                .cloned()
                .unwrap_or_default();
            let selected = if header_inferred && !header_records.is_empty() {
                header_records
            } else {
                records
            };
            for (idx, rec) in selected.into_iter().enumerate() {
                if let Some(obj) = rec.as_object() {
                    let mut row = obj.clone();
                    row.insert("__sheet_id".to_string(), sheet_id.clone());
                    row.insert("__sheet_name".to_string(), sheet_name.clone());
                    row.insert("__row_ordinal".to_string(), serde_json::json!(idx));
                    out.push(row);
                }
            }
        }
    } else if src_format == "dbt" {
        let records = data
            .get("records")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();
        for (idx, rec) in records.into_iter().enumerate() {
            if let Some(obj) = rec.as_object() {
                let mut row = obj.clone();
                row.insert("__row_ordinal".to_string(), serde_json::json!(idx));
                out.push(row);
            }
        }
    }

    if out.is_empty() {
        return Err(WpsError::Validation(
            "--export-parquet 当前仅支持 xlsx/dbt 的表格化读取结果".to_string(),
        ));
    }
    Ok(out)
}

fn json_value_to_string(v: &Value) -> Option<String> {
    match v {
        Value::Null => None,
        Value::String(s) => Some(s.clone()),
        Value::Number(n) => Some(n.to_string()),
        Value::Bool(b) => Some(b.to_string()),
        Value::Array(_) | Value::Object(_) => serde_json::to_string(v).ok(),
    }
}

fn read_text_content(m: &clap::ArgMatches, inline_key: &str, file_key: &str) -> Result<String, WpsError> {
    if let Some(v) = m.get_one::<String>(inline_key) {
        if !v.trim().is_empty() {
            return Ok(v.clone());
        }
    }
    if let Some(path) = m.get_one::<String>(file_key) {
        let txt = std::fs::read_to_string(path)
            .map_err(|e| WpsError::Validation(format!("failed to read {file_key}: {e}")))?;
        if !txt.trim().is_empty() {
            return Ok(txt);
        }
    }
    Err(WpsError::Validation(format!("missing --{inline_key} or --{file_key}")))
}

fn parse_csv_fields(v: Option<&String>) -> Vec<String> {
    v.map(|x| {
        x.split(',')
            .map(|f| f.trim().to_string())
            .filter(|f| !f.is_empty())
            .collect::<Vec<_>>()
    })
    .unwrap_or_default()
}

async fn write_otl_markdown(
    m: &clap::ArgMatches,
    file_id: &str,
    auth_type: &str,
    dry_run: bool,
    retry: u32,
) -> Result<Value, WpsError> {
    let markdown = read_text_content(m, "content", "content-file")?;
    let write_mode = m
        .get_one::<String>("write-mode")
        .cloned()
        .unwrap_or_else(|| "replace".to_string());
    airpage::write_markdown(file_id, &markdown, &write_mode, auth_type, dry_run, retry).await
}

async fn write_dbt_records(
    m: &clap::ArgMatches,
    file_id: &str,
    auth_type: &str,
    dry_run: bool,
    retry: u32,
) -> Result<Value, WpsError> {
    let sheet_id = m
        .get_one::<String>("sheet-id")
        .ok_or_else(|| WpsError::Validation("dbt 写入需要 --sheet-id".to_string()))?;
    let action_raw = m
        .get_one::<String>("db-action")
        .cloned()
        .unwrap_or_else(|| "create".to_string());
    let action = match action_raw.as_str() {
        "create" => dbsheet::DocDbAction::Create,
        "update" => dbsheet::DocDbAction::Update,
        "delete" => dbsheet::DocDbAction::Delete,
        _ => {
            return Err(WpsError::Validation(format!(
                "不支持的 db-action: {action_raw}"
            )))
        }
    };
    let batch_size = *m.get_one::<usize>("db-batch-size").unwrap_or(&100);
    let prefer_id = m.get_flag("db-prefer-id");
    let record_id = m.get_one::<String>("record-id").cloned();
    let payload = read_text_content(m, "records-json", "records-file")?;
    let body_json: Value = serde_json::from_str(&payload)
        .map_err(|e| WpsError::Validation(format!("invalid records json: {e}")))?;

    let result = dbsheet::sql_like_write_records(
        file_id,
        sheet_id,
        action,
        body_json,
        record_id,
        prefer_id,
        batch_size,
        auth_type,
        dry_run,
        retry,
    )
    .await?;
    Ok(serde_json::json!({
        "ok": true,
        "status": 200,
        "data": {
            "code": 0,
            "msg": "ok",
            "data": {
                "target_format": "dbt",
                "action": action.as_str(),
                "file_id": file_id,
                "sheet_id": sheet_id,
                "batch_size": batch_size,
                "prefer_id": prefer_id,
                "result": result
            }
        }
    }))
}

async fn write_xlsx_values(
    m: &clap::ArgMatches,
    file_id: &str,
    auth_type: &str,
    dry_run: bool,
    retry: u32,
) -> Result<Value, WpsError> {
    let payload = read_text_content(m, "xlsx-values-json", "xlsx-values-file")?;
    let values: Value = serde_json::from_str(&payload)
        .map_err(|e| WpsError::Validation(format!("invalid xlsx values json: {e}")))?;

    let worksheet_id = m
        .get_one::<String>("xlsx-sheet-id")
        .map(|v| {
            v.parse::<i64>()
                .map_err(|e| WpsError::Validation(format!("--xlsx-sheet-id 必须是整数: {e}")))
        })
        .transpose()?;
    let row_from = m
        .get_one::<String>("xlsx-row-from")
        .map(|v| {
            v.parse::<i64>()
                .map_err(|e| WpsError::Validation(format!("--xlsx-row-from 必须是整数: {e}")))
        })
        .transpose()?;
    let col_from = m
        .get_one::<String>("xlsx-col-from")
        .map(|v| {
            v.parse::<i64>()
                .map_err(|e| WpsError::Validation(format!("--xlsx-col-from 必须是整数: {e}")))
        })
        .transpose()?;
    let write_mode = m
        .get_one::<String>("xlsx-write-mode")
        .cloned()
        .unwrap_or_else(|| "replace".to_string());

    sheets::write_xlsx_values(
        file_id,
        worksheet_id,
        values,
        row_from,
        col_from,
        &write_mode,
        auth_type,
        dry_run,
        retry,
    )
    .await
}

async fn resolve_doc_ids(
    m: &clap::ArgMatches,
    auth_type: &str,
    dry_run: bool,
    retry: u32,
) -> Result<(String, String), WpsError> {
    if let Some(url) = m.get_one::<String>("url") {
        let v = link_resolver::resolve_share_link(url, auth_type, dry_run, retry).await?;
        let drive_id = v
            .get("drive_id")
            .and_then(|x| x.as_str())
            .ok_or_else(|| WpsError::Validation("resolved link missing drive_id".to_string()))?;
        let file_id = v
            .get("file_id")
            .and_then(|x| x.as_str())
            .ok_or_else(|| WpsError::Validation("resolved link missing file_id".to_string()))?;
        return Ok((drive_id.to_string(), file_id.to_string()));
    }
    let drive_id = m
        .get_one::<String>("drive-id")
        .ok_or_else(|| WpsError::Validation("missing --drive-id (or provide --url)".to_string()))?;
    let file_id = m
        .get_one::<String>("file-id")
        .ok_or_else(|| WpsError::Validation("missing --file-id (or provide --url)".to_string()))?;
    Ok((drive_id.clone(), file_id.clone()))
}
