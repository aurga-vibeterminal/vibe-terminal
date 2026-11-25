在 Windows 上使用 WSL 有两种方式:轻量方式直接把 WSL shell 加为终端 Profile,完整方式在 WSL 里安装 vterm-server。前者只能打命令,后者才拥有文件管理、Git 面板、MCP 等全部能力。

## 轻量方式:WSL 终端 Profile

Windows 上安装 vterm-server 后,**WSL 终端 Profile 是自动加载的**:服务器启动时检测 `wsl.exe` 并枚举已安装的发行版,每个发行版在 Profile 列表中生成一个条目(如 `Ubuntu (WSL)`;未列出发行版时提供通用的 `WSL` 条目)。

使用时只需在客户端标题栏的 Profile 菜单中选择对应的 WSL 条目,即启动该发行版并直接进入终端,与打开本地终端无异。

局限:这类会话运行在 Windows 服务器侧——客户端的**文件浏览器、Git 面板、环境变量等功能不可用**(它们按 Windows 文件系统语义工作,看不到 WSL 内的 Linux 文件系统)。

![终端 Profile 菜单 —— 自动列出的 WSL 发行版](images/terminal-wsl-profile.jpg)

## 完整方式:在 WSL 里安装 vterm-server

把 WSL 当作一台独立的 Linux 机器,安装 headless 版 vterm-server。官网提供一键脚本,复制到 WSL 终端执行:

```bash
curl -fsSL https://vterminal.dev/install.sh | bash
```

脚本自动完成:

1. 识别 WSL/headless 环境,下载 **headless 版** vterm-server 到 `~/.local/vterm`;
2. 安装 **systemd 用户服务**并立即启动——每次登录(包括 WSL 启动)自动运行,无需手动开机;
3. 把 `~/.local/vterm` 加入 PATH。

> WSL 默认未开启 systemd 时,脚本会提示在 `/etc/wsl.conf` 中加入 `[boot]` `systemd=true`,然后在 Windows 侧执行 `wsl --shutdown` 重启 WSL 再运行一次。

## 打开配置页面

WSL 内执行:

```bash
vterm-server --open
```

命令打印 Web UI 地址(`http://localhost:<配置端口>/`)并尝试调起浏览器(WSL 下经 `wslview` 打开 Windows 默认浏览器)。得益于 WSL2 的 localhost 转发,**在 Windows 浏览器里直接打开同样的地址**即可配置。服务运行时,打印的地址会带上一枚一次性登录令牌(`?ticket=...`,由运行中的服务经本地 socket 签发),打开即完成登录。

## (可选)把 WSL 服务器共享到本机中继

如果 Windows 本机也在运行 vterm-server(充当中继),可以让 WSL 的服务器挂到它下面,客户端只连本机一处就能看到两台"机器":

1. 在 Windows 服务器的 Web 界面点击二维码复制分享串;
2. 打开 WSL 服务器的 Web 界面(上节地址),点击 `中继地址` 输入框旁出现的粘贴按钮,自动填充地址与密钥后 `应用`;
3. Windows 端 vterm-server 运行期间,客户端在本机 relay 下即可看到 WSL 服务器的终端、文件与 Git。

> WSL2 NAT 模式下,WSL 访问 Windows 主机需用主机在 WSL 侧的地址(默认网关,`cat /etc/resolv.conf` 中的 nameserver);mirrored 网络模式下 `localhost` 即可。中继细节见「中继」章节。
