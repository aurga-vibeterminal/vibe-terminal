自訂分為兩個層次：給現有附加元件打原始碼補丁，或參照內建附加元件撰寫新附加元件並上傳。

## 原始碼熱補丁

任意附加元件卡片上的鉛筆按鈕可開啟其 JavaScript 原始碼編輯。儲存即存入一份覆寫原始原始碼的快照 —— 內建與上傳附加元件皆可 —— 且立即生效、無需重新啟動。適合調整提示詞、修正 URL，或在線上快速驗證想法。

`Reset`（重設）丟棄快照並還原原始原始碼（僅內建附加元件 —— 上傳的附加元件本來就屬於你）。

![編輯附加元件原始碼 —— 儲存的快照覆寫原始原始碼](images/addon-edit-source.jpg)

## 撰寫自己的附加元件

一個附加元件由兩個檔案組成：

- `<id>.json` —— 清單：`id`、`name`、`description`、`author`、`email`、`date`、`type`，以及會被 Web UI 渲染成表單的 `config_fields`（key、label、欄位類型、預設值、描述）；
- `<id>.js` —— 原始碼：純 JavaScript 檔案（由內嵌 QuickJS 引擎執行，提供 `addon.get_config()` 橋接已設定的欄位），匯出其類型所需的函式 —— 例如 LLM 提供者的 `chat_completion()`、MCP 附加元件的工具清單與橋接呼叫。

最快的上手方式是匯出現有附加元件：卡片上的 `Export`（匯出）按鈕會把清單和原始碼打包成一個 zip 下載。挑選一個與目標最接近的內建附加元件（它們也位於 `crates/vterm-server/src/addon/builtins/`），修改 id、name、type 和匯出的函式，然後用 `Upload Addon` 安裝成果。上傳的附加元件可隨時用垃圾桶按鈕刪除。

![匯出附加元件 —— 清單與 JavaScript 原始碼打包為一個 zip](images/addon-export.jpg)

## 儲存

上傳的附加元件和原始碼快照以 `<id>.json` + `<id>.js` 的形式存放在設定目錄的 `addons/` 資料夾中，重新啟動後依然有效。對於內建 id，只有 `.js` 覆寫會生效 —— 內建附加元件保留自己的清單。
