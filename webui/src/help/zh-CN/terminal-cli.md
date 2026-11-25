`vtermc` 是命令行客户端：连接服务器、创建或接入终端会话、管理已保存的服务器列表 —— 全部在普通终端里完成，适合 SSH 远程或无图形界面的机器。它与桌面客户端共用 `~/.vibeterminal/client.json`，两边看到同一份服务器列表。

## 启动终端

不带模式参数直接运行 `vtermc`，会在本地服务器（默认 `wss://localhost:5153`）上用检测到的 shell 新建一个会话，并直接串流到当前终端。在远端 shell 里输入 `exit`（或 `Ctrl+D`）结束会话。

会话始终运行在服务器上：连接断开（网络中断、关闭窗口、杀掉 `vtermc`）时会话继续运行，之后可用 `vtermc -s` 重新接入。

- `vtermc -p <id>` —— 使用指定的终端 Profile 启动（Profile 列表见 Web UI 的 `Profiles` 页面）；
- `vtermc -p` —— 不带 Profile id：交互式列出服务器的 Profile 供选择；
- `vtermc -w <dir>` —— 设置新会话的工作目录。

当 stdin 不是终端（管道或文件）时，`vtermc` 切换为批处理模式：把 stdin 送入会话，持续输出直到会话结束 —— 适合脚本或 Agent 调用：

```sh
echo "make test" | vtermc
```

## 连接服务器

- `--server wss://host:port` —— 连接指定服务器而不是本地（IPv6 地址加方括号：`wss://[fd00::1]:5153`）；环境变量 `VTERM_SERVER` 可设置同样的默认值；
- 握手时使用该服务器的密钥应答：通过 `-a`/`-s` 添加的服务器自带保存在 `client.json` 里的密钥；本地服务器回退到 `TERMINAL_GUI_SECRET_KEY` 环境变量（或 `server.json` 中的明文密钥）。

## 添加服务器

`-a` 把服务器保存到 `client.json` 后退出 —— 它不会打开终端。两种用法：

- `vtermc -a` —— 交互式输入 `Server name`、`Server address` 和 `Secret key`。地址必须是 `wss://host:port` 格式（不合法会重新提示；输入空地址取消）。名称留空时回退使用地址；
- `vtermc -a "vtshare1:..."` —— 直接粘贴加密分享串。在 Web UI 的 `Server` 页面点击配对 `QR Code` 即可复制到剪贴板；`vtermc` 会自动解密并填入名称、地址和密钥。

几个细节：

- 重复添加同一地址会原地更新该条目（保留其 id），不会产生重复项；
- 本机地址会被拒绝 —— `Local` 始终隐式可用，无需添加；
- 这些条目同样会出现在桌面客户端的 Server Manager 里。

## 选择服务器与会话

`vtermc -s` 依次打开两个全屏选择器（需要交互式终端）：

1. **服务器列表** —— `Local` 固定在第一行，之后是每个已保存服务器，显示为 `名称 (地址)`。方向键或 `j`/`k` 移动，数字键直达，`Enter` 确认，`q` 取消；
2. **终端列表** —— 第一项是 `New Terminal`，其后是该服务器的活跃会话（显示工作目录、标题和所属服务器）。接入现有会话，或新建一个（新建时 `-p` / `-w` 参数照常生效）。

接入不会打断会话 —— 它一直在服务器上运行，随时可以再接入。

### 删除服务器

在服务器列表中把光标移到某个已保存的服务器上按 `d`：该行变红并出现确认提示。按 `y`（或 `Enter`）确认从 `client.json` 删除该条目，其他任意键取消。`Local` 不可删除。

## 环境变量

新会话默认从当前 shell 继承一组固定变量：`PATH`、`HOME`、`USER`、`SHELL`、`TERM`、`WT_PROFILE_ID`、`PSMODULEPATH`：

- `--env-inherit <patterns>` —— 逗号分隔的匹配列表（也可用环境变量 `VTERM_ENV_INHERIT`）；支持 `*` 通配符；
- `-e VAR=VALUE` —— 注入额外变量（可重复）；
- `--no-env` —— 完全不继承任何变量。

```sh
vtermc -w ~/project -e RUST_LOG=debug --env-inherit "PATH,HOME,TERM"
```

## 全部选项

```text
-p, --profile [<ID>]          选择终端 Profile（省略值则交互式列出）
-w, --working-dir <DIR>       设置工作目录
-s, --select                  交互式选择服务器和已有终端
-a, --add [<SHARE>]           添加服务器（vtshare1: 分享串或交互式输入）
    --server <URL>            服务器 WebSocket 地址 [env: VTERM_SERVER]
-e, --env <VAR=VALUE>         附加环境变量
    --no-env                  不继承环境变量
    --env-inherit <PATTERNS>  要继承的环境变量匹配列表 [env: VTERM_ENV_INHERIT]
```
