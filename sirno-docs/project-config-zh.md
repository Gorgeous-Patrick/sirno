---
name: 项目配置
desc: 标记和配置 Sirno 管理仓库的 Sirno.toml 文件。
category:
  - concept-zh
belongs:
  - sirno-lake-zh
refines:
  - storage-zh
---

`Sirno.toml` 将仓库标记为 Sirno 管理的。

文件配置公开条目 Lake
和 Sirno 应用于 Lake 的操作策略。
它还可以配置专著、
仓库见证成员和 Sirno Frost。
生成的配置文件包含简洁的注释，描述每个写入字段如何使用。

`[mono].path` 可选地命名专著。
`[lake].path` 命名 Markdown 条目 Lake。
`[frost].path` 可选地命名私有 Sirno Frost 路径。
`[repo].members` 可选地列出为见证块扫描的仓库路径或 glob。
`[witness]` 配置用于查找见证块的定界符正则。
相对路径从包含 `Sirno.toml` 的目录解析。
CLI `--lake-path PATH` 选项可以为单条命令覆盖 `[lake].path`。

项目可以在没有配置专著、仓库成员或 Sirno Frost 的情况下使用 Sirno。
`sirno init` 创建配置和公开条目 Lake。
`sirno move PATH` 更改 `[lake].path` 并重命名公开 Lake 目录。
`sirno mv PATH` 是其短形式。
`sirno frost init` 添加 Sirno Frost 配置并记录空版本 `0`。
`sirno frost move PATH` 更改 `[frost].path` 并重命名私有 frost 路径。
`sirno frost mv PATH` 是其短形式。

`Sirno.lock.toml` 在配置了 Sirno Frost 时记录公开 Lake 的 frost 状态。
它与 `Sirno.toml` 相邻。
锁说明 Lake 是当前的还是检出到冻结版本。

`[lake].ignore` 列出相对于 Lake 根目录的路径。
Sirno 在读取、检查、查询和更改生成链接时跳过这些路径及其后代。
被忽略的路径用于相邻工具状态，而非条目。

`[repo].members` 在仓库见证启用时列出相对于 `Sirno.toml` 的路径和 glob。
文件成员直接扫描。
目录成员递归扫描。
Glob 成员可以匹配文件或目录。

`[[witness.delimiters]]` 配置一种见证定界符语法。
每个定界符表有 `begin` 和 `end` 正则字段。
每个正则应将条目 ID 捕获为其第一捕获组。
Sirno 拒绝空、无效、无捕获或空匹配的定界符正则。
至少需要一个定界符表以使仓库语法显式。
生成的配置写入标准语法，
它接受 `//` 行注释和隐藏的 Markdown HTML 注释。
标准正则使用一个针对类似文件名条目 ID 的规范捕获。
配置的正则可以更窄，
但应包括活跃项目策略允许的每个条目 ID。

`[check].link` 控制生成链接新鲜度检查。
它默认启用。
格式错误的生成链接哨兵仍为错误，
因为格式错误的哨兵使 Sirno 所有权不明确。

`[structural]` 控制哪些元数据字段被视为结构的。
每个字段键映射到一个带有 `link = { to = bool, from = bool, clique = bool }` 的表。
本仓库推荐 `category`、`belongs` 和 `refines`。
键的顺序是用户编写的项目结构。
Sirno 在重写 `Sirno.toml` 时保留该顺序。
每个 `link` 布尔值是可选的，
缺失的布尔值意味着 false。

`to` 从条目链接到元数据目标。
`from` 从条目链接到将其命名为元数据目标的条目。
`clique` 通过该字段中的共享目标添加独立的团派生部分。

[generated-footer-zh]: generated-footer-zh.md "Generated Footer"

---

> **Sirno generated links begin. Do not edit this section.**

- belongs (to):
  - [sirno-lake-zh](sirno-lake-zh.md)
- belongs (from): (none)

> **Sirno generated links end.**
