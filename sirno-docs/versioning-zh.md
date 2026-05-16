---
name: 版本化
desc: 通过 eter 对 Sirno 条目进行 Lake 范围的不可变快照。
category:
  - concept-zh
belongs:
  - frost-versioning-zh
refines:
  - storage-zh
---

当配置了 Sirno Frost 时，
Sirno 通过将公开 Markdown Lake 冻结到单独的 `eter` frost 路径中来版本化 `sirno` 形式。

版本化条目是 frost 行为的前门。
其局部细化定义了私有的 Sirno Frost 和公开的锁文件。

一个 Sirno 版本是一个 `eter` `SnapshotRef`：
一个 GC 代加上一个 `Eterator` 坐标。
它是条目 Lake 的一个不可变全局快照。
它标识整个 Lake 状态，
而不是一个单独的条目修订。
坐标在其代内是有序的。
条目元数据不存储它，
条目 ID 在各版本间保持稳定。

涟漪是两个 Lake 状态之间的命名增量。
它是通过比较版本、检出状态或其他未来 Lake 快照
而变得可见的可评审差异。

公开 Lake 始终是可编辑的工作形式。
frost 路径是私有存储，
约定为 `sirno-frost`。
它不作为条目 Lake 的一部分被读取，
并且不得放置在 Lake 发现可以将其视为条目的位置。
`sirno frost move PATH` 重命名此路径并更新 `[frost].path`。
`sirno frost mv PATH` 是其短形式。
`Sirno.lock.toml` 记录公开 Lake 相对于该 frost 路径的状态。
它包含一个 `[frost]` 表，其中有 `status`、`generation`、`version`，
以及一个可选的 `mutable` 标志。

`sirno frost init` 配置 frost 路径并记录空版本 `0`。
`sirno frost init --frost-path PATH` 选择非默认 frost 路径。
第一次 frost 提交创建第一个冻结快照。
一次 frost 提交导入选定的公开条目集并写入一个 `eter` 事务。
事务包含变更的条目和生命周期删除。
未变更的活跃条目不接收新版本文件。
它们通过 `eter` 快照读取保留为新 Lake 快照的一部分。
事务写入的所有行接收相同的快照坐标。
在写入事务之前，
Sirno 从提交的条目正文中移除每个保护边界的生成链接区域。
生成链接保持为公开 Lake 投影。
Sirno Frost 保留没有生成导航区域的元数据和散文。
成功提交返回新的 `SnapshotRef`。
如果公开 Lake 与当前冻结快照匹配，
提交返回当前快照引用而不写入新快照。
如果一个条目存在于当前冻结快照中但不在公开 Lake 中，
提交为该条目写入一个 `eter` 生命周期删除标记。
提交后，
`Sirno.lock.toml` 记录 `status = "current"` 和已提交的快照引用。

对公开 Lake 的直接编辑是工作状态编辑。
它们只在 frost 提交后成为冻结版本。
没有版本选择器的读取接口读取公开 Lake。
版本选择器将请求的坐标与当前 frost 代配对。
它从 frost 路径读取并更改观察到的 Lake 状态，
而不更改查询或检查语义。

检出将一份 frost 版本具象化到公开 Markdown 目录中。
它在选定的 `SnapshotRef` 处解析活跃条目并渲染规范的条目文件。
检出使用显式的冲突策略。
保守策略仅写入不存在或为空的目标目录。
CLI 检出替换已配置公开 Lake 中的受管理 Markdown 文件，
同时保留被忽略的路径。
`sirno frost checkout --latest` 将当前快照写为可变当前 Lake。
显式版本检出后，
`Sirno.lock.toml` 记录 `status = "checked-out"` 和选定的快照引用。

普通检出是不可变的。
Sirno 移除公开 Lake 根目录和受管理条目文件的写权限。
它还在每个检出的条目正文开头写入一个可见的 Markdown 引用块，
标记该文件为只读并说明不要手动编辑。
`sirno frost checkout VERSION --unsafe-mutable` 使检出保持可写，
并记录 `mutable = true`。
提交可变 Lake 创建一个新的当前版本。
Sirno 拒绝提交不可变检出。

版本化在 `eter` 中是字段级，在 Sirno 中是条目级。
Sirno 可以通过在连续快照处读取字段来暴露条目历史、diff 和恢复操作。
它将那些结果呈现为条目和结构字段的变更。
公开条目模式保持不变。

恢复一个版本是检出加上随后的 frost 提交。
检出将快照写回公开 Lake。
提交恢复的公开 Lake 创建一个新的当前冻结快照，
因此后续工作保持有序，旧快照保持不可变。
撤销树分支属于 git 或其他外部仓库机制。
Sirno 自己的版本线是线性的。

保留是策略。
Sirno 可以保留命名版本、近期版本、
绑定到导出评审的版本，或所有版本。
未保留的版本可以通过 `eter` 退役和垃圾回收，
仅当没有保留版本需要它们的行时才可以。
文件系统后端不持久化退役快照状态，
因此 Sirno 在执行回收时必须提供活跃集合。

[generated-footer-zh]: generated-footer-zh.md "Generated Footer"

---

> **Sirno generated links begin. Do not edit this section.**

- belongs (to):
  - [frost-versioning-zh](frost-versioning-zh.md)
- belongs (from): (none)

> **Sirno generated links end.**
