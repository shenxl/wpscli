# 文档助手命令

> 此文件由 `scripts/generate_docs.py` 自动生成，请勿手工编辑。

`doc` 文档读写与链接解析能力。

---

## doc

```bash
wpscli doc --help
```

```text
文档读写助手（链接解析/读取/写入/检索）

Usage: doc [COMMAND]

Commands:
  resolve-link  解析分享链接，获取 drive_id/file_id
  read-doc      读取文档内容（支持 URL 或 drive/file_id）
  write-doc     按文件类型写入内容（otl/dbt/xlsx）
  file-info     读取文件元信息
  list-files    列出子文件/子目录
  search        按关键字搜索文件
  help          Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help

示例：
  wpscli doc read-doc --url "https://365.kdocs.cn/l/xxxx" --user-token
  wpscli doc write-doc --url "https://365.kdocs.cn/l/xxxx" --target-format otl --content "# 标题" --user-token
  wpscli doc write-doc --url "https://365.kdocs.cn/l/xxxx" --target-format xlsx --xlsx-values-json '[["标题","值"],["A",1]]' --user-token
```

## doc read-doc

```bash
wpscli doc read-doc --help
```

```text
读取文档内容（支持 URL 或 drive/file_id）

Usage: doc read-doc [OPTIONS]

Options:
      --url <url>
          文档分享链接
      --drive-id <drive-id>
          网盘 ID（与 --file-id 搭配）
      --file-id <file-id>
          文件 ID
      --format <format>
          输出格式 [default: markdown] [possible values: markdown, plain, kdc]
      --mode <mode>
          读取模式：sync/async/auto [default: sync] [possible values: sync, async, auto]
      --task-id <task-id>
          异步任务 ID（续跑时使用）
      --include-elements <include-elements>
          元素过滤，逗号分隔：para,table,component,textbox,all
      --auth-type <auth-type>
          鉴权类型：app 或 user [default: user] [possible values: app, user]
      --user-token
          快捷方式：等价于 --auth-type user
      --dry-run
          
      --retry <retry>
          [default: 1]
      --poll-interval-ms <poll-interval-ms>
          异步任务轮询间隔（毫秒） [default: 1500]
      --max-wait-seconds <max-wait-seconds>
          异步任务最大等待时长（秒） [default: 120]
      --xlsx-max-sheets <xlsx-max-sheets>
          xlsx 最多读取工作表数量（兼容参数，等价于 --xlsx-sheet-head） [default: 10]
      --xlsx-max-rows <xlsx-max-rows>
          xlsx 每个工作表最多读取行数（兼容参数，等价于 --xlsx-row-head） [default: 200]
      --xlsx-max-cols <xlsx-max-cols>
          xlsx 每个工作表最多读取列数（兼容参数，等价于 --xlsx-col-head） [default: 50]
      --xlsx-sheet-offset <xlsx-sheet-offset>
          xlsx 读取工作表偏移量（从第几个工作表开始） [default: 0]
      --xlsx-sheet-head <xlsx-sheet-head>
          xlsx 读取工作表数量上限（优先于 --xlsx-max-sheets）
      --xlsx-row-offset <xlsx-row-offset>
          xlsx 每个工作表行偏移量（从 active_area 的第几行开始） [default: 0]
      --xlsx-row-head <xlsx-row-head>
          xlsx 每个工作表读取行数上限（优先于 --xlsx-max-rows）
      --xlsx-col-offset <xlsx-col-offset>
          xlsx 每个工作表列偏移量（从 active_area 的第几列开始） [default: 0]
      --xlsx-col-head <xlsx-col-head>
          xlsx 每个工作表读取列数上限（优先于 --xlsx-max-cols）
      --output-file <output-file>
          将 read-doc 结果写入指定 JSON 文件（大数据推荐）
      --export-parquet [<export-parquet>]
          将表格结果导出为 Parquet（可选路径；不传值则写入系统临时目录）
      --output-stdout
          配合 --output-file 使用：写文件后仍输出完整结果到 stdout
      --dbsheet-sheet-id <dbsheet-sheet-id>
          dbt 读取时指定 sheet_id（可选）
      --dbsheet-where <dbsheet-where>
          dbt 读取过滤条件（SQL-like）
      --dbsheet-fields <dbsheet-fields>
          dbt 读取字段投影，逗号分隔
      --dbsheet-limit <dbsheet-limit>
          dbt 读取返回条数上限 [default: 100]
      --dbsheet-offset <dbsheet-offset>
          dbt 读取偏移量 [default: 0]
  -h, --help
          Print help

示例：
  wpscli doc read-doc --url "https://365.kdocs.cn/l/xxxx" --format markdown --mode auto --user-token
  wpscli doc read-doc --drive-id <drive_id> --file-id <file_id> --user-token
  wpscli doc read-doc --url "https://365.kdocs.cn/l/xxxx" --xlsx-row-offset 500 --xlsx-row-head 200 --output-file /tmp/xlsx_page_3.json --user-token
  wpscli doc read-doc --url "https://365.kdocs.cn/l/xxxx" --xlsx-row-head 1000 --export-parquet /tmp/xlsx.parquet --user-token
```

## doc write-doc

```bash
wpscli doc write-doc --help
```

```text
按文件类型写入内容（otl/dbt/xlsx）

Usage: doc write-doc [OPTIONS]

Options:
      --url <url>
          文档分享链接
      --drive-id <drive-id>
          网盘 ID（与 --file-id 搭配）
      --file-id <file-id>
          文件 ID
      --target-format <target-format>
          目标格式（默认自动识别） [default: auto] [possible values: auto, otl, dbt, xlsx]
      --write-mode <write-mode>
          OTL 写入模式：replace 或 append [default: replace] [possible values: replace, append]
      --content <content>
          OTL 写入内容（Markdown）
      --content-file <content-file>
          从文件读取 OTL Markdown 内容
      --db-action <db-action>
          DBT 操作类型 [default: create] [possible values: create, update, delete]
      --sheet-id <sheet-id>
          DBT 必填：工作表 ID
      --records-json <records-json>
          DBT 记录 JSON 字符串
      --records-file <records-file>
          从文件读取 DBT 记录 JSON
      --record-id <record-id>
          DBT update 单条记录 ID
      --db-batch-size <db-batch-size>
          DBT 批量写入大小 [default: 100]
      --db-prefer-id
          DBT 写入优先使用 id 字段映射
      --xlsx-sheet-id <xlsx-sheet-id>
          XLSX 写入目标 worksheet_id（可选）
      --xlsx-row-from <xlsx-row-from>
          XLSX 写入起始行（默认 0；append 模式自动追加）
      --xlsx-col-from <xlsx-col-from>
          XLSX 写入起始列（默认取工作表 active_area.col_from）
      --xlsx-write-mode <xlsx-write-mode>
          XLSX 写入模式：replace/append [default: replace] [possible values: replace, append]
      --xlsx-values-json <xlsx-values-json>
          XLSX 写入二维数组 JSON
      --xlsx-values-file <xlsx-values-file>
          从文件读取 XLSX 写入 JSON
      --auth-type <auth-type>
          鉴权类型：app 或 user [default: user] [possible values: app, user]
      --user-token
          快捷方式：等价于 --auth-type user
      --dry-run
          
      --retry <retry>
          [default: 1]
  -h, --help
          Print help

示例：
  wpscli doc write-doc --url "https://365.kdocs.cn/l/xxxx" --target-format otl --content "# 周报" --user-token
  wpscli doc write-doc --file-id <file_id> --target-format dbt --db-action create --sheet-id 2 --records-json '[{"热点标题":"A"}]' --user-token
  wpscli doc write-doc --file-id <file_id> --target-format xlsx --xlsx-sheet-id 0 --xlsx-values-json '[["项目","状态"],["M1","完成"]]' --user-token
```
