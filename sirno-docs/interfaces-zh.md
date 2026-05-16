---
name: 接口
desc: 操作 Sirno 项目存储的 CLI 和 MCP 表面。
category:
  - concept-zh
belongs:
  - sirno-zh
---

Sirno 通过 CLI 和 MCP 接口暴露已配置的项目存储。
轻量级 GUI 或 Obsidian 扩展未来可能提供直接编辑体验。

CLI 是第一个操作接口。
它可以初始化 Lake、创建条目、查询条目、检查结构、
移动已配置的存储路径，以及维护生成页脚链接。
全局 `-C, --config PATH` 选项选择 Sirno 项目配置文件。
全局 `-L, --lake-path PATH` 选项为读写活跃 Lake 的命令
覆盖已配置的公开 Lake。
常用命令别名保持终端使用紧凑：
`q` 代表 `query`，`st` 代表 `status`，`w` 或 `wit` 代表 `witness`。
Lake 操作也存在于 `sirno lake` 下。
分组拼写使用与顶层拼写相同的子命令和别名。
例如，`sirno query`、`sirno q`、`sirno lake query` 和 `sirno lake q`
选择相同的操作。
这些命令应保持足够简洁以便从终端使用，
并足够稳定以供智能体和技能调用。

`sirno status` 总结已配置的仓库。
它报告配置路径、专著状态、Lake 路径、可选的 frost 路径、
frost 锁状态、条目计数、检查策略、结构策略和当前检查结果。

`sirno move PATH` 更改已配置的公开 Lake 路径
并在文件系统上重命名当前 Lake 目录。
`sirno mv PATH` 是其短形式。

`sirno frost init` 配置私有 frost 路径并记录空版本 `0`。
`sirno frost init --frost-path PATH` 选择非默认 frost 路径。
`sirno frost move PATH` 更改已配置的 frost 路径
并在文件系统上重命名当前 frost 路径。
`sirno frost mv PATH` 是其短形式。
`sirno frost commit` 冻结当前公开 Lake
并将生成的当前快照引用写入 `Sirno.lock.toml`。
`sirno frost checkout --latest` 将最新版本具象化为可变公开 Lake。
`sirno frost checkout VERSION` 将旧版本具象化到公开 Lake 中。
版本检出是不可变的，除非提供 `--unsafe-mutable`。

`sirno new` 从键入的命令行元数据创建一个 Markdown 条目。
`-d`、`-n` 和 `-b` 标志是 `--desc`、`--name` 和 `--body` 的短形式。
`--structural FIELD=ENTRY_ID` 选项添加已配置的结构元数据目标。
它拒绝覆盖已有的条目文件。

`sirno freeze ENTRY_ID` 向一个公开条目添加 `frozen:`
并从该文件移除写权限。
`sirno melt ENTRY_ID` 从一个公开条目移除 `frozen:`
并恢复写权限。
`sirno unfreeze ENTRY_ID` 是 `sirno melt ENTRY_ID` 的别名。

`sirno query` 读取已配置的 Markdown Lake。
其默认模式是模糊文本查询。
精确结构谓词使用 `-x, --exact FIELD=ENTRY_ID`。
`-f, --fields` 选项选择输出字段。
`-o, --format` 选项选择输出格式。

`sirno check` 检查活跃 Lake。
`-m, --mode` 选项选择检查边界。

`sirno rg` 对活跃 Lake 路径运行 `rg`。
它将参数转发给 `rg` 二进制文件，
然后附加已解析的 Lake 路径。
它保持 `rg` 的退出码。
默认情况下，
它要求 `rg` 通过一个预处理器搜索 Markdown 条目，
该预处理器屏蔽属于 Sirno 的生成页脚区域。
屏蔽保留路径、换行符和这些区域之外的字节偏移。
使用 `--with-generated-footer` 时，
它搜索完整的 Markdown 文件，包括生成链接。

`sirno witness ENTRY_ID` 通过 `mosaika` 扫描已配置的仓库成员，
并报告选定条目 ID 的仓库见证块。
它首先在活跃 Lake 中解析 `ENTRY_ID`。
缺失的条目在扫描仓库成员之前失败。
`sirno witness ENTRY_ID -f, --full` 还打印完整的匹配仓库区域。
见证输出报告开头和结尾的定界符范围。
定界符范围从哨兵文本开始，不包括前导缩进。
在完整模式下，摘要行仅包含范围。
显示的区域是由见证块跨越的完整行集。
Sirno 保留匹配的缩进。
空行将摘录与区域分开。
多个完整区域由空行、`---` 和另一个空行分隔。

`sirno gen-link` 创建或替换属于 Sirno 的生成页脚区域。
`sirno gen-link -n, --dry` 报告将变更的生成页脚区域而不写入文件。
`--dry-run` 是 `--dry` 的别名。
`sirno gen-link delete` 移除这些区域。
生成链接命令在活跃 Lake 路径上操作。

`sirno util completion` 输出 shell 补全脚本。
补全生成是实用接口，
而不是 Lake 操作。

MCP 接口服务于交互式工具。
它可以将相同的 Lake 模型暴露给智能体和编辑器，而不要求它们为每个操作调用 shell。
未来的 GUI 或 Obsidian 工作应保持相同的所有权规则：
元数据是结构的，
生成页脚区域属于 Sirno，
生成区域之外的散文保持用户拥有。

[generated-footer-zh]: generated-footer-zh.md "Generated Footer"

---

> **Sirno generated links begin. Do not edit this section.**

- belongs (to):
  - [sirno-zh](sirno-zh.md)
- belongs (from): (none)

> **Sirno generated links end.**
