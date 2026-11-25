Code Agent（Claude Code、Codex、Kimi、Pi、OpenCode……）只知道送到它输入框里的内容。「添加上下文」让你无需手动输入,就能把文件路径和代码片段交给它们:从 `Explorer` 拖入文件,或从代码编辑器和 Git diff 视图中拖选文本,直接拖到 Agent 所在的终端或 Copilot 对话输入框。

## 拖入文件

在 `Explorer` 文件列表中按住一个文件 —— 或多选 —— 拖拽:

- **拖到终端上**:完整路径会以空格分隔输入到放置点,可以直接成为发给 Agent 的下一条提示词的一部分;
- **拖到 Copilot 对话输入框**:路径按每行一个插入到消息草稿中,方便在发送前组织多文件提示。

从操作系统文件管理器拖入终端的路径会自动加上引号,包含空格的文件名也能保持完整。

![从 Explorer 文件列表拖文件到终端,会自动输入完整路径](images/terminal-context-drag-file.gif)

## 拖选代码作为上下文

在代码编辑器或 Git diff 视图中选中文本,然后按住选区本身拖到终端上:

- 当终端**正在运行 Code Agent** 时,放置会插入一个*上下文块* —— 文件路径、行号范围和带代码围栏的片段:

````text
File: crates/vterm/src/ui/terminal_view.rs
Lines: 558-566
```
fn can_accept_native_drop(&self, info: &DndInfo) -> bool { … }
```
````

- 当终端处于**普通 shell 提示符**时,放置则输入原始选中文本,同样的手势在普通命令下也能用;
- 当 diff 定格在某个提交上查看时,上下文块还会额外带一行 `Commit:`,标明确切的版本。

![把编辑器选区拖入运行 Agent 的终端,会放置一个 File/Lines 上下文块](images/terminal-context-drag-selection.mp4)

## Copy Context

同样的上下文块也在右键菜单里:代码编辑器和 diff 视图中的 `Copy Context` 会把它复制到剪贴板。不方便拖拽时,可以把它粘贴到 Copilot 对话输入框 —— 或任何其他 Agent 中,上下文块的效果见上面的动画。

## 提示

- 在 `Explorer` 中多选可以一次拖入多个路径;
- Copilot 对话输入框的内容会像键盘输入一样直接送入终端（见 `Code Agent 会话历史`),在这里添加的上下文和你手动输入的效果完全一样;
- 配合 `自动确认`,Agent 可以直接对拖入的上下文采取行动,无需再逐条确认。
