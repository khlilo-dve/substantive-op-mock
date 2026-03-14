use chrono::NaiveDate;
use rand::distributions::{Distribution, Uniform, WeightedIndex};
use rand::Rng;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

// ── Playbook 剧本结构 ─────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct ActionRule {
    name: String,
    weight: u32,
}

#[derive(Debug, Deserialize)]
struct Playbook {
    company_name: String,
    employee_count: u32,
    actions: Vec<ActionRule>,
}

// ── 核心数据结构 ──────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
struct BusinessLog {
    timestamp: String,
    employee_id: String,
    ip_address: String,
    action_type: String,
    #[serde(flatten)]
    payload: Value,
}

// ── 时间陷阱算法：工作日人类生物钟 ───────────────────────────────────────────
//
// 以"小时段"为粒度做 WeightedIndex，映射关系：
//   segment 0  →  00–08  (9h)  夜间休眠，weight = 1
//   segment 1  →  09–11  (3h)  上午高峰，weight = 50
//   segment 2  →  12–13  (2h)  午休摸鱼，weight = 5
//   segment 3  →  14–17  (4h)  下午高峰，weight = 60
//   segment 4  →  18–21  (4h)  零星加班，weight = 10
//   segment 5  →  22–23  (2h)  深夜，    weight = 1
//
// 确定段后，在段内均匀采样具体小时，再均匀采样分钟/秒。

fn generate_realistic_timestamp(base_date: &str, rng: &mut impl Rng) -> String {
    // 每段：(起始小时, 段内小时数, 权重)
    const SEGMENTS: &[(u32, u32, u32)] = &[
        (0, 9, 1),   // 00:00–08:59  夜间
        (9, 3, 50),  // 09:00–11:59  上午高峰
        (12, 2, 5),  // 12:00–13:59  午休
        (14, 4, 60), // 14:00–17:59  下午高峰
        (18, 4, 10), // 18:00–21:59  加班
        (22, 2, 1),  // 22:00–23:59  深夜
    ];

    let weights: Vec<u32> = SEGMENTS.iter().map(|&(_, _, w)| w).collect();
    let seg_dist = WeightedIndex::new(&weights).unwrap();
    let (seg_start, seg_len, _) = SEGMENTS[seg_dist.sample(rng)];

    let hour = seg_start + rng.sample(Uniform::new(0, seg_len));
    let minute: u32 = rng.sample(Uniform::new(0, 60));
    let second: u32 = rng.sample(Uniform::new(0, 60));

    // 验证 base_date 合法性，panic 属预期外输入
    NaiveDate::parse_from_str(base_date, "%Y-%m-%d")
        .expect("base_date 格式应为 YYYY-MM-DD")
        .format(&format!("%Y-%m-%d {hour:02}:{minute:02}:{second:02}"))
        .to_string()
}

// ── 核心生成函数 ──────────────────────────────────────────────────────────────

fn generate_mock_logs(playbook: &Playbook, count: usize) -> Vec<BusinessLog> {
    // 动态员工池：EMP-001 … EMP-{employee_count}
    let employee_pool: Vec<String> = (1..=playbook.employee_count)
        .map(|n| format!("EMP-{n:03}"))
        .collect();

    let action_weights: Vec<u32> = playbook.actions.iter().map(|a| a.weight).collect();
    let action_dist = WeightedIndex::new(&action_weights).expect("权重列表不能为空");
    let emp_dist = Uniform::new(0, employee_pool.len());
    let mut rng = rand::thread_rng();

    println!("剧本加载：{} / 员工规模：{}", playbook.company_name, playbook.employee_count);

    (0..count)
        .map(|_| {
            let action = &playbook.actions[action_dist.sample(&mut rng)];
            let emp_id = employee_pool[emp_dist.sample(&mut rng)].clone();
            let ts = generate_realistic_timestamp("2026-03-14", &mut rng);

            BusinessLog {
                timestamp: ts,
                employee_id: emp_id,
                ip_address: "192.168.1.1".to_string(),
                action_type: action.name.clone(),
                payload: json!({}),
            }
        })
        .collect()
}

// ── Playbook 加载 ─────────────────────────────────────────────────────────────

fn load_playbook(path: &str) -> Result<Playbook, Box<dyn std::error::Error>> {
    let content = std::fs::read_to_string(path)?;
    let playbook = serde_yaml::from_str(&content)?;
    Ok(playbook)
}

// ── 入口 ──────────────────────────────────────────────────────────────────────

fn main() {
    // 场景 A：跨境电商（阶段一留存）
    let ecommerce_log = BusinessLog {
        timestamp: "2026-03-14T08:00:00Z".to_string(),
        employee_id: "EMP-1024".to_string(),
        ip_address: "203.0.113.42".to_string(),
        action_type: "ORDER_SUBMIT".to_string(),
        payload: json!({
            "order_id": "ORD-20260314-8848",
            "payment_amount": 2999.00
        }),
    };

    // 场景 B：Web3 科技公司（阶段一留存）
    let web3_log = BusinessLog {
        timestamp: "2026-03-14T09:15:33Z".to_string(),
        employee_id: "EMP-0007".to_string(),
        ip_address: "10.0.0.7".to_string(),
        action_type: "CODE_PUSH".to_string(),
        payload: json!({
            "git_commit_hash": "a3f5c2d9e1b84760f2e3c4d5a6b7c8d9e0f1a2b3",
            "api_latency_ms": 42
        }),
    };

    println!("--- 场景 A：跨境电商 ---");
    println!("{}\n", serde_json::to_string_pretty(&ecommerce_log).unwrap());
    println!("--- 场景 B：Web3 科技公司 ---");
    println!("{}\n", serde_json::to_string_pretty(&web3_log).unwrap());

    // ── 剧本驱动生成 ──────────────────────────────────────────────────────────
    let playbook = load_playbook("playbook.yaml").expect("无法加载 playbook.yaml");
    let logs = generate_mock_logs(&playbook, 20);

    println!("\n--- 仿真日志（20 条）---");
    for (i, log) in logs.iter().enumerate() {
        println!("[{:02}] {}", i + 1, serde_json::to_string(log).unwrap());
    }
}
