在 Windows 上使用 WSL 有兩種方式:輕量方式直接把 WSL shell 加為終端機 Profile,完整方式在 WSL 裡安裝 vterm-server。前者只能打命令,後者才擁有檔案管理、Git 面板、MCP 等全部能力。

## 輕量方式:WSL 終端機 Profile

Windows 上安裝 vterm-server 後,**WSL 終端機 Profile 是自動載入的**:伺服器啟動時偵測 `wsl.exe` 並列舉已安裝的發行版,每個發行版在 Profile 清單中產生一個條目(如 `Ubuntu (WSL)`;未列出發行版時提供通用的 `WSL` 條目)。

使用時只需在用戶端標題列的 Profile 選單中選擇對應的 WSL 條目,即啟動該發行版並直接進入終端機,與開啟本機終端機無異。

局限:這類工作階段執行在 Windows 伺服器側——用戶端的**檔案瀏覽器、Git 面板、環境變數等功能不可用**(它們按 Windows 檔案系統語意工作,看不到 WSL 內的 Linux 檔案系統)。

![終端機 Profile 選單 —— 自動列出的 WSL 發行版](images/terminal-wsl-profile.jpg)

## 完整方式:在 WSL 裡安裝 vterm-server

把 WSL 當作一台獨立的 Linux 機器,安裝 headless 版 vterm-server。官網提供一鍵指令碼,複製到 WSL 終端機執行:

```bash
curl -fsSL https://vterminal.dev/install.sh | bash
```

指令碼自動完成:

1. 識別 WSL/headless 環境,下載 **headless 版** vterm-server 到 `~/.local/vterm`;
2. 安裝 **systemd 使用者服務**並立即啟動——每次登入(包括 WSL 啟動)自動執行,無需手動開機;
3. 把 `~/.local/vterm` 加入 PATH。

> WSL 預設未開啟 systemd 時,指令碼會提示在 `/etc/wsl.conf` 中加入 `[boot]` `systemd=true`,然後在 Windows 側執行 `wsl --shutdown` 重啟 WSL 再執行一次。

## 開啟設定頁面

WSL 內執行:

```bash
vterm-server --open
```

命令列印 Web UI 位址(`http://localhost:<設定連接埠>/`)並嘗試呼叫瀏覽器(WSL 下經 `wslview` 開啟 Windows 預設瀏覽器)。得益於 WSL2 的 localhost 轉發,**在 Windows 瀏覽器裡直接開啟同樣的位址**即可設定。服務執行時,列印的位址會帶上一次性登入權杖(`?ticket=...`,由執行中的服務經本機 socket 簽發),開啟即完成登入。

## (可選)把 WSL 伺服器共享到本機中繼

如果 Windows 本機也在執行 vterm-server(充當中繼),可以讓 WSL 的伺服器掛到它下面,用戶端只連本機一處就能看到兩台「機器」:

1. 在 Windows 伺服器的 Web 介面點擊 QR Code 複製分享字串;
2. 開啟 WSL 伺服器的 Web 介面(上節位址),點擊`中繼位址`輸入框旁出現的貼上按鈕,自動填充位址與金鑰後`套用`;
3. Windows 端 vterm-server 執行期間,用戶端在本機 relay 下即可看到 WSL 伺服器的終端機、檔案與 Git。

> WSL2 NAT 模式下,WSL 存取 Windows 主機需用主機在 WSL 側的位址(預設閘道,`cat /etc/resolv.conf` 中的 nameserver);mirrored 網路模式下 `localhost` 即可。中繼細節見「中繼」章節。
