伺服器內建 Model Context Protocol 宿主，Web UI 的 `MCP` 區塊用於設定它：Code Agent（Claude Code、Codex、Kimi Code、Pi、OpenCode、Qwen Code、DeepSeek Harness 等）可以從它們自己的會話中直接連線並操作你的機器 —— 讀取和驅動終端機，甚至操作 EasyEDA Pro。

## 伺服器設定

啟用 MCP 伺服器並設定：

- `Port`（預設 `6180`）與 `Bind Address` —— `127.0.0.1` 僅限本機存取；`0.0.0.0` 對網路開放；
- `Access Token` —— 非本機請求必須攜帶的 Bearer 權杖；來自 localhost 的請求一律放行。權杖永不為空 —— 全新安裝會產生隨機權杖；骰子按鈕可重新產生，複製按鈕可將其取出；
- `Stream Throttle` —— 向已連線 agent 推送螢幕串流的最小間隔。

變更在 `Save` 後生效；伺服器熱重載，無需重新啟動。

## 端點

每個能力集都有自己的 URL：`http://localhost:{port}/{id}/mcp`：

- **Terminal MCP**（`/terminal/mcp`）—— 內建終端機能力。Agent 可以列出終端機、讀取可見畫面與回捲歷史、輸入原始文字與按鍵、調整終端機尺寸、等待提示符號。工具呼叫透過 Terminal ID 定位終端機 —— 在終端機上右鍵 → `MCP` → `Copy Terminal ID`，把 ID 貼到 agent 的上下文中（詳見「終端機」章節的 Terminal MCP）；
- **EasyEDA Pro MCP**（`/easyeda/mcp`）—— 內建 MCP 附加元件，將 EasyEDA Pro 橋接給 agent：一組固定工具加上可搜尋的 `eda.*` API 目錄，經 WebSocket 轉送給 EasyEDA 連接器擴充功能。需先在 `Addons` 區塊啟用該附加元件 —— 安裝後其端點即出現在清單中，徽章顯示附加元件常駐程序目前是否已連線。附加元件的 `Timeout` 欄位限制每個橋接請求的時長（EDA 操作較慢時可調大）；
- **其他 MCP 附加元件** —— 任何已安裝的 MCP 類型附加元件都會以同樣方式加入端點，並帶即時連線/斷線徽章。

在清單上方的下拉選單中選擇你的 agent CLI，端點旁的複製按鈕即可給出該應用可直接執行的接入命令。

## 接入 agent

權杖無需手動貼上。伺服器會維護一個自動產生的 `Vibe Terminal MCP Bearer Token` 環境設定（見「環境變數」章節），匯出 `VTERM_MCP_TOKEN`；接入 agent 前先將其套用到終端機，再使用複製的命令：

- **命令列應用**（Claude Code、Codex）—— 在一般終端機中執行複製的 `mcp add` 命令。Codex 透過 `--bearer-token-env-var` 從 `VTERM_MCP_TOKEN` 讀取權杖，命令列中不會出現明文；
- **對話式應用**（Kimi Code、Pi、OpenCode、Qwen Code、DeepSeek Harness）—— 套用環境設定後啟動應用，再把複製的提示詞傳送到其會話中，由 Agent 直接更新 MCP 設定。Kimi 使用 `bearerTokenEnvVar`；Pi、OpenCode 和 Qwen Code 使用 `VTERM_MCP_TOKEN` 中的 Bearer 權杖，權杖本身不會出現在對話中。

在 Pi 和 Qwen Code 中，`/mcp` 檢視連線狀態。

設定下方的 `Command Templates` 卡片保存各應用的接入命令：可編輯條目、覆蓋內建項，或為新的 agent CLI 新增範本 —— 預覽即為複製按鈕將產生的內容。
