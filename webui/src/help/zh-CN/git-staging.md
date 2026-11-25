Git 面板的 `CHANGES` 区覆盖日常提交的全部动作:查看改动、暂存、放弃、储藏、写信息提交。

## 暂存与取消暂存

- 未暂存的更改列在 `CHANGES` 下,每个文件带状态徽标——`M` 修改、`A` 新增、`D` 删除、`?` 未跟踪、`!` 冲突(子模块行另带 `SUB` 标记);
- 悬停文件行显示内联操作按钮:未暂存行为 `+`(暂存)和 `×`(放弃),已暂存行为 `−`(取消暂存);区头的 `Stage All Changes` / `Unstage All Changes` 一键处理全部;
- `Ctrl`-点击把文件加入多选,`Shift`-点击选择范围——操作按钮会作用于全部选中文件;
- 点击文件名展开该文件的差异对比,确认改动内容再决定去留;
- 右键文件菜单提供 `View File History`、`Stash This File`、`Copy Full Path` / `Copy Relative Path`、`Reveal in File Explorer`,以及让终端 `cd` 到文件所在目录的快捷项。

![STAGED CHANGES 区 —— 已暂存文件与逐行取消暂存按钮](images/git-staging-staged.jpg)

## 提交

1. 暂存好要提交的内容;
2. 在底部提交框输入信息（`Ctrl+Enter` 快速提交）,或点 AI 按钮生成（见「LLM 辅助」）;
3. 点击 `Commit`。

![提交框 —— 信息输入与 AI 生成按钮](images/git-staging-commit.jpg)

提交完成后,悬停提交历史上方的头部（显示分支名的那条）——`Fetch`、`Pull`、`Push` 按钮在右侧浮现。点 `Push` 立即把当前分支推送到其上游;需要 `Force with lease` 等选项时,改用分支列表的推送对话框（见「Push、Checkout 与更多」）。

![悬停历史头部浮现 Fetch、Pull、Push 按钮](images/git-staging-push.jpg)

## 储藏

- `CHANGES` 区头的储藏按钮打开 `Stash All Changes` 对话框,可填写可选的储藏消息;
- 右键选中的未暂存文件可选 `Stash This File`(多选时为 `Stash <n> Selected Files`),只储藏这些路径——子模块行不提供;
- 储藏条目归入 `STASHES` 区,显示为 `WIP on <分支>`;展开可列出捕获的文件,点击文件查看其差异;
- 悬停储藏条目出现 `Apply`(应用并保留)、`Pop`(应用并移除)、`Drop`(直接丢弃)按钮——`Pop` 与 `Drop` 有确认弹窗,右键菜单另提供 `View Diff`;
- 没有储藏时 `STASHES` 区整体隐藏。

![Stash All Changes 对话框与可选的储藏消息](images/git-staging-stash-all.jpg)

![STASHES 区 —— 展开的储藏条目与 Apply、Pop、Drop 按钮](images/git-staging-stashes.jpg)

![Pop 在确认后应用储藏并将其移出列表](images/git-staging-stash-pop.jpg)

## 放弃更改

未暂存文件的 `Discard Changes` 会丢弃该文件的全部未提交改动——有确认弹窗且**不可恢复**,谨慎使用。

> 提交只包含已暂存的内容;未暂存的改动继续留在工作区。
