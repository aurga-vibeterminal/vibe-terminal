檔案瀏覽器操作伺服器端的檔案系統。左側面板頂部一排按鈕切換 Recent / Explorer / Git / Debug / Search 檢視，Explorer 即檔案瀏覽器。

## Home 按鈕：回到終端機目前目錄

點工具列的 `Home`（房子）按鈕：

- 自動切到 `Explorer` 檢視；
- 跳轉到**目前活動終端機的工作目錄**（cwd）；
- 同時把該目錄釘住為樹狀模式，並清空搜尋框——始終從「終端機在哪，檔案就看到哪」開始。

![普通模式下的檔案瀏覽器 —— Home、麵包屑與扁平清單](images/files-explorer.jpg)

## 樹狀模式與搜尋

- **釘住為樹**：麵包屑列右側的圖釘按鈕，在「扁平清單」與「樹狀模式」之間切換。樹狀模式下資料夾展開為可摺疊的目錄樹（Home 跳轉會自動釘住）；
- **搜尋**：頂部的搜尋框按名稱即時過濾目前清單；`Recent` 右鍵選單裡的 `Search in Folder` 則對整個目錄做遞迴搜尋。

![樹狀模式下的檔案瀏覽器 —— 釘住的根目錄展開為可摺疊的目錄樹](images/files-explorer-tree.jpg)

## 基本操作

在檔案或資料夾上右鍵：

- **新建**：`New Folder` / `New File`；
- **編輯**：`Rename` 重新命名；`Copy` / `Cut` / `Paste` 複製、剪下、貼上（支援跨目錄）；
- **刪除**：`Delete`，帶確認彈窗；
- **終端機聯動**：`Open in Terminal` 以該目錄開新終端機分頁；`cd to Directory` 讓目前終端機 cd 過去；
- **本機聯動**：`Reveal in File Explorer` 在系統檔案管理員中顯示（本地目錄）；
- **路徑**：`Copy Full Path` / `Copy Relative Path` 複製完整或相對路徑；
- **服務**：`Host with HTTP server` 把目錄發布為 HTTP 檔案服務（見「HTTP 檔案服務」）；`Open in App` 用伺服器上的 Agent 應用開啟。

所有操作都直接作用於伺服器端的檔案系統，遠端工作階段與本地工作階段體驗一致。
