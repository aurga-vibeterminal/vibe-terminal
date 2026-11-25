`vtermc` 是命令列用戶端：連線伺服器、建立或接入終端機工作階段、管理已儲存的伺服器清單 —— 全部在普通終端機裡完成，適合 SSH 遠端或無圖形介面的機器。它與桌面用戶端共用 `~/.vibeterminal/client.json`，兩邊看到同一份伺服器清單。

## 啟動終端機

不帶模式參數直接執行 `vtermc`，會在本機伺服器（預設 `wss://localhost:5153`）上用偵測到的 shell 新建一個工作階段，並直接串流到目前終端機。在遠端 shell 裡輸入 `exit`（或 `Ctrl+D`）結束工作階段。

工作階段始終執行在伺服器上：連線中斷（網路斷線、關閉視窗、終止 `vtermc`）時工作階段繼續執行，之後可用 `vtermc -s` 重新接入。

- `vtermc -p <id>` —— 使用指定的終端機 Profile 啟動（Profile 清單見 Web UI 的 `Profiles` 頁面）；
- `vtermc -p` —— 不帶 Profile id：互動式列出伺服器的 Profile 供選擇；
- `vtermc -w <dir>` —— 設定新工作階段的工作目錄。

當 stdin 不是終端機（管線或檔案）時，`vtermc` 切換為批次模式：把 stdin 送入工作階段，持續輸出直到工作階段結束 —— 適合指令碼或 Agent 呼叫：

```sh
echo "make test" | vtermc
```

## 連線伺服器

- `--server wss://host:port` —— 連線指定伺服器而不是本機（IPv6 位址加方括號：`wss://[fd00::1]:5153`）；環境變數 `VTERM_SERVER` 可設定同樣的預設值；
- 握手時使用該伺服器的金鑰應答：透過 `-a`/`-s` 新增的伺服器自帶保存在 `client.json` 裡的金鑰；本機伺服器回退到 `TERMINAL_GUI_SECRET_KEY` 環境變數（或 `server.json` 中的明文金鑰）。

## 新增伺服器

`-a` 把伺服器儲存到 `client.json` 後離開 —— 它不會開啟終端機。兩種用法：

- `vtermc -a` —— 互動式輸入 `Server name`、`Server address` 和 `Secret key`。位址必須是 `wss://host:port` 格式（不合法會重新提示；輸入空位址取消）。名稱留空時回退使用位址；
- `vtermc -a "vtshare1:..."` —— 直接貼上加密分享字串。在 Web UI 的 `Server` 頁面點擊配對 `QR Code` 即可複製到剪貼簿；`vtermc` 會自動解密並填入名稱、位址和金鑰。

幾個細節：

- 重複新增同一位址會原地更新該條目（保留其 id），不會產生重複項；
- 本機位址會被拒絕 —— `Local` 始終隱式可用，無需新增；
- 這些條目同樣會出現在桌面用戶端的 Server Manager 裡。

## 選擇伺服器與工作階段

`vtermc -s` 依序開啟兩個全螢幕選擇器（需要互動式終端機）：

1. **伺服器清單** —— `Local` 固定在第一行，之後是每個已儲存伺服器，顯示為 `名稱 (位址)`。方向鍵或 `j`/`k` 移動，數字鍵直達，`Enter` 確認，`q` 取消；
2. **終端機清單** —— 第一項是 `New Terminal`，其後是該伺服器的活躍工作階段（顯示工作目錄、標題和所屬伺服器）。接入現有工作階段，或新建一個（新建時 `-p` / `-w` 參數照常生效）。

接入不會打斷工作階段 —— 它一直在伺服器上執行，隨時可以再接入。

### 刪除伺服器

在伺服器清單中把游標移到某個已儲存的伺服器上按 `d`：該行變紅並出現確認提示。按 `y`（或 `Enter`）確認從 `client.json` 刪除該條目，其他任意鍵取消。`Local` 不可刪除。

## 環境變數

新工作階段預設從目前 shell 繼承一組固定變數：`PATH`、`HOME`、`USER`、`SHELL`、`TERM`、`WT_PROFILE_ID`、`PSMODULEPATH`：

- `--env-inherit <patterns>` —— 逗號分隔的比對清單（也可用環境變數 `VTERM_ENV_INHERIT`）；支援 `*` 萬用字元；
- `-e VAR=VALUE` —— 注入額外變數（可重複）；
- `--no-env` —— 完全不繼承任何變數。

```sh
vtermc -w ~/project -e RUST_LOG=debug --env-inherit "PATH,HOME,TERM"
```

## 全部選項

```text
-p, --profile [<ID>]          選擇終端機 Profile（省略值則互動式列出）
-w, --working-dir <DIR>       設定工作目錄
-s, --select                  互動式選擇伺服器和既有終端機
-a, --add [<SHARE>]           新增伺服器（vtshare1: 分享字串或互動式輸入）
    --server <URL>            伺服器 WebSocket 位址 [env: VTERM_SERVER]
-e, --env <VAR=VALUE>         附加環境變數
    --no-env                  不繼承環境變數
    --env-inherit <PATTERNS>  要繼承的環境變數比對清單 [env: VTERM_ENV_INHERIT]
```
