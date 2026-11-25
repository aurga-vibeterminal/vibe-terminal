分支列表的右键菜单集中了 push、checkout 与日常分支操作。

## Push

分支右键 → `Push '{分支}'...` 打开推送对话框：

![分支右键 —— Push 入口](images/git-push-contextmenu.jpg)

- 显示 `本地 → 远程` 的目标映射,本地/远程分支名都可以改（推送到不同名的远程分支时会二次确认）;
- 选项:`Push all branches`、`Set upstream`（首推绑定上游）、`Push tags`、`Dry run`（演练不实际推送）;
- 覆盖远端历史的 `Force` 与更安全的 `Force with lease`（他人已推送时拒绝,推荐）。带改写历史的操作（如 rebase 后推送）必须用其中之一;
- 底部有 force 的风险提示。

![Push 对话框 —— 目标映射与选项](images/git-push-dialog.jpg)

## Checkout

分支右键 → `Checkout` 切换分支——本地分支直接切换,远程分支或 tag 则打开检出对话框：

![分支右键 —— Checkout 入口](images/git-checkout-contextmenu.jpg)

- `New branch name`——要创建的本地分支名,由该引用预填;
- `Create new local branch`（`-B`）、`Track remote as upstream`（`--track`,仅远程分支）、`Force`（`-f`）、`Merge`（`--merge`）。

![检出对话框 —— 分支名与选项](images/git-checkout-dialog.jpg)

切换后当前会话的工作目录随之更新,终端与 Git 面板立即反映新分支。

## Detached HEAD

检出裸提交（如查看历史时 Checkout 某提交）会使 HEAD 脱离分支。此时提交会弹出 `Detached HEAD` 对话框:选择把提交落到已有分支,或就地 `+ Create new branch…`,也可以 Skip 暂不处理。

## 其他操作

- **Stash**:`Stage All` 旁可把全部更改暂存到栈上（可附信息）;`STASHES` 列表支持 `Pop`（恢复并移除）与 `Drop`（丢弃,不可恢复）;
- **删除分支/Worktree**:右键对应条目,均带不可恢复确认;
- **历史与回退**:`HISTORY` 列出提交,右键查看文件历史、与工作区对比、回退到指定版本;
- **复制**:分支名 / 提交哈希一键复制,方便贴给 Agent 或同事。
