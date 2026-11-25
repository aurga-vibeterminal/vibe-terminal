檔案管理讓你在用戶端裡直接操作伺服器上的檔案：左側活動欄有 `Recent`（最近專案）與檔案瀏覽器兩個入口，配合終端機分頁使用。所有操作都作用於工作階段所在的伺服器——本機或經中繼連線的遠端/共享機器均可。

## Recent：最近專案

`Recent` 列出目前伺服器上程式碼 Agent 最近開啟過的資料夾，按時間排序：

- **點擊**一條記錄，檔案瀏覽器立即切換到該目錄；
- **右鍵**開啟快捷選單，直接執行常用動作：
  - `Open in VS Code` —— 在本機 VS Code 裡開啟該目錄；
  - `Reveal in File Explorer` —— 在系統檔案管理員中顯示；
  - `Copy Full Path` / `Copy Relative Path` —— 複製完整路徑 / 相對儲存庫的路徑；
  - `Search in Folder` —— 在該目錄內搜尋檔案；
  - `Open in Terminal` —— 以該目錄為新分頁開終端機；`cd Here` —— 在目前終端機裡 cd 過去；
  - `Host with HTTP server` —— 把該目錄變成 HTTP 檔案服務（見「HTTP 檔案服務」）；
  - `Open in App` —— 用伺服器上偵測到的 Agent 應用開啟。

![左側面板 —— Recent 最近專案清單](images/files-overview.jpg)

## 檔案瀏覽器

`Explorer` 分頁瀏覽和管理伺服器端的檔案系統：目錄導航、樹狀/清單兩種檢視、搜尋、新建/重新命名/刪除等檔案操作，以及與本機之間的檔案拖拽上傳下載。詳見「檔案瀏覽器」與「上傳與下載」。
