每个终端都可以成为 MCP（Model Context Protocol）工具链的一等公民：外部 Agent 通过服务器暴露的 MCP 工具列出终端、读取屏幕、执行命令、等待任务完成——用你熟悉的编程 Agent 驱动真实的交互式 Shell。

## 服务器提供的 MCP 工具

围绕终端会话,MCP 客户端可以：

- `list_terminals` 列出所有打开的终端;`get_terminal` 查看某个终端的标题、工作目录与应用状态；
- `get_screen` 读取当前可见屏幕内容;`get_scrollback` 读取滚动历史；
- `send_command` 在终端中执行命令行（自动补回车）;`send_input` 发送原始文本（如确认 `y`、填表）;`send_key` 发送命名按键（Enter、Ctrl+C、方向键等）；
- `clear_terminal` 清屏、`resize_terminal` 调整 PTY 尺寸、`wait_terminal` 等待命令执行完毕（轮询忙闲状态）。

## 定位终端：Terminal ID

工具调用用 Terminal ID 指定目标终端。在终端内右键 → `MCP` → `复制 Terminal ID`,把 ID 粘给 Agent 即可精确指挥这一一个终端。

![终端右键 —— MCP 子菜单](images/terminal-mcp-menu.jpg)

## 一键接入 Agent 应用

`MCP` 子菜单还会列出服务器检测到的 Agent 应用（如编辑器 CLI）。点击某个应用,终端里会直接键入一条就绪的 `mcp add` 命令——**不带回车**，检查无误后自己按 Enter 执行,即把本服务器的 MCP 服务注册进该 Agent。会话中已运行应用时这些条目自动隐藏,避免文本落进应用界面。

> 典型工作流：复制 Terminal ID 给 Agent → Agent 调 `send_command` 启动构建 → `wait_terminal` 等完成 → `get_screen` 取结果。配合「自动确认」（见终端章节）,长时间交互任务几乎无需人工值守。
