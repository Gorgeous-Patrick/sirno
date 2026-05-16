---
name: Frost 版本化
desc: Sirno Frost 版本化和冻结子系统的评审邻域。
category:
  - concept-zh
---

Frost 版本化是 Sirno 如何将 Lake 冻结为不可变状态的评审前门。

子系统有几个部分：
`versioning` 陈述 Lake 范围的快照模型，
`sirno-frost` 是持有快照的私有 `eter` 支持路径，
`sirno-lock` 记录公开 Lake 的 frost 状态，
`entry-freeze` 保护一个条目免于 Frost 提交。

这些部分一起评审。
对快照模型、frost 路径、锁文件或条目保护的变更
通常会约束其他部分，因此此条目为它们提供一个邻域。

`versioning` 和 `storage` 仍是这些部分细化的更广泛声明。
该邻域是独立水平视图：
`refines` 说明一个部分特化什么，
这里的 `belongs` 说明哪些部分一起评审。

[generated-footer-zh]: generated-footer-zh.md "Generated Footer"

---

> **Sirno generated links begin. Do not edit this section.**

- belongs (to): (none)
- belongs (from):
  - [entry-freeze-zh](entry-freeze-zh.md)
  - [sirno-frost-zh](sirno-frost-zh.md)
  - [sirno-lock-zh](sirno-lock-zh.md)
  - [versioning-zh](versioning-zh.md)

> **Sirno generated links end.**
