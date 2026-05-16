---
name: 项目自应用
desc: 使用 Sirno 自身模型来描述 Sirno 本身。
category:
  - concept-zh
  - narrative-zh
belongs:
  - sirno-zh
---

Sirno 通过自身模型描述自己的设计。

本仓库是一个 Sirno 管理的项目，其设计主题是 Sirno 本身。
这创建了一种递归阅读：
Sirno 是被描述的工具，
Sirno 也是用来组织描述的工具。

Lake 使 Sirno 的设计在小条目中可寻址。
具象化的介绍条目给 Lake 一条第一叙事路线。
具象化的方法论条目给 Lake 一个工作指南。
可选的 Sirno Frost 路径可以保留冻结的 Lake 快照。
仓库工件可以通过 `mosaika` 见证条目。

递归形式很有用，
但当散文在 Sirno 作为工具
和本仓库作为使用 Sirno 的项目之间切换时，它可能模糊视角。
此 Lake 现在通过显式的视角标签保持这些阅读分离。
`Sirno` 命名工具和项目模型。
`a Sirno-managed project` 命名任何应用 Sirno 的项目。
`this repository` 命名 Sirno 的实现工作区。
`this lake` 命名 `sirno-docs/`，
即描述 Sirno 的自托管 Sirno Lake。

介绍应保持为一条可读路线。
变得密集的局部细节应留在条目中，
并通过类别、`belongs`、`refines` 和见证链接。

这种自应用在自己的约束下锻炼了设计。
当实现工作改变了模型时，
该变更可以在任何叙事路线被修订之前反射到 Lake 中。

`meta` 类别是自举表面。
它包含回答 Sirno 管理项目希望其文档如何发展的条目。
这些条目在人、智能体和工具修订 Lake 的其余部分之前对其可用。

Lake 应命名项目期望未来工作引用的对象：
形式、条目、结构字段、变换、元数据、
检查、生成页脚、见证和存储边界。
这些名字成为代码工作、文档工作和评审使用的句柄。

Sirno 术语在与 Sirno 一起出现时成为专有名称：
Sirno Lake 和 Sirno Frost。
否则，小写斜体标记局部模型术语：
*lake*、*entry*、*witness*、*ripple*、*transform*、*monograph* 和 *repository*。
普通词在描述正常项目工作时保持朴素。
那个词汇边界让 Sirno 可以解释任何项目，
包括本仓库，
而不使每句话听起来像属于工具的内部模型。

Sirno 不只是文档化项目；
它让项目文档化自己的文档方法。

仓库见证使自应用更强。
当代码实现了条目解析、生成页脚处理或结构检查时，
该代码可以放在相关条目 ID 的见证块中。
然后 Sirno 可以回答设计问题的两面：
此条目意味着什么，
以及它在哪被见证？

[generated-footer-zh]: generated-footer-zh.md "Generated Footer"

---

> **Sirno generated links begin. Do not edit this section.**

- belongs (to):
  - [sirno-zh](sirno-zh.md)
- belongs (from): (none)

> **Sirno generated links end.**
