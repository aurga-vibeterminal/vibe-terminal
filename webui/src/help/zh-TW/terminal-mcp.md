每個終端機都可以成為 MCP（Model Context Protocol）工具鏈的一等公民：外部 Agent 透過伺服器暴露的 MCP 工具列出終端機、讀取螢幕、執行命令、等待任務完成——用你熟悉的程式 Agent 驅動真實的互動式 Shell。

## 伺服器提供的 MCP 工具

圍繞終端機工作階段，MCP 用戶端可以：

- `list_terminals` 列出所有開啟的終端機;`get_terminal` 檢視某個終端機的標題、工作目錄與應用狀態；
- `get_screen` 讀取目前可見螢幕內容;`get_scrollback` 讀取捲動歷史；
- `send_command` 在終端機中執行命令列（自動補換行）;`send_input` 發送原始文字（如確認 `y`、填表）;`send_key` 發送命名按鍵（Enter、Ctrl+C、方向鍵等）；
- `clear_terminal` 清屏、`resize_terminal` 調整 PTY 尺寸、`wait_terminal` 等待命令執行完畢（輪詢忙閒狀態）。

## 定位終端機：Terminal ID

工具呼叫用 Terminal ID 指定目標終端機。在終端機內右鍵 → `MCP` → `複製 Terminal ID`，把 ID 給 Agent 即可精確指揮這一個終端機。

![終端機右鍵 —— MCP 子選單](images/terminal-mcp-menu.jpg)

## 一鍵接入 Agent 應用

`MCP` 子選單還會列出伺服器偵測到的 Agent 應用（如編輯器 CLI）。點擊某個應用，終端機裡會直接鍵入一條就緒的 `mcp add` 命令——**不帶換行**，檢查無誤後自己按 Enter 執行，即把本伺服器的 MCP 服務註冊進該 Agent。工作階段中已執行應用時這些條目自動隱藏，避免文字落進應用介面。

> 典型工作流：複製 Terminal ID 給 Agent → Agent 呼叫 `send_command` 啟動建置 → `wait_terminal` 等完成 → `get_screen` 取結果。配合「自動確認」（見終端機章節），長時間互動任務幾乎無需人工值守。
