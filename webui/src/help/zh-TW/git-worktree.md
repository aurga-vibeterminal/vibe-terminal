Worktree 讓一個儲存庫同時檢出多個分支到不同目錄：在不動主工作區的情況下並行開發，完成後把分支合併回主分支並清理工作樹。Vibe Terminal 把這套流程做成了點選操作。

## 建立 Worktree

1. 在 Git 面板的分支清單中，右鍵目標分支（或遠端分支）→ `New Worktree`；
2. 對話框預設以分支名作為新工作樹名稱——可修改；確認後在伺服器端建立工作樹目錄並檢出到該分支；
3. **自動開啟一個新的終端機分頁**，其工作目錄就是新工作樹——立即可以在隔離環境裡開發，主工作區保持原分支不動。

![New Worktree 對話框 —— 名稱預設為分支名](images/git-worktree-create.jpg)

![Worktree 建立後自動開啟的新分頁](images/git-worktree-terminal.jpg)

## 並行工作

每個 worktree 是一個完整獨立的工作目錄：在不同分頁裡同時跑主分支和新分支的建置/測試互不干擾。分支清單裡帶 worktree 標記的分支會顯示其檢出的路徑，在 worktree 分頁裡照常提交、推送。

右鍵工作樹分支還有兩個專屬選單項：`Reveal in File Explorer`（本機工作階段）在系統檔案管理員中開啟其目錄，`Open in Terminal` 再開一個分頁進入該目錄。

## 完成後合併與清理

1. 在 worktree 分頁裡正常提交、推送；
2. 把主分支的分頁切為主分支（如 `main`），在分支清單右鍵 feature 分支 → `Merge into Current Branch`，將工作成果合併回主分支；

   ![把工作樹分支合併回主分支](images/git-worktree-merge.jpg)

3. 合併完成後，右鍵該分支 → `Delete Worktree`：同時刪除工作樹目錄與對應分支。刪除前請確認已合併或推送——彈窗會再次提醒，操作不可恢復。

   ![Delete Worktree 確認彈窗 —— 目錄與分支一併刪除](images/git-worktree-delete.jpg)

