Git 面板的 `CHANGES` 區涵蓋日常提交的全部動作：檢視改動、暫存、放棄、儲藏、寫訊息提交。

## 暫存與取消暫存

- 未暫存的變更列在 `CHANGES` 下，每個檔案帶狀態徽標——`M` 修改、`A` 新增、`D` 刪除、`?` 未追蹤、`!` 衝突（子模組行另帶 `SUB` 標記）；
- 懸停檔案行顯示內聯操作按鈕：未暫存行為 `+`（暫存）和 `×`（放棄），已暫存行為 `−`（取消暫存）；區頭的 `Stage All Changes` / `Unstage All Changes` 一鍵處理全部；
- `Ctrl`-點擊把檔案加入多選，`Shift`-點擊選擇範圍——操作按鈕會作用於全部選取檔案；
- 點擊檔名展開該檔案的差異對比，確認改動內容再決定去留；
- 右鍵檔案選單提供 `View File History`、`Stash This File`、`Copy Full Path` / `Copy Relative Path`、`Reveal in File Explorer`，以及讓終端機 `cd` 到檔案所在目錄的快捷項。

![STAGED CHANGES 區 —— 已暫存檔案與逐行取消暫存按鈕](images/git-staging-staged.jpg)

## 提交

1. 暫存好要提交的內容；
2. 在底部提交框輸入訊息（`Ctrl+Enter` 快速提交），或點 AI 按鈕產生（見「LLM 輔助」）；
3. 點擊 `Commit`。

![提交框 —— 訊息輸入與 AI 產生按鈕](images/git-staging-commit.jpg)

提交完成後，懸停提交歷史上方的標頭（顯示分支名稱的那條）——`Fetch`、`Pull`、`Push` 按鈕在右側浮現。點 `Push` 立即把目前分支推送到其上游；需要 `Force with lease` 等選項時，改用分支清單的推送對話框（見「Push、Checkout 與更多」）。

![懸停歷史標頭浮現 Fetch、Pull、Push 按鈕](images/git-staging-push.jpg)

## 儲藏

- `CHANGES` 區頭的儲藏按鈕開啟 `Stash All Changes` 對話框，可填寫選填的儲藏訊息；
- 右鍵選取的未暫存檔案可選 `Stash This File`（多選時為 `Stash <n> Selected Files`），只儲藏這些路徑——子模組行不提供；
- 儲藏條目歸入 `STASHES` 區，顯示為 `WIP on <分支>`；展開可列出擷取的檔案，點擊檔案檢視其差異；
- 懸停儲藏條目出現 `Apply`（套用並保留）、`Pop`（套用並移除）、`Drop`（直接捨棄）按鈕——`Pop` 與 `Drop` 有確認彈窗，右鍵選單另提供 `View Diff`；
- 沒有儲藏時 `STASHES` 區整體隱藏。

![Stash All Changes 對話框與選填的儲藏訊息](images/git-staging-stash-all.jpg)

![STASHES 區 —— 展開的儲藏條目與 Apply、Pop、Drop 按鈕](images/git-staging-stashes.jpg)

![Pop 在確認後套用儲藏並將其移出清單](images/git-staging-stash-pop.jpg)

## 放棄變更

未暫存檔案的 `Discard Changes` 會丟棄該檔案的全部未提交改動——有確認彈窗且**不可恢復**，謹慎使用。

> 提交只包含已暫存的內容；未暫存的改動繼續留在工作區。
