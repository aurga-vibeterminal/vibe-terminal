分支清單的右鍵選單集中了 push、checkout 與日常分支操作。

## Push

分支右鍵 → `Push '{分支}'...` 開啟推送對話方塊：

![分支右鍵 —— Push 入口](images/git-push-contextmenu.jpg)

- 顯示 `本地 → 遠端` 的目標映射，本地/遠端分支名都可以改（推送到不同名的遠端分支時會二次確認）；
- 選項：`Push all branches`、`Set upstream`（首推綁定上游）、`Push tags`、`Dry run`（演練不實際推送）；
- 覆寫遠端歷史的 `Force` 與更安全的 `Force with lease`（他人已推送時拒絕，推薦）。帶改寫歷史的操作（如 rebase 後推送）必須使用其中之一；
- 底部有 force 的風險提示。

![Push 對話方塊 —— 目標映射與選項](images/git-push-dialog.jpg)

## Checkout

分支右鍵 → `Checkout` 切換分支——本地分支直接切換，遠端分支或 tag 則開啟檢出對話方塊：

![分支右鍵 —— Checkout 入口](images/git-checkout-contextmenu.jpg)

- `New branch name`——要建立的本地分支名稱，由該引用預填；
- `Create new local branch`（`-B`）、`Track remote as upstream`（`--track`，僅遠端分支）、`Force`（`-f`）、`Merge`（`--merge`）。

![檢出對話方塊 —— 分支名與選項](images/git-checkout-dialog.jpg)

切換後目前工作階段的工作目錄隨之更新，終端機與 Git 面板立即反映新分支。

## Detached HEAD

檢出裸提交（如檢視歷史時 Checkout 某提交）會使 HEAD 脫離分支。此時提交會彈出 `Detached HEAD` 對話方塊：選擇把提交落到已有分支，或就地 `+ Create new branch…`，也可以 Skip 暫不處理。

## 其他操作

- **Stash**：可把全部更改暫存到堆疊上（可附訊息）；`STASHES` 清單支援 `Pop`（恢復並移除）與 `Drop`（丟棄，不可恢復）；
- **刪除分支/Worktree**：右鍵對應條目，均帶不可恢復確認；
- **歷史與回退**：`HISTORY` 列出提交，右鍵檢視檔案歷史、與工作區對比、回退到指定版本；
- **複製**：分支名 / 提交雜湊一鍵複製，方便貼給 Agent 或同事。
