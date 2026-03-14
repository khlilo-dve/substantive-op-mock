use std::io::Write;
use std::path::Path;

use crate::models::BusinessLog;

pub fn flush_logs_to_file(logs: &[BusinessLog], path: &str) -> std::io::Result<()> {
    // 确保父目录存在（幂等）
    if let Some(parent) = Path::new(path).parent() {
        std::fs::create_dir_all(parent)?;
    }

    let file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)?;
    let mut writer = std::io::BufWriter::new(file);

    for log in logs {
        let line = serde_json::to_string(log).expect("BusinessLog 序列化不应失败");
        writer.write_all(line.as_bytes())?;
        writer.write_all(b"\n")?;
    }

    Ok(())
}
