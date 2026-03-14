# ⚙️ Op-Sim (Operation Simulator)

> An industrial-grade, config-driven business operation simulator built with Rust.
> 专为合规审计、压力测试与高保真数据仿真打造的“企业级数字打工人”引擎。

![Rust](https://img.shields.io/badge/rust-edition%202021-orange?style=flat-square&logo=rust)
![Architecture](https://img.shields.io/badge/architecture-modular-blue?style=flat-square)
![License](https://img.shields.io/badge/license-MIT-green?style=flat-square)

## 🎯 核心愿景 (Vision)

传统的数据伪装与压测脚本往往依赖于劣质的均匀随机数 (`rand()`) 与硬编码逻辑，在专业的数据分析工具面前破绽百出。

**Op-Sim** 摒弃了低级的数据生成方式，全面引入了**“剧本驱动 (Playbook-Driven)”**、**“自定义 DSL 生成器”**与**“生物钟时间陷阱 (Time-Trap)”**。它不仅能模拟千万级的并发业务流，还能精准还原人类在真实商业环境中的概率分布，实现真正意义上的高保真数据仿真。

## 🚀 核心特性 (Core Features)

- 🧬 **剧本驱动的 DSL 引擎**: 告别硬编码。通过 YAML 定义字段级生成规则，原生支持 `hex`, `float`, `int`, `choice`, `order_id` 等多种 DSL 动态载荷注入。
- ⏳ **高保真时间陷阱**: 拒绝均匀分布。算法级模拟打工人“早高峰”、“午休骤降”、“零星加班”的真实生物钟，并支持跨多天 (`date_range`) 的全局随机分布。
- 🌐 **多源 IP 拓扑模拟**: 支持基于 `ip_subnet` (如 `10.12.8.x`) 的末位动态穷举，完美应对审计系统的 IP 来源追溯。
- 📦 **万能日志信封**: 依托 `#[serde(flatten)]` 宏，无论业务字段如何变幻，输出始终保持为干净、平铺的单层 JSONL 结构，无缝对接 ELK / 阿里云 SLS。
- ⚡ **HPC 级落盘性能**: 采用 `BufWriter` 内存缓冲刷盘机制，避免海量并发下的 `write` 系统调用退化。

## 🏗️ 架构设计 (Architecture)

系统采用现代 Rust 工程标准，实现了绝对的职责解耦：
- `src/main.rs`: 纯粹的 CLI 入口与生命周期路由。
- `src/models.rs`: 核心数据结构与序列化契约。
- `src/generator.rs`: 包含 DSL 引擎、状态机游走与时间陷阱的算法核心。
- `src/writer.rs`: 负责高性能的 I/O 缓冲落盘。

## 💻 极速上手 (Quick Start)

### 1. 编译构建
采用 release 模式榨干硬件性能：
\`\`\`bash
cargo build --release
\`\`\`

### 2. 编写剧本 (playbook.yaml)
利用强大的 DSL 引擎，定义你的业务逻辑：
\`\`\`yaml
company_name: "跨境电商模拟节点"
employee_count: 50
ip_subnet: "10.12.8"
date_range: 
  start: "2026-03-10"
  end: "2026-03-15"

actions:
  - name: "ORDER_SUBMIT"
    weight: 80
    payload_schema:
      order_id: { type: "order_id" }
      payment_amount: { type: "float", min: 10.0, max: 999.9 }
      currency: { type: "choice", values: ["USD", "EUR", "CNY"] }
      session_hash: { type: "hex", length: 16 }
\`\`\`

### 3. CLI 自动化调用
通过 `clap` 集成了优雅的命令行参数，完美支持 CI/CD 脚本化运行：
\`\`\`bash
./target/release/op-sim -p playbook.yaml -c 5000 -o logs/output.log
\`\`\`
*参数说明：-p (指定剧本路径), -c (生成日志条数), -o (输出文件路径)*

## 📄 输出样例 (Output Example)

引擎将输出绝对平铺的 JSONL，所有 DSL 字段完美展平：
\`\`\`json
{"timestamp":"2026-03-11 09:42:09","employee_id":"EMP-005","ip_address":"10.12.8.54","action_type":"ORDER_SUBMIT","currency":"EUR","order_id":"ORD-67681","payment_amount":922.06,"session_hash":"a7f9c2b4e1d08a5f"}
\`\`\`

---
*Built with strict memory safety and zero-cost abstractions.*
