---
name: Sirno Frost
desc: 将公开 Sirno Lake 冻结为不可变快照的私有 eter 支持路径。
category:
  - concept-zh
belongs:
  - frost-versioning-zh
refines:
  - versioning-zh
---

Sirno Frost 是可选的私有 `eter` 存储路径，用于不可变的 Sirno Lake 快照。
默认约定是 `sirno-frost`。
`Sirno.toml` 中的 `[frost].path` 命名该路径，
该路径必须与公开 Markdown Lake 保持分离。
公开 Lake 保持为可编辑的工作形式。
frost 层是该形式背后的持久快照基底。

`SirnoFrost` 外观打开已配置的文件系统后端，
并将冻结数据暴露为普通的有类型的 Sirno 条目。
每个条目存储在其稳定的 ID 下。
后端将 `name`、`desc`、有序的结构元数据和 Markdown 正文记录为有类型的字段。
条目的存在通过 `eter` 生命周期字段表示。
这将版本化保留在存储层中，
同时保持公开条目模式。
结构字段顺序保留在 Sirno 的有类型结构元数据中，
因此 Frost 往返将相同的顺序渲染回 Markdown。

`sirno frost init` 在需要时配置 Sirno Frost，
并将空快照记录为版本 `0`。
它不会立即导入或提交公开 Lake。
`--frost-path PATH` 选择非默认路径。
`sirno frost move PATH` 重命名已配置的 frost 路径，
并将新路径写回 `[frost].path`。
`sirno frost mv PATH` 是其短形式。
移动拒绝替换已有的目标。

一次 frost 提交导入选定的公开条目集。
公开目录必须在写入任何快照前通过 review 模式的检查。
携带 `frozen:` 的条目是受保护的公开文件。
Frost 拒绝提交它们，直到 `sirno melt ENTRY_ID` 移除该标记。
Sirno 从提交的正文中剥离生成链接区域，
因为生成页脚是公开 Lake 投影。
提交写入一个 `eter` 事务并返回一个 `SnapshotRef`。
事务仅包含已变更的条目和生命周期删除标记。
未变更的活跃条目在读取时从早期版本文件继承。
该快照引用命名整个已提交的 Lake 状态。
对于文件系统后端，
`Eter.lock.toml` 存储已提交的版本边界。
该边界之上的版本文件被忽略，
并在下次写入前移除。
如果公开 Lake 未变更，
提交返回当前快照引用而不写入。
如果先前活跃的条目从公开 Lake 缺失，
提交记录一个 `eter` 生命周期删除标记。

frost 读取路径从选定的快照重建条目。
它可以读取一个条目、
当前快照的所有活跃条目、
或特定 `SnapshotRef` 处的所有活跃条目。
CLI 版本坐标在快照被读取前与当前 `eter` GC 代配对。

检出将一份冻结快照具象化为 Markdown 文件。
保守写入策略仅写入不存在或为空的目标目录。
CLI 检出替换已配置公开 Lake 中的受管理 Markdown 文件，
并保留被忽略的路径。
`sirno frost checkout --latest` 将当前快照具象化为可变当前 Lake。
显式版本检出写入一个可见的只读引用块，
并从 Lake 根目录和受管理条目文件移除写权限。
`--unsafe-mutable` 使显式版本检出保持可写。

`Sirno.lock.toml` 记录公开 Lake 相对于 frost 的状态。
`status = "current"` 意味着公开 Lake 是可编辑的当前版本。
`status = "checked-out"` 意味着公开 Lake 具象化了一个选定的冻结版本。
锁存储快照代和版本，
加上仅在非安全可变检出时的 `mutable = true`。
Sirno 拒绝提交不可变检出。
提交可变检出创建一个新的当前快照。

Sirno Frost 是私有基底。
用户和工具在调试存储时可以检查它，
但正常的 Sirno 工作应该读取和编辑公开 Lake，
或使用版本感知的 Sirno 接口。
本条目的见证区域展示了外观、快照读取、
提交路径、检出路径、
种子初始化和 `src/frost.rs` 中的删除处理。

[generated-footer-zh]: generated-footer-zh.md "Generated Footer"

---

> **Sirno generated links begin. Do not edit this section.**

- belongs (to):
  - [frost-versioning-zh](frost-versioning-zh.md)
- belongs (from): (none)

> **Sirno generated links end.**
