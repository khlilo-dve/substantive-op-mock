# Op-Sim · 操作日志仿真引擎

> 一个用来骗过审计系统的 Rust 小引擎——当然，是合法地骗。

![Rust](https://img.shields.io/badge/rust-edition%202024-orange?style=flat-square&logo=rust)
![License](https://img.shields.io/badge/license-MIT-green?style=flat-square)

---

## 这是什么

做合规审计、压测或者 SIEM 规则调试的时候，你总得有一批"看起来像真人干的"日志。

用 `random()` 生成的那种，一眼假：时间戳均匀分布在 24 小时、IP 永远同一个、所有动作概率相等。真实业务数据长什么样？上午九点到十一点最忙、午饭时间骤降、下午两点到五点再来一波、偶尔有人加班到晚上八点。

Op-Sim 就是干这个的。你写一份 YAML 剧本，描述"这家公司有多少人、会做哪些操作、每种操作发生的概率"，引擎负责把仿真日志按照人类生物钟的规律吐出来。

---

## 核心能力

**剧本驱动，改 YAML 不改代码**

每种业务动作的字段、类型、权重，全部在 `playbook.yaml` 里定义。换一家公司的模拟场景，换一份 YAML 就行了。

**时间陷阱**

不用均匀随机。小时粒度加权采样，上午/下午工作时段权重高，午休权重低，深夜接近零。5000 条日志跑下来，分布和真实系统的审计日志几乎没有视觉差异。

**动态 Payload DSL**

在 YAML 里给每个动作配 `payload_fields`，支持这些生成规则：

| 规格写法 | 生成内容 |
|----------|----------|
| `hex:16` | 16 字节的随机十六进制字符串 |
| `float:10.0:9999.0` | 区间内随机浮点，保留两位小数 |
| `int:1:20` | 区间内随机整数 |
| `choice:USD:CNY:EUR` | 随机从选项里取一个 |
| `order_id` | `ORD-{5位随机数}` 格式 |

**`#[serde(flatten)]` 平铺结构**

输出的每一行 JSON，动态字段和固定字段都在同一层，没有嵌套。直接扔给 ELK 或阿里云 SLS 不需要任何预处理。

---

## 快速上手

**构建**

```bash
cargo build --release
```

**写剧本** (`playbook.yaml`)

```yaml
company_name: "SilkRoute Global Commerce"
employee_count: 320
ip_subnet: "10.12.8"
date_range:
  start: "2026-03-10"
  end: "2026-03-14"

actions:
  - name: "USER_LOGIN"
    weight: 50
    payload_fields:
      session_id: "hex:16"
      login_method: "choice:password:sso:oauth"

  - name: "ORDER_SUBMIT"
    weight: 30
    payload_fields:
      order_id: "order_id"
      payment_amount: "float:10.0:9999.0"
      currency: "choice:USD:CNY:EUR"
      item_count: "int:1:20"

  - name: "ADMIN_EXPORT"
    weight: 2
    payload_fields:
      export_format: "choice:csv:xlsx:json"
      record_count: "int:100:50000"
```

**运行**

```bash
# 默认：读 playbook.yaml，生成 1000 条，写入 logs/business_operation.log
./target/release/op-sim

# 自定义参数
./target/release/op-sim --playbook playbook.yaml --count 5000 --output logs/march.log
```

**输出样例**

每行一条干净的 JSONL，动态字段直接平铺：

```json
{"timestamp":"2026-03-11 09:42:09","employee_id":"EMP-005","ip_address":"10.12.8.54","action_type":"ORDER_SUBMIT","currency":"EUR","item_count":11,"order_id":"ORD-67681","payment_amount":922.06}
{"timestamp":"2026-03-13 14:28:33","employee_id":"EMP-214","ip_address":"10.12.8.203","action_type":"USER_LOGIN","login_method":"sso","session_id":"3fdcd5bcef313d8f"}
```

---

## 项目结构

```
src/
├── main.rs        CLI 入口，参数解析
├── models.rs      数据结构：BusinessLog / Playbook / ActionRule
├── generator.rs   核心算法：时间陷阱 / IP 生成 / DSL 解析 / 日志组装
└── writer.rs      BufWriter 追加写入 JSONL
playbook.yaml      跨境电商示例剧本
```

---

## 适用场景

- 审计平台、SOC/SIEM 系统的规则调试和告警测试
- 合规审查前的样本数据准备
- 数据分析模型的冷启动，需要有分布合理的历史数据
- 压测时需要"像真人一样的"请求序列

---

## 依赖

| crate | 用途 |
|-------|------|
| `serde` + `serde_json` | 序列化 / JSONL 输出 |
| `serde_yaml` | 剧本文件解析 |
| `rand` | 加权随机采样 (`WeightedIndex`) |
| `chrono` | 日期处理与格式化 |
| `clap` | CLI 参数解析 |
