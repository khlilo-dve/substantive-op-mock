use rand::distributions::{Distribution, WeightedIndex};
use rand::thread_rng;
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

fn generate_mock_logs(playbook: &Playbook, count: usize) -> Vec<BusinessLog> {
    let weights: Vec<u32> = playbook.actions.iter().map(|a| a.weight).collect();
    let dist = WeightedIndex::new(&weights).expect("权重列表不能为空");
    let mut rng = thread_rng();

    (0..count)
        .map(|_| {
            let action = &playbook.actions[dist.sample(&mut rng)];
            BusinessLog {
                timestamp: "2026-03-14T10:00:00Z".to_string(),
                employee_id: "EMP-001".to_string(),
                ip_address: "192.168.1.1".to_string(),
                action_type: action.name.clone(),
                payload: json!({}),
            }
        })
        .collect()
}

fn load_playbook(path: &str) -> Result<Playbook, Box<dyn std::error::Error>> {
    let content = std::fs::read_to_string(path)?;
    let playbook = serde_yaml::from_str(&content)?;
    Ok(playbook)
}

#[derive(Debug, Serialize, Deserialize)]
struct BusinessLog {
    timestamp: String,
    employee_id: String,
    ip_address: String,
    action_type: String,
    #[serde(flatten)]
    payload: Value,
}

fn main() {
    // 场景 A：跨境电商
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

    // 场景 B：Web3 科技公司
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

    let a = serde_json::to_string_pretty(&ecommerce_log).unwrap();
    let b = serde_json::to_string_pretty(&web3_log).unwrap();

    println!("--- 场景 A：跨境电商 ---\n{a}\n");
    println!("--- 场景 B：Web3 科技公司 ---\n{b}\n");

    // ── 剧本加载 + 采样验证 ────────────────────────────────────────────────────
    let playbook = load_playbook("playbook.yaml").expect("无法加载 playbook.yaml");
    let logs = generate_mock_logs(&playbook, 10);

    println!("--- 采样日志（10 条）---");
    for (i, log) in logs.iter().enumerate() {
        println!("[{:02}] {}", i + 1, serde_json::to_string(log).unwrap());
    }
}
