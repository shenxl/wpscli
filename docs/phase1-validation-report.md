# 阶段1验证报告（基础先行）

更新时间：2026-03-14

本报告对应“先打基础再进阶段2”的目标复盘，聚焦三件事：

1. `-h` 与 `schema` 信息完备与一致
2. 66 个 service / 全量 endpoint 的质量门禁可执行
3. 质量观测可在 `doctor` 中稳定暴露

---

## 1) 执行的验证项

### 1.1 编译与单测

```bash
cargo check
cargo test --quiet
```

结果：通过。

### 1.2 全量质量门禁

```bash
wpscli quality --output json
```

核心结果：

- `service_coverage`: `66 / 66`
- `endpoint_coverage.total`: `493`
- `help_schema_consistent`: `493`
- `dry_run_passed`: `493`
- `descriptor_static.error_count`: `0`
- `descriptor_static.warning_count`: `0`
- `score`: `100`
- `top_domains.coverage_ratio`: `1.0`
- `top_domains.domain_count`: `8`

说明：默认执行中连通抽样门禁未启用（`connectivity_sample=0`），因此 `connectivity` 为 `skipped`，属预期行为。

### 1.3 诊断可观测

```bash
wpscli doctor --output json
```

`doctor.quality_probe` 结果：

- `descriptor_health_score`: `100`
- `descriptor_error_count`: `0`
- `descriptor_warning_count`: `0`
- `help_schema_inconsistent_endpoints`: `0`
- `red_lines`: 全部 `triggered=false`

### 1.4 AI 机器索引可用性

```bash
wpscli catalog --mode ai --output compact
```

结果：

- `total_services`: `66`
- `total_endpoints`: `493`
- `contract_version`: `1.0.0`

---

## 2) 目标达成评估

### 2.1 信息完备（目标1）

结论：已达成。

- 统一合同版本：`help_schema_contract` `1.0.0`
- `catalog --mode ai` 可输出机器可读索引
- `schema --mode invoke` 与动态命令构建共享同源信息模型

### 2.2 质量门禁（目标2）

结论：已达成。

- 静态门禁：descriptor 完整性校验通过
- 构造门禁：493 个 endpoint dry-run 全通过
- 一致性门禁：493 个 endpoint help/schema 一致
- 连通抽样门禁：能力已就绪（按需启用）

### 2.3 可观测（目标3）

结论：已达成。

- `doctor` 新增 `quality_probe`
- 包含健康分、红线项、能力域覆盖摘要
- 可用于发布前快速阻断检查

---

## 3) 风险与备注

- 连通抽样门禁依赖运行环境鉴权与网络；默认关闭以保证基线验证可重复。
- 对真实 API 可用性回归，建议在 CI/CD 或预发环境按白名单启用：
  - `wpscli quality --connectivity-sample <N> --connectivity-auth auto`

---

## 4) 阶段2前置条件结论

当前基础满足进入阶段2：

- `-h/schema` 合同稳定
- 66 service 全量覆盖并可自动校验
- 质量与红线可观测
- 顶层能力域（8 domains）完成全覆盖映射

