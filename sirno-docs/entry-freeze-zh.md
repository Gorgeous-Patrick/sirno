---
name: 条目冻结
desc: 一个只读条目标记，将被保护的 Markdown 排除在 Frost 提交之外。
category:
  - concept-zh
belongs:
  - frost-versioning-zh
---

条目冻结是一个公开 Markdown 条目的显式保护状态。
元数据标记是规范的 `frozen:`，不带值。

`sirno freeze ENTRY_ID` 添加标记并从条目文件移除写权限。
`sirno melt ENTRY_ID` 移除标记并恢复写权限。
`sirno unfreeze ENTRY_ID` 是 `sirno melt ENTRY_ID` 的别名。
命令对是更改状态的受支持方式。

冻结的条目在公开 Lake 中保持可见，用于读取、检查和查询。
Sirno Frost 拒绝提交带有 `frozen:` 的条目。
在从它创建 Frost 快照之前先融化条目。

文件权限是本地强制层。
在 Unix 上，冻结从条目文件移除写位。
在其他平台上，当文件系统支持时，Sirno 使用平台只读标志。

[generated-footer-zh]: generated-footer-zh.md "Generated Footer"

---

> **Sirno generated links begin. Do not edit this section.**

- belongs (to):
  - [frost-versioning-zh](frost-versioning-zh.md)
- belongs (from): (none)

> **Sirno generated links end.**
