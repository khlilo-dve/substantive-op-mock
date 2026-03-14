use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

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
    println!("--- 场景 B：Web3 科技公司 ---\n{b}");
}
