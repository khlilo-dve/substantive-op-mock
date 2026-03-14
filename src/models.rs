use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct BusinessLog {
    pub timestamp: String,
    pub employee_id: String,
    pub ip_address: String,
    pub action_type: String,
    #[serde(flatten)]
    pub payload: Value,
}

#[derive(Debug, Deserialize)]
pub struct DateRange {
    pub start: String,
    pub end: String,
}

#[derive(Debug, Deserialize)]
pub struct ActionRule {
    pub name: String,
    pub weight: u32,
    /// 每个字段 → DSL 规格串，例如 "hex:16" / "float:10.0:9999.0"
    #[serde(default)]
    pub payload_fields: HashMap<String, String>,
}

#[derive(Debug, Deserialize)]
pub struct Playbook {
    pub company_name: String,
    pub employee_count: u32,
    /// 内网子网前缀，如 "10.12.8"，末位由生成器随机填充
    #[serde(default = "default_subnet")]
    pub ip_subnet: String,
    /// 可选：日志日期区间；缺省时使用固定日期
    pub date_range: Option<DateRange>,
    pub actions: Vec<ActionRule>,
}

fn default_subnet() -> String {
    "192.168.1".to_string()
}
