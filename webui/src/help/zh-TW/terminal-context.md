Code Agent（Claude Code、Codex、Kimi、Pi、OpenCode……）只知道送到它輸入框裡的內容。「新增上下文」讓你無需手動輸入，就能把檔案路徑和程式碼片段交給它們：從 `Explorer` 拖入檔案，或從程式碼編輯器和 Git diff 檢視中拖選文字，直接拖到 Agent 所在的終端機或 Copilot 對話輸入框。

## 拖入檔案

在 `Explorer` 檔案列表中按住一個檔案 —— 或多選 —— 拖曳：

- **拖到終端機上**：完整路徑會以空格分隔輸入到放置點，可以直接成為發給 Agent 的下一則提示詞的一部分；
- **拖到 Copilot 對話輸入框**：路徑按每行一個插入到訊息草稿中，方便在傳送前組織多檔案提示。

從作業系統檔案管理員拖入終端機的路徑會自動加上引號，包含空白的檔名也能保持完整。

![從 Explorer 檔案列表拖檔案到終端機，會自動輸入完整路徑](images/terminal-context-drag-file.gif)

## 拖選程式碼作為上下文

在程式碼編輯器或 Git diff 檢視中選取文字，然後按住選取範圍本身拖到終端機上：

- 當終端機**正在執行 Code Agent** 時，放置會插入一個*上下文區塊* —— 檔案路徑、行號範圍和帶程式碼圍欄的片段：

````text
File: crates/vterm/src/ui/terminal_view.rs
Lines: 558-566
```
fn can_accept_native_drop(&self, info: &DndInfo) -> bool { … }
```
````

- 當終端機處於**一般 shell 提示字元**時，放置則輸入原始選取文字，同樣的手勢在一般指令下也能用；
- 當 diff 定格在某個提交上檢視時，上下文區塊還會額外帶一行 `Commit:`，標明確切的版本。

![把編輯器選取範圍拖入執行 Agent 的終端機，會放置一個 File/Lines 上下文區塊](images/terminal-context-drag-selection.mp4)

## Copy Context

同樣的上下文區塊也在右鍵選單裡：程式碼編輯器和 diff 檢視中的 `Copy Context` 會把它複製到剪貼簿。不方便拖曳時，可以把它貼上到 Copilot 對話輸入框 —— 或任何其他 Agent 中，上下文區塊的效果見上面的動畫。

## 提示

- 在 `Explorer` 中多選可以一次拖入多個路徑；
- Copilot 對話輸入框的內容會像鍵盤輸入一樣直接送入終端機（見 `Code Agent 會話歷史`），在這裡新增的上下文和你手動輸入的效果完全一樣；
- 配合 `自動確認`，Agent 可以直接對拖入的上下文採取行動，無需再逐條確認。
