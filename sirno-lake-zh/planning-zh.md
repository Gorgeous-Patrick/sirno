---
name: 规划
desc: 将 Sirno 条目用于持久工作工件的一种用法。
category:
  - concept-zh
  - narrative-zh
belongs:
  - sirno-zh
---

规划存在于构建在 Sirno 之上的技能中。

条目是持久的、命名的，并由元数据结构化。
这种结构可以支持持久规划，而无需向 Sirno 添加规划原语。

技能可以将工作表表示为普通条目。
这些条目可以使用类别、`belongs`、`refines` 和见证，就像其他 Lake 条目一样。

这保持核心模型小巧。
规划通常需要状态、优先级、排序、所有权和进度信号。
这些关注因团队和项目而异。
Sirno 提供名称、散文、结构字段、检查和见证；
规划技能可以决定如何使用这些原语表达工作表。

好处是连续性。
以 Sirno 条目形式编写的计划可以引用与设计 Lake 相同的概念和实现承诺。
它可以将相关任务放在一个评审邻域中，
细化更广泛的设计条目，
或标记应由仓库工件见证的工作。
计划保持为可检查的 Markdown，而不是隐藏在单独的任务系统中。

规划条目仍应尊重 Lake。
它们不应该偷偷引入新的结构字段，除非项目显式地设计了它们。
如果工作表需要特殊行为，
该行为属于技能或未来的 Sirno 设计，
而不是核心工具默默忽略的临时元数据。

[generated-footer-zh]: generated-footer-zh.md "Generated Footer"

---

> **Sirno generated links begin. Do not edit this section.**

- belongs (to):
  - [sirno-zh](sirno-zh.md)
- belongs (from): (none)

> **Sirno generated links end.**
