---
name: 未来工作
desc: 保留的设计区域，可在以后细化。
category:
  - narrative-zh
belongs:
  - sirno-zh
---

几个设计区域保留供以后细化。

`locked` 字段未来可定义条目或生成区域如何抵抗意外编辑。

长期版本保留策略保留给以后设计。
核心模型已经将版本视为 `eter` 快照。
未来工作决定 Sirno 默认保留哪些快照，
哪些快照可以被命名，
以及评审接口如何暴露它们。

变换名称可能仍会细化。
当前名称是 `lower`、`raise`、`realize` 和 `reflect`。

规划技能是未来工作。
它们可以使用条目留下持久的工作工件而不改变 Sirno 的核心字段。

未来工作应保持显式而不变成投机架构。
当前设计因为其核心小而有价值：
条目、元数据、结构字段、生成页脚、形式、变换、检查和见证。
新特性应保持这种清晰性。

`locked` 字段是一个例子。
它最终可能保护项目视为受控的条目、
元数据字段或生成区域。
那个设计在成为模式的一部分之前需要一个清晰的所有权模型。
在那之前，将字段保留比接受模糊的锁行为更安全。

版本保留是另一个例子。
`eter` 提供历史、快照、退役和垃圾回收。
Sirno 仍然需要哪些快照保持活跃的策略。
该策略应保留可评审的 Lake 状态而不使条目元数据更难阅读。

变换名称也可能演变。
当前名称紧凑且易记，
但它们应保持从属于它们描述的模型。
如果项目学到了更清晰的词汇，
条目和手册可以深思熟虑地反映它。

[generated-footer-zh]: generated-footer-zh.md "Generated Footer"

---

> **Sirno generated links begin. Do not edit this section.**

- belongs (to):
  - [sirno-zh](sirno-zh.md)
- belongs (from): (none)

> **Sirno generated links end.**
