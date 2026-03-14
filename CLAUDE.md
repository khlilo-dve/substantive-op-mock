cat << 'EOF' > CLAUDE.md
# Claude Code 战术级操作手册 v1.0 (OpenRouter 架构)

## 一、 绝对红线：环境与上下文防御
- **装甲加固**：新建项目必须先配置 `.claudeignore`，严格封杀 `target/`、`Cargo.lock`、`.git/` 和 `.env` 等 Token 黑洞。
- **引擎确认**：启动前使用 `/status` 确认走的是 OpenRouter 通道，绝不盲目发 `hi` 浪费上下文。

## 二、 外科手术式提问法则
- **拒绝无效寒暄**：直接下达结果导向的指令。
- **指定作用域**：精确指出需要读取的 `.rs` 文件（例如：`请读取 src/transaction.rs`），避免全局漫无目的的扫描。

## 三、 内存与弹药管理 (Token 止血)
- **/compact**：完成单一模块重构后，立刻压缩上下文。
- **/clear**：开启全新任务分支前，彻底清空记忆，轻装上阵。
- **/cost**：建立成本直觉，随时监控 100u 额度的燃烧进度。

## 四、 结合 Git 的高频工作流 (颗粒度归档)
1. 下达明确的 Rust 编写/审查指令。
2. 严格审查 AI 修改代码前弹出的 Diff 界面，确认无误按 `Y`。
3. 终端手动运行 `cargo check` 或 `cargo fmt` 验证。
4. 验证通过后，立刻执行 `git add .` 与 `git commit -m "feat/fix: ..."` 进行归档。
5. 若被 AI 带偏或代码改崩，果断输入 `/undo` 撤销，保护本地源码防线。
EOF

echo "CLAUDE.md 操作手册已成功写入当前目录！"