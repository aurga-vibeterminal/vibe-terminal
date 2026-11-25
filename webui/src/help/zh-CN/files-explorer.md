文件浏览器操作服务器端的文件系统。左侧面板顶部一排按钮切换 Recent / Explorer / Git / Debug / Search 视图,Explorer 即文件浏览器。

## Home 按钮:回到终端当前目录

点工具栏的 `Home`(房子)按钮:

- 自动切到 `Explorer` 视图;
- 跳转到**当前活动终端的工作目录**(cwd);
- 同时把该目录钉住为树形模式,并清空搜索框——始终从"终端在哪,文件就看到哪"开始。

![普通模式下的文件浏览器 —— Home、面包屑与扁平列表](images/files-explorer.jpg)

## 树形模式与搜索

- **钉住为树**:面包屑栏右侧的图钉按钮,在"扁平列表"与"树形模式"之间切换。树形模式下文件夹展开为可折叠的目录树(Home 跳转会自动钉住);
- **搜索**:顶部的搜索框按名称即时过滤当前列表;`Recent` 右键菜单里的 `Search in Folder` 则对整个目录做递归搜索。

![树形模式下的文件浏览器 —— 钉住的根目录展开为可折叠的目录树](images/files-explorer-tree.jpg)

## 基本操作

在文件或文件夹上右键:

- **新建**:`New Folder` / `New File`;
- **编辑**:`Rename` 重命名;`Copy` / `Cut` / `Paste` 复制、剪切、粘贴(支持跨目录);
- **删除**:`Delete`,带确认弹窗;
- **终端联动**:`Open in Terminal` 以该目录开新终端标签;`cd to Directory` 让当前终端 cd 过去;
- **本机联动**:`Reveal in File Explorer` 在系统文件管理器中显示(本地目录);
- **路径**:`Copy Full Path` / `Copy Relative Path` 复制完整或相对路径;
- **服务**:`Host with HTTP server` 把目录发布为 HTTP 文件服务(见「HTTP 文件服务」);`Open in App` 用服务器上的 Agent 应用打开。

所有操作都直接作用于服务器端的文件系统,远程会话与本地会话体验一致。
