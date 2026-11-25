互動式 Rebase 把 `git rebase -i` 變成視覺化流程：拖拽整理提交佇列、逐個套用、衝突視覺化解決，全程無需命令列。

## 發起

開啟分支面板，在 `Filter branches...` 輸入框裡篩選找到目標分支，右鍵 → `Rebase Current Branch onto '{分支名稱}'`，把目前分支變基到目標分支上。面板整體切換為 Rebase 檢視，流程分四個階段：

![分支面板中搜尋分支，然後變基到目標分支](images/git-rebase-search.jpg)

## 1. 編輯提交佇列

目前分支上目標分支沒有的提交排成佇列：

- **拖拽**調整提交順序；
- 每個提交右側的動作下拉：`pick`（保留）/ `squash`（併入上一個，合併訊息）/ `fixup`（併入上一個，丟棄訊息）/ `edit`（停在該提交修改）/ `drop`（丟棄）；
- 被合併或丟棄的提交卡片會摺疊顯示。

![Rebase 提交佇列 —— 拖拽排序與動作選單](images/git-rebase-queue.jpg)

## 2. 套用

點 `Start Rebase` 後按佇列逐個套用提交，進度條即時顯示；中途無衝突則直達完成。隨時可 `Abort` 退出。

## 3. 衝突（如有）

套用某個提交發生衝突時進入衝突檢視——檔案清單 + 合併編輯器的四種模式（`Merge` / `Split` / `Raw` / `AI`），逐個衝突塊做出決策（`Ours` / `Theirs` / `Both` 等，詳見「解決衝突」）。全部解決後點 `Continue` 繼續 rebase。

![衝突檢視 —— 合併編輯器中逐塊決策](images/git-rebase-resolve-conflict.jpg)

## 4. 完成

完成後面板顯示 `Branch '{分支}' was rebased onto '{目標}'`，並給出推送卡片——變基改寫了歷史，遠端分支需要強制更新：

- `Force Push with Lease`（推薦）——僅在他人沒有推送過時才覆蓋，絕不使用裸 `--force`；
- `Not now`——跳過推送，本地變基已經完成。

![完成頁 —— Force Push with Lease 或跳過推送](images/git-rebase-done.jpg)

> 變基過程中工作階段照常可用；關閉面板不會中斷正在進行的 rebase。
