定制分为两个层次：给现有插件打源码补丁，或参照内置插件编写新插件并上传。

## 源码热补丁

任意插件卡片上的铅笔按钮可打开其 JavaScript 源码编辑。保存即存入一份覆盖原始源码的快照 —— 内置与上传插件皆可 —— 且立即生效、无需重启。适合调整提示词、修正 URL，或在线上快速验证想法。

`Reset`（重置）丢弃快照并恢复原始源码（仅内置插件 —— 上传的插件本来就属于你）。

![编辑插件源码 —— 保存的快照覆盖原始源码](images/addon-edit-source.jpg)

## 编写自己的插件

一个插件由两个文件组成：

- `<id>.json` —— 清单：`id`、`name`、`description`、`author`、`email`、`date`、`type`，以及会被 Web UI 渲染成表单的 `config_fields`（key、label、字段类型、默认值、描述）；
- `<id>.js` —— 源码：纯 JavaScript 文件（由内嵌 QuickJS 引擎运行，提供 `addon.get_config()` 桥接已配置的字段），导出其类型所需的函数 —— 例如 LLM 提供商的 `chat_completion()`、MCP 插件的工具列表与桥接调用。

最快的上手方式是导出现有插件：卡片上的 `Export`（导出）按钮会把清单和源码打包成一个 zip 下载。挑选一个与目标最接近的内置插件（它们也位于 `crates/vterm-server/src/addon/builtins/`），修改 id、name、type 和导出的函数，然后用 `Upload Addon` 安装成果。上传的插件可随时用垃圾桶按钮删除。

![导出插件 —— 清单与 JavaScript 源码打包为一个 zip](images/addon-export.jpg)

## 存储

上传的插件和源码快照以 `<id>.json` + `<id>.js` 的形式存放在配置目录的 `addons/` 文件夹中，重启后依然有效。对于内置 id，只有 `.js` 覆盖会生效 —— 内置插件保留自己的清单。
