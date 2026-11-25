交互式 Rebase 把 `git rebase -i` 变成可视化流程:拖拽整理提交队列、逐个应用、冲突可视化解决,全程无需命令行。

## 发起

打开分支面板,在 `Filter branches...` 输入框里筛选找到目标分支,右键 → `Rebase Current Branch onto '{分支名}'`,把当前分支变基到目标分支上。面板整体切换为 Rebase 视图,流程分四个阶段:

![分支面板中搜索分支,然后变基到目标分支](images/git-rebase-search.jpg)

## 1. 编辑提交队列

当前分支上目标分支没有的提交排成队列:

- **拖拽**调整提交顺序;
- 每个提交右侧的动作下拉: `pick`（保留）/ `squash`（并入上一个,合并信息）/ `fixup`（并入上一个,丢弃信息）/ `edit`（停在该提交修改）/ `drop`（丢弃）;
- 被合并或丢弃的提交卡片会折叠显示。

![Rebase 提交队列 —— 拖拽排序与动作菜单](images/git-rebase-queue.jpg)

## 2. 应用

点 `Start Rebase` 后按队列逐个应用提交,进度条实时显示;中途无冲突则直达完成。随时可 `Abort` 退出。

## 3. 冲突（如有）

应用某个提交发生冲突时进入冲突视图——文件列表 + 合并编辑器的四种模式（`Merge` / `Split` / `Raw` / `AI`）,逐个冲突块做出决策（`Ours` / `Theirs` / `Both` 等,详见「解决冲突」）。全部解决后点 `Continue` 继续 rebase。

![冲突视图 —— 合并编辑器中逐块决策](images/git-rebase-resolve-conflict.jpg)

## 4. 完成

完成后面板显示 `Branch '{分支}' was rebased onto '{目标}'`,并给出推送卡片——变基改写了历史,远端分支需要强制更新:

- `Force Push with Lease`（推荐）——仅在他人没有推送过时才覆盖,绝不使用裸 `--force`;
- `Not now`——跳过推送,本地变基已经完成。

![完成页 —— Force Push with Lease 或跳过推送](images/git-rebase-done.jpg)

> 变基过程中会话照常可用;关闭面板不会中断正在进行的 rebase。
