快速命令把项目常用的构建/测试/部署命令挂在终端右键菜单里，一键执行、随处可用。命令按仓库维护,存放在仓库根目录的 `.vterm/commands.json`。

## 创建 commands.json

在终端内右键 → `命令`：

- 该会话的仓库还没有 commands.json 时,菜单里只有一项 `Generate commands.json`——点击后服务器分析项目类型（npm / Cargo / .NET / Xcode / Gradle-Android / 通用）,生成一份保守的起始模板;已存在的非空文件**永远不会被覆盖**；
- 文件位置：Git 仓库根目录的 `.vterm/commands.json`（非 Git 目录则用当前工作目录）。

## 完善

生成的模板只包含该项目类型的常规命令。建议用你接入的代码 Agent（如编辑器里连接了本服务器 MCP 的 Agent）阅读项目并补充/修正命令内容——文件格式：

```json
{
  "commands": [
    { "name": "Build", "command": "cargo build --release" },
    { "name": "Test verbose", "command": "cargo test -- --nocapture",
      "env": { "RUST_LOG": "debug" } },
    { "name": "Package (Windows)", "command": "./packaging/windows/package.bat",
      "os": "win" }
  ]
}
```

也接受顶层数组形式;`env` 可选,为命令执行前追加的环境变量；旧版 `value` 键兼容读取。可选的 `os` 字段把条目限制到运行 `vterm-server` 的操作系统：Windows 用 `"win"`，Linux 用 `"linux"`，macOS 用 `"osx"`；若省略 `os`、设为 `"all"` 或填了无法识别的值，条目在所有平台上都可用。保存后重新打开菜单即可看到新命令（服务器监听 CWD 与 commands.json 变更自动广播）。

## 使用

在终端内右键 → `命令` → 点击命令,即在当前终端执行（先应用可选 env）。没有运行 Agent 应用时菜单才可用,避免命令文本落进应用的输入框。

## 示例：Wi-Fi 无线编译安装 Android 应用

以一个 Gradle-Android 项目为例，完整走一遍：把无线 ADB 设置和编译安装流程挂进右键菜单，之后一键重新编译安装应用。

**1. 把手机切换到无线 ADB。** 这两条命令每台设备只需输入一次 —— 手机接 USB 时执行 `adb tcpip 5555`，然后 `adb connect` 加手机地址：

![终端 —— 用 adb tcpip 5555 和 adb connect 把 Android 手机切换到无线 ADB](images/terminal-commands-adb-setup.gif)

**2. 让代码 Agent 编写命令。** 在终端里打开项目的代码 Agent（这里是 Claude Code），让它添加编译安装命令 —— Agent 阅读项目结构后把条目写入 `.vterm/commands.json`：

![终端 —— Claude Code 在 commands.json 中编写编译安装 APK 命令](images/terminal-commands-agent-edit.mp4)

**3. 从菜单编译安装。** 拆分终端让编译输出占独立一栏，然后右键 → `命令` → 点击条目 —— Gradle 编译 APK 并通过 Wi-Fi 安装到手机：

![终端 —— 拆分终端后从右键菜单执行 Android 编译安装命令](images/terminal-commands-split-run.mp4)
