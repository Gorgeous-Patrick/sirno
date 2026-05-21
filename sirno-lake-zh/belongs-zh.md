---
name: 归属
desc: 一个结构字段，将条目置于一个评审邻域中。
category:
  - concept-zh
belongs:
  - structural-field-zh
---

`belongs` 将条目置于一个命名的评审邻域中。

目标条目是评审邻域。
它给一个共享主题、局部词汇或设计区域提供一个前门。
关系是水平的。
一个局部设计或程序变更通常应通过访问该目标、
其成员及其见证或细化来评审。
该字段是列表值的，不排他。
当每个目标是一个真实的评审视角时，一个条目可以命名多个 `belongs` 目标。

该字段涵盖了标签、作用域、命名空间和领域的实用部分。
成员条目保留自己的 ID，而目标条目提供进入该组的路线。

当条目应一起访问因为它们共享工作上下文时，使用 `belongs`。
该字段说明成员属于一个命名的邻域。
它不说明成员是某种类的实例，
也不说明成员使目标条目更具体。
用 `category` 表示种类。
当当前条目收窄一个更广泛的设计声明时，使用 `refines`。

保持 `belongs` 目标稀疏。
目标应帮助导航、评审或问责。
一个松散的浏览标签不应成为结构元数据。

生成的 `belongs` 链接保留直接目标和来源链接。
`structural.belongs.link.clique` 可以添加独立的团派生部分。
启用团部分时，
目标链接到其成员，
每个成员链接到目标和其他成员。

这对跨类别的领域很有用。
例如，一个 Lake 邻域可能包括概念、元数据规则、
生成页脚行为和检查。
这些条目是不同种类的对象，
但它们归属于一起因为它们解释了项目的同一部分。

目标条目应承载真正的解释价值。
如果该组帮助读者进入 Lake 的一个复杂区域，
那么目标赋予该区域一个稳定的前门。
当拆分条目时，
如果同一评审应一起访问它们，则将新条目保持在同一 `belongs` 目标下。
仅当它们改善评审局部性时才添加 `belongs` 目标。
仅当拆分创建了一个真正的新评审边界时才创建新的 `belongs` 目标。

[generated-footer-zh]: generated-footer-zh.md "Generated Footer"

---

> **Sirno generated links begin. Do not edit this section.**

- belongs (to):
  - [structural-field-zh](structural-field-zh.md)
- belongs (from): (none)

> **Sirno generated links end.**
