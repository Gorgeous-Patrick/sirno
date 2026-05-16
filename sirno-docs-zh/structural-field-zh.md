---
name: 结构字段
desc: 承载操作型 Sirno 结构的元数据字段。
category:
  - concept-zh
belongs:
  - sirno-zh
---

结构字段是 Sirno 将其作为项目结构读取的已配置元数据字段。

本仓库推荐 `category`、`belongs` 和 `refines`。
`Sirno.toml` 在 `[structural]` 下定义活跃集。
已配置的字段是普通条目元数据，
但 Sirno 将其值视为驱动查询、检查和生成链接的图。

结构字段按 ID 引用条目。
它们是列表值的，可以命名多个目标。
它们的配置顺序是用户管理的。
Sirno 在渲染已配置的结构表面时使用该顺序。
智能体应通过 `sirno witness ENTRY_ID --full` 机械地发现见证区域。

这是结构字段条目的评审前门。
它为字段集提供一个评审前门，同时让每个字段条目自由
承载自己的含义和其他 `belongs` 目标。

本条目的仓库见证应展示通用的结构元数据映射。
活跃字段集由 `Sirno.toml` 定义。

[generated-footer-zh]: generated-footer-zh.md "Generated Footer"

---

> **Sirno generated links begin. Do not edit this section.**

- belongs (to):
  - [sirno-zh](sirno-zh.md)
- belongs (from):
  - [belongs-zh](belongs-zh.md)
  - [category-zh](category-zh.md)
  - [refines-zh](refines-zh.md)

> **Sirno generated links end.**
