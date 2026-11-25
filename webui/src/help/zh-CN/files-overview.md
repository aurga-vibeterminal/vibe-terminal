文件管理让你在客户端里直接操作服务器上的文件:左侧活动栏有 `Recent`(最近项目)与文件浏览器两个入口,配合终端标签使用。所有操作都作用于会话所在的服务器——本地机器或经中继连接的远程/共享机器均可。

## Recent:最近项目

`Recent` 列出当前服务器上代码 Agent 最近打开过的文件夹,按时间排序:

- **点击**一条记录,文件浏览器立即切换到该目录;
- **右键**打开快捷菜单,直接执行常用动作:
  - `Open in VS Code` —— 在本机 VS Code 里打开该目录;
  - `Reveal in File Explorer` —— 在系统文件管理器中显示;
  - `Copy Full Path` / `Copy Relative Path` —— 复制完整路径 / 相对仓库的路径;
  - `Search in Folder` —— 在该目录内搜索文件;
  - `Open in Terminal` —— 以该目录为新标签页开终端;`cd Here` —— 在当前终端里 cd 过去;
  - `Host with HTTP server` —— 把该目录变成 HTTP 文件服务(见「HTTP 文件服务」);
  - `Open in App` —— 用服务器上检测到的 Agent 应用打开。

![左侧面板 —— Recent 最近项目列表](images/files-overview.jpg)

## 文件浏览器

`Explorer` 标签浏览和管理服务器端的文件系统:目录导航、树形/列表两种视图、搜索、新建/重命名/删除等文件操作,以及与本地机器之间的文件拖拽上传下载。详见「文件浏览器」与「上传与下载」。
