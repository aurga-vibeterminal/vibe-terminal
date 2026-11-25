快速命令把專案常用的建置/測試/部署命令掛在終端機右鍵選單裡，一鍵執行、隨處可用。命令按儲存庫維護，存放在儲存庫根目錄的 `.vterm/commands.json`。

## 建立 commands.json

在終端機內右鍵 → `命令`：

- 該工作階段的儲存庫還沒有 commands.json 時，選單裡只有一項 `Generate commands.json`——點擊後伺服器分析專案類型（npm / Cargo / .NET / Xcode / Gradle-Android / 通用），產生一份保守的起始範本；已存在的非空檔案**永遠不會被覆寫**；
- 檔案位置：Git 儲存庫根目錄的 `.vterm/commands.json`（非 Git 目錄則用目前工作目錄）。

## 完善

產生的範本只包含該專案類型的常規命令。建議用你接入的程式碼 Agent（如編輯器裡連接了本伺服器 MCP 的 Agent）閱讀專案並補充/修正命令內容——檔案格式：

```json
{
  "commands": [
    { "name": "Build", "command": "cargo build --release" },
    { "name": "Test verbose", "command": "cargo test -- --nocapture",
      "env": { "RUST_LOG": "debug" } },
    { "name": "Package (Windows)", "command": "./packaging/windows/package.bat",
      "os": "win" }
  ]
}
```

也接受頂層陣列形式;`env` 可選，為命令執行前追加的環境變數；舊版 `value` 鍵相容讀取。可選的 `os` 欄位把條目限制到執行 `vterm-server` 的作業系統：Windows 用 `"win"`，Linux 用 `"linux"`，macOS 用 `"osx"`；若省略 `os`、設為 `"all"` 或填了無法識別的值，條目在所有平台上皆可用。儲存後重新開啟選單即可看到新命令（伺服器監聽 CWD 與 commands.json 變更自動廣播）。

## 使用

在終端機內右鍵 → `命令` → 點擊命令，即在目前終端機執行（先套用可選 env）。沒有執行 Agent 應用時選單才可用，避免命令文字落進應用的輸入框。

## 範例：Wi-Fi 無線編譯安裝 Android 應用

以一個 Gradle-Android 專案為例，完整走一遍：把無線 ADB 設定和編譯安裝流程掛進右鍵選單，之後一鍵重新編譯安裝應用。

**1. 把手機切換到無線 ADB。** 這兩條命令每台裝置只需輸入一次 —— 手機接 USB 時執行 `adb tcpip 5555`，然後 `adb connect` 加手機位址：

![終端機 —— 用 adb tcpip 5555 和 adb connect 把 Android 手機切換到無線 ADB](images/terminal-commands-adb-setup.gif)

**2. 讓程式碼 Agent 撰寫命令。** 在終端機裡開啟專案的程式碼 Agent（這裡是 Claude Code），讓它新增編譯安裝命令 —— Agent 閱讀專案結構後把條目寫入 `.vterm/commands.json`：

![終端機 —— Claude Code 在 commands.json 中撰寫編譯安裝 APK 命令](images/terminal-commands-agent-edit.mp4)

**3. 從選單編譯安裝。** 拆分終端機讓編譯輸出佔獨立一欄，然後右鍵 → `命令` → 點擊條目 —— Gradle 編譯 APK 並透過 Wi-Fi 安裝到手機：

![終端機 —— 拆分終端機後從右鍵選單執行 Android 編譯安裝命令](images/terminal-commands-split-run.mp4)
