use chrono::{Duration, NaiveDate};
use rand::distributions::{Distribution, Uniform, WeightedIndex};
use rand::Rng;
use serde_json::{Map, Value};

use crate::models::{ActionRule, BusinessLog, DateRange, Playbook};

// ── 时间陷阱：工作日生物钟权重段 ─────────────────────────────────────────────
// (起始小时, 段内小时数, 权重)
const SEGMENTS: &[(u32, u32, u32)] = &[
    (0, 9, 1),   // 00–08  夜间休眠
    (9, 3, 50),  // 09–11  上午高峰
    (12, 2, 5),  // 12–13  午休摸鱼
    (14, 4, 60), // 14–17  下午高峰
    (18, 4, 10), // 18–21  零星加班
    (22, 2, 1),  // 22–23  深夜
];

fn generate_realistic_timestamp(date: &str, rng: &mut impl Rng) -> String {
    let weights: Vec<u32> = SEGMENTS.iter().map(|&(_, _, w)| w).collect();
    let seg_dist = WeightedIndex::new(&weights).unwrap();
    let (seg_start, seg_len, _) = SEGMENTS[seg_dist.sample(rng)];

    let hour = seg_start + rng.sample(Uniform::new(0, seg_len));
    let minute: u32 = rng.gen_range(0..60);
    let second: u32 = rng.gen_range(0..60);

    NaiveDate::parse_from_str(date, "%Y-%m-%d")
        .expect("date 格式应为 YYYY-MM-DD")
        .format(&format!("%Y-%m-%d {hour:02}:{minute:02}:{second:02}"))
        .to_string()
}

// ── 日期区间：随机选取区间内某天 ─────────────────────────────────────────────

fn pick_date(date_range: &Option<DateRange>, rng: &mut impl Rng) -> String {
    match date_range {
        None => "2026-03-14".to_string(),
        Some(dr) => {
            let start = NaiveDate::parse_from_str(&dr.start, "%Y-%m-%d").unwrap();
            let end = NaiveDate::parse_from_str(&dr.end, "%Y-%m-%d").unwrap();
            let span = (end - start).num_days().max(0) as u32;
            let offset = rng.gen_range(0..=span);
            (start + Duration::days(offset as i64))
                .format("%Y-%m-%d")
                .to_string()
        }
    }
}

// ── IP 生成：基于子网末位随机 ─────────────────────────────────────────────────

fn generate_ip(subnet: &str, rng: &mut impl Rng) -> String {
    let last: u8 = rng.gen_range(1..=254);
    format!("{subnet}.{last}")
}

// ── Payload DSL 解析器 ────────────────────────────────────────────────────────
//
// 支持规格串:
//   hex:N              → N 字节的十六进制字符串（2N 字符）
//   float:min:max      → [min, max] 范围内的浮点数，保留两位小数
//   int:min:max        → [min, max] 范围内的整数
//   choice:a:b:c       → 从选项中随机取一
//   order_id           → "ORD-{5位随机数}" 格式订单号
//   <其他字面量>        → 原样作为字符串输出

fn generate_payload_value(spec: &str, rng: &mut impl Rng) -> Value {
    if let Some(rest) = spec.strip_prefix("hex:") {
        let n: usize = rest.parse().unwrap_or(8);
        let hex: String = (0..n).map(|_| format!("{:02x}", rng.gen_range(0u8..=255))).collect();
        Value::String(hex)
    } else if let Some(rest) = spec.strip_prefix("float:") {
        let mut parts = rest.splitn(2, ':');
        let min: f64 = parts.next().and_then(|s| s.parse().ok()).unwrap_or(0.0);
        let max: f64 = parts.next().and_then(|s| s.parse().ok()).unwrap_or(100.0);
        let val = (rng.gen_range(min..=max) * 100.0).round() / 100.0;
        Value::Number(serde_json::Number::from_f64(val).unwrap())
    } else if let Some(rest) = spec.strip_prefix("int:") {
        let mut parts = rest.splitn(2, ':');
        let min: i64 = parts.next().and_then(|s| s.parse().ok()).unwrap_or(0);
        let max: i64 = parts.next().and_then(|s| s.parse().ok()).unwrap_or(100);
        Value::Number(rng.gen_range(min..=max).into())
    } else if let Some(rest) = spec.strip_prefix("choice:") {
        let options: Vec<&str> = rest.split(':').collect();
        let idx = rng.gen_range(0..options.len());
        Value::String(options[idx].to_string())
    } else if spec == "order_id" {
        let n: u32 = rng.gen_range(10000..99999);
        Value::String(format!("ORD-{n}"))
    } else {
        Value::String(spec.to_string())
    }
}

fn build_payload(action: &ActionRule, rng: &mut impl Rng) -> Value {
    if action.payload_fields.is_empty() {
        return Value::Object(Map::new());
    }
    let mut map = Map::new();
    // 按 key 排序保证序列化顺序一致（审计友好）
    let mut fields: Vec<(&String, &String)> = action.payload_fields.iter().collect();
    fields.sort_by_key(|(k, _)| k.as_str());
    for (key, spec) in fields {
        map.insert(key.clone(), generate_payload_value(spec, rng));
    }
    Value::Object(map)
}

// ── 核心生成函数 ──────────────────────────────────────────────────────────────

pub fn generate_mock_logs(playbook: &Playbook, count: usize) -> Vec<BusinessLog> {
    let employee_pool: Vec<String> = (1..=playbook.employee_count)
        .map(|n| format!("EMP-{n:03}"))
        .collect();

    let action_weights: Vec<u32> = playbook.actions.iter().map(|a| a.weight).collect();
    let action_dist = WeightedIndex::new(&action_weights).expect("actions 权重列表不能为空");
    let emp_dist = Uniform::new(0, employee_pool.len());
    let mut rng = rand::thread_rng();

    (0..count)
        .map(|_| {
            let action = &playbook.actions[action_dist.sample(&mut rng)];
            BusinessLog {
                timestamp: generate_realistic_timestamp(
                    &pick_date(&playbook.date_range, &mut rng),
                    &mut rng,
                ),
                employee_id: employee_pool[emp_dist.sample(&mut rng)].clone(),
                ip_address: generate_ip(&playbook.ip_subnet, &mut rng),
                action_type: action.name.clone(),
                payload: build_payload(action, &mut rng),
            }
        })
        .collect()
}
