# Op-Sim · 企业操作行为仿真引擎

> 基于 Markov Chain 状态机与剧本 DSL，生成具有真实因果逻辑的合成业务日志。

![Rust](https://img.shields.io/badge/rust-edition%202024-orange?style=flat-square&logo=rust)
![License](https://img.shields.io/badge/license-MIT-green?style=flat-square)

---

## 这是什么

做 IDS/SIEM 规则调试、压力测试或 AI 训练数据集构建时，你需要的不只是随机数据，而是**看起来像真人干的**日志。

两种常见的假数据有明显破绽：

- **纯随机**：时间均匀分布在 24 小时，所有动作等概率，没有任何因果关系
- **固定脚本**：每次运行完全相同，分布方差为零

Op-Sim 的解决方案是**剧本驱动的 Markov Chain 状态机**。你在 YAML 里定义"这家公司的员工会做哪些操作、操作之间如何转移"，引擎负责按照人类生物钟的节律，生成具有真实工作流程的日志序列。

一个真实的员工会话长这样：

```
09:23:15  USER_LOGIN       login_method=sso
09:38:15  VIEW_DASHBOARD   page_load_ms=842
09:51:15  ORDER_SUBMIT     order_id=ORD-67681, amount=922.06, currency=EUR
10:04:15  PAYMENT_CONFIRM  transaction_id=3fdc..., method=alipay
10:19:15  USER_LOGOUT      session_duration_s=3360
```

每条会话从 `LOGIN` 到 `LOGOUT` 闭合，时间戳单调递增，动作之间存在逻辑依赖。

---

## 适用场景

- **IDS/SIEM 训练数据**：为威胁检测系统提供逼真的正常流量基线，训练异常检测模型
- **压力测试**：用符合真实业务分布的请求序列替代 ab/wrk 的均匀压力
- **AI 合成数据**：真实业务日志含有 PII，无法直接用于训练；合成数据保留统计特征但不含敏感信息
- **告警规则验证**：在不动生产数据的前提下，验证"ADMIN_EXPORT 后紧跟 LOGOUT"等异常模式的告警灵敏度

---

## 核心机制

### Markov Chain 状态机

每个 `ActionRule` 配置 `transitions`，定义从当前状态出发，到达下一状态的概率权重：

```
USER_LOGIN ──75%──→ VIEW_DASHBOARD ──35%──→ ORDER_SUBMIT ──80%──→ PAYMENT_CONFIRM
                          │                      │                       │
                         20%                    15%                     35%
                          ↓                      ↓                       ↓
                     USER_LOGOUT           REFUND_REQUEST           USER_LOGOUT
```

`transitions: {}` 空 map 即为终止状态，会话自动结束。

### 生物钟时间陷阱

操作时间不是 0–24 小时均匀分布，而是按时段加权采样：

| 时段 | 类型 | 权重 |
|------|------|------|
| 00–08 | 夜间 | 1 |
| 09–11 | 上午高峰 | 50 |
| 12–13 | 午休 | 5 |
| 14–17 | 下午高峰 | 60 |
| 18–21 | 加班 | 10 |
| 22–23 | 深夜 | 1 |

会话内每次操作间隔 2–30 分钟（模拟阅读屏幕/打字延迟），时间戳单调递增。

### Payload DSL

每个动作携带的业务字段在 YAML 里用一行规格串定义：

| 写法 | 生成内容 |
|------|----------|
| `hex:16` | 16 字节随机十六进制 |
| `float:10.0:9999.0` | 区间浮点，保留两位小数 |
| `int:1:20` | 区间整数 |
| `choice:USD:CNY:EUR` | 随机取一 |
| `order_id` | `ORD-{5位随机数}` |

输出使用 `#[serde(flatten)]`，所有字段平铺在 JSON 顶层，无嵌套，直接兼容 ELK / Loki。

---

## 快速上手

```bash
cargo build --release
```

**编写剧本** (`playbook.yaml`)：

```yaml
company_name: "SilkRoute Global Commerce"
employee_count: 320
ip_subnet: "10.12.8"
date_range:
  start: "2026-03-10"
  end: "2026-03-14"

actions:
  - name: "USER_LOGIN"
    weight: 100             # 唯一入口
    payload_fields:
      session_id: "hex:16"
      login_method: "choice:password:sso:oauth"
    transitions:
      VIEW_DASHBOARD: 75
      USER_LOGOUT: 5

  - name: "ORDER_SUBMIT"
    weight: 0
    payload_fields:
      order_id: "order_id"
      payment_amount: "float:10.0:9999.0"
      currency: "choice:USD:CNY:EUR"
    transitions:
      PAYMENT_CONFIRM: 80
      USER_LOGOUT: 20

  - name: "USER_LOGOUT"
    weight: 0
    payload_fields:
      session_duration_s: "int:60:28800"
    transitions: {}         # 终止状态
```

**运行**：

```bash
# 模拟 200 个员工会话（默认）
./target/release/op-sim

# 自定义参数
./target/release/op-sim --playbook playbook.yaml --count 500 --output logs/march.log
```

**输出样例**（JSONL，每行一条，字段平铺）：

```json
{"timestamp":"2026-03-13 09:23:15","employee_id":"EMP-214","ip_address":"10.12.8.86","action_type":"USER_LOGIN","login_method":"oauth","session_id":"aa445d6584279504"}
{"timestamp":"2026-03-13 09:38:15","employee_id":"EMP-214","ip_address":"10.12.8.86","action_type":"ORDER_SUBMIT","currency":"EUR","order_id":"ORD-67681","payment_amount":922.06}
{"timestamp":"2026-03-13 09:51:15","employee_id":"EMP-214","ip_address":"10.12.8.86","action_type":"USER_LOGOUT","session_duration_s":1680}
```

同一会话的日志共享 `employee_id` 和 `ip_address`，可按这两个字段重建完整操作轨迹。

---

## 项目结构

```
src/
├── main.rs        CLI 入口（clap）
├── models.rs      数据结构：BusinessLog / Playbook / ActionRule
├── generator.rs   状态机引擎 / 时间陷阱 / DSL 解析器
└── writer.rs      BufWriter JSONL 追加写入
playbook.yaml      跨境电商示例剧本
```

---

## 依赖

| crate | 用途 |
|-------|------|
| `serde` + `serde_json` | 序列化 / JSONL 输出 |
| `serde_yaml` | 剧本文件解析 |
| `rand` | 加权随机采样（`WeightedIndex`） |
| `chrono` | 日期时间运算与格式化 |
| `clap` | CLI 参数解析 |
