服务器内置 Model Context Protocol 宿主，Web UI 的 `MCP` 区块用于配置它：Code Agent（Claude Code、Codex、Kimi Code、Pi、OpenCode、Qwen Code、DeepSeek Harness 等）可以从它们自己的会话中直接连接并操作你的机器 —— 读取和驱动终端，甚至操作 EasyEDA Pro。

## 服务器设置

启用 MCP 服务器并配置：

- `Port`（默认 `6180`）与 `Bind Address` —— `127.0.0.1` 仅限本机访问；`0.0.0.0` 对网络开放；
- `Access Token` —— 非本机请求必须携带的 Bearer 令牌；来自 localhost 的请求始终放行。令牌永不为空 —— 全新安装会生成随机令牌；骰子按钮可重新生成，复制按钮可将其取出；
- `Stream Throttle` —— 向已连接 agent 推送屏幕流的最小间隔。

更改在 `Save` 后生效；服务器热重载，无需重启。

## 端点

每个能力集都有自己的 URL：`http://localhost:{port}/{id}/mcp`：

- **Terminal MCP**（`/terminal/mcp`）—— 内置终端能力。Agent 可以列出终端、读取可见屏幕与回滚历史、输入原始文本与按键、调整终端尺寸、等待提示符。工具调用通过 Terminal ID 定位终端 —— 在终端上右键 → `MCP` → `Copy Terminal ID`，把 ID 粘贴到 agent 的上下文中（详见“终端”章节的 Terminal MCP）；
- **EasyEDA Pro MCP**（`/easyeda/mcp`）—— 内置 MCP 外挂，将 EasyEDA Pro（嘉立创EDA专业版）桥接给 agent：一组固定工具加上可搜索的 `eda.*` API 目录，经 WebSocket 转发给 EasyEDA 连接器扩展。需先在 `Addons` 区块启用该外挂 —— 安装后其端点即出现在列表中，徽标显示外挂守护进程当前是否已连接。外挂的 `Timeout` 字段限制每个桥接请求的时长（EDA 操作较慢时可调大）；
- **其他 MCP 外挂** —— 任何已安装的 MCP 类型外挂都会以同样方式加入端点，并带实时连接/断开徽标。

在列表上方的下拉框中选择你的 agent CLI，端点旁的复制按钮即可给出该应用可直接运行的接入命令。

## 接入 agent

令牌无需手动粘贴。服务器会维护一个自动生成的 `Vibe Terminal MCP Bearer Token` 环境配置（见“环境变量”章节），导出 `VTERM_MCP_TOKEN`；接入 agent 前先将其应用到终端，再使用复制的命令：

- **命令行应用**（Claude Code、Codex）—— 在普通终端中运行复制的 `mcp add` 命令。Codex 通过 `--bearer-token-env-var` 从 `VTERM_MCP_TOKEN` 读取令牌，命令行中不会出现明文；
- **对话式应用**（Kimi Code、Pi、OpenCode、Qwen Code、DeepSeek Harness）—— 应用环境配置后启动应用，然后把复制的提示词发送到其会话中，由 Agent 直接更新 MCP 配置。Kimi 使用 `bearerTokenEnvVar`；Pi、OpenCode 和 Qwen Code 使用 `VTERM_MCP_TOKEN` 中的 Bearer 令牌，令牌本身不会出现在对话中。

在 Pi 和 Qwen Code 中，`/mcp` 查看连接状态。

设置下方的 `Command Templates` 卡片保存各应用的接入命令：可编辑条目、覆盖内置项，或为新的 agent CLI 添加模板 —— 预览即为复制按钮将生成的内容。
