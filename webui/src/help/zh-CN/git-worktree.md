Worktree 让一个仓库同时检出多个分支到不同目录：在不动主工作区的情况下并行开发,完成后把分支合并回主分支并清理工作树。Vibe Terminal 把这套流程做成了点选操作。

## 创建 Worktree

1. 在 Git 面板的分支列表中,右键目标分支（或远程分支）→ `New Worktree`;
2. 对话框默认以分支名作为新工作树名——可修改;确认后在服务端创建工作树目录并检出到该分支;
3. **自动打开一个新的终端标签页**,其工作目录就是新工作树——立即可以在隔离环境里开发,主工作区保持原分支不动。

![New Worktree 对话框 —— 名称默认为分支名](images/git-worktree-create.jpg)

![Worktree 创建后自动打开的新标签页](images/git-worktree-terminal.jpg)

## 并行工作

每个 worktree 是一个完整独立的工作目录:在不同标签页里同时跑主分支和新分支的构建/测试互不干扰。分支列表里带 worktree 标记的分支会显示其检出的路径,在 worktree 标签页里照常提交、推送。

右键工作树分支还有两个专属菜单项:`Reveal in File Explorer`（本地会话）在系统文件管理器中打开其目录,`Open in Terminal` 再开一个标签页进入该目录。

## 完成后合并与清理

1. 在 worktree 标签页里正常提交、推送;
2. 把主分支的标签页切为主分支（如 `main`）,在分支列表右键 feature 分支 → `Merge into Current Branch`,将工作成果合并回主分支;

   ![把工作树分支合并回主分支](images/git-worktree-merge.jpg)

3. 合并完成后,右键该分支 → `Delete Worktree`:同时删除工作树目录与对应分支。删除前请确认已合并或推送——弹窗会再次提醒,操作不可恢复。

   ![Delete Worktree 确认弹窗 —— 目录与分支一并删除](images/git-worktree-delete.jpg)

