---
name: Sirno 锁
desc: 记录公开 Lake frost 状态的 TOML 文件。
category:
  - concept-zh
belongs:
  - frost-versioning-zh
refines:
  - versioning-zh
---

`Sirno.lock.toml` 记录公开 Lake 相对于已配置 frost 路径的状态。
它是 TOML 格式，与 `Sirno.toml` 相邻。
仅当配置了 Sirno Frost 时才被写入。

锁包含一个 `[frost]` 表。
`status = "current"` 意味着公开 Lake 代表当前可编辑的 frost 版本。
`status = "checked-out"` 意味着公开 Lake 具象化了一个选定的冻结版本。
`sirno frost checkout --latest` 记录 `status = "current"` 并使文件保持可写。
`generation` 和 `version` 字段存储该状态的 `eter` `SnapshotRef`。
`version` 是存储的 GC 代内的原始 `Eterator` 坐标。
Sirno 通过将完整 TOML 文件渲染到同级临时路径
并将其重命名到位来写入锁。
失败的写入将先前的完整锁保留为公开状态。

普通检出是不可变的。
Sirno 移除公开 Lake 根目录和受管理条目文件的写权限。
它还在每个检出的条目正文开头写入一个可见的 Markdown 引用块，
说明该文件为只读且不应手动编辑。
`sirno frost checkout VERSION --unsafe-mutable` 使检出保持可写，
并记录 `mutable = true`。

提交可变 Lake 写入一个新的当前 frost 版本，
并将锁重写为 `status = "current"`。
Sirno 拒绝提交不可变检出。

[generated-footer-zh]: generated-footer-zh.md "Generated Footer"

---

> **Sirno generated links begin. Do not edit this section.**

- belongs (to):
  - [frost-versioning-zh](frost-versioning-zh.md)
- belongs (from): (none)

> **Sirno generated links end.**
