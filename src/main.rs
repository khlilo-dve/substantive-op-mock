mod generator;
mod models;
mod writer;

use clap::Parser;
use generator::generate_mock_logs;
use models::Playbook;
use writer::flush_logs_to_file;

/// Op-Sim — 企业操作日志仿真引擎
#[derive(Parser)]
#[command(name = "op-sim", version)]
struct Cli {
    /// 剧本文件路径
    #[arg(short, long, default_value = "playbook.yaml")]
    playbook: String,

    /// 生成日志条数
    #[arg(short, long, default_value_t = 1000)]
    count: usize,

    /// 输出文件路径（追加模式）
    #[arg(short, long, default_value = "logs/business_operation.log")]
    output: String,
}

fn load_playbook(path: &str) -> Result<Playbook, Box<dyn std::error::Error>> {
    let content = std::fs::read_to_string(path)?;
    Ok(serde_yaml::from_str(&content)?)
}

fn main() {
    let cli = Cli::parse();

    let playbook = load_playbook(&cli.playbook).unwrap_or_else(|e| {
        eprintln!("\x1b[31m[错误] 无法加载剧本 \"{}\": {e}\x1b[0m", cli.playbook);
        std::process::exit(1);
    });

    eprintln!(
        "剧本: {}  |  员工规模: {}  |  生成: {} 条  →  {}",
        playbook.company_name, playbook.employee_count, cli.count, cli.output
    );

    let logs = generate_mock_logs(&playbook, cli.count);

    match flush_logs_to_file(&logs, &cli.output) {
        Ok(()) => println!(
            "\x1b[32m✓ 成功写入 {} 条日志 → {}\x1b[0m",
            logs.len(),
            cli.output
        ),
        Err(e) => {
            eprintln!("\x1b[31m[错误] 写入失败: {e}\x1b[0m");
            std::process::exit(1);
        }
    }
}
