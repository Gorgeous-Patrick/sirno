---
name: 介绍
desc: 穿越 Sirno 项目模型的第一条叙事路线。
category:
  - narrative-zh
---

*名义对象的语义中间表示*

Sirno 在设计形式之间编译，服务于有设计意识的编程工作，
在可选的专著、
由紧凑命名 Markdown 条目组成的 Lake、
和仓库之间移动。

本仓库是一个 Sirno 管理的项目，其设计主题是 Sirno 本身。
通用声明将 Sirno 描述为工具和项目模型。
`this repository` 命名 Sirno 的实现工作区。
`this lake` 命名 `sirno-docs/`，
即描述 Sirno 的自托管 Sirno Lake。
`perspective-and-terms` 条目直接陈述了这一约定。

设计需要一种形式，人类可以阅读，
工具可以索引，
智能体可以在不携带整个项目上下文的情况下操作。
Sirno 通过命名赋予设计这种形式。
产生的名字对人类可读，
对工具稳定，
且小巧到可以流通。

中心形式是 Sirno Lake。
Lake 是一个 Markdown 条目目录。
每个条目有一个稳定的 ID、一个简短的元数据块和一段散文正文。
ID 为人类、工具和智能体提供了一个名义句柄来指代正在讨论的事物。
散文使句柄有意义。

Sirno 保持其元数据词汇小巧。
`category` 说明一个条目是什么种类。
`belongs` 将一个条目置于一个或多个评审邻域中。
`refines` 将一个局部条目连接回它使之更具体的更广泛条目。
这些推荐的字段构成本仓库的结构图。
其他 Sirno 项目可以配置不同的结构字段集。
仓库见证状态通过 `sirno witness ENTRY_ID --full` 发现。
结构字段是显式元数据，
因此工具可以查询它们而无需假装在语义上理解整个设计。

Lake 不仅仅是词汇表。
条目应承载足够的含义以帮助未来的工作。
一些条目定义概念。
一些条目给出穿越这些概念的叙事路线。
一些条目命名局部的实现承诺、
存储边界、生成区域或见证查找行为。
关键是保留后续编辑或评审应该能够引用的设计对象。

Sirno 还命名形式之间的运动。
`lower` 将叙事设计移入 Lake 条目。
`realize` 用条目指导仓库工作。
`reflect` 记录从仓库学到的持久设计事实回到 Lake。
`raise` 在项目需要时将条目组成为可读的专著。
这些变换是工作词汇。
它们不使 Sirno 成为设计质量的裁判。
它们使相关设计对象更易于命名和检查。

仓库见证与实现闭合循环。
见证块存在于已配置的仓库成员中，
以 `sirno:witness:<entry-id>:begin` 打开，
以 `sirno:witness:<entry-id>:end` 关闭。
Sirno 要求 `mosaika` 按条目 ID 定位这些区域。
条目陈述声明。
仓库区域显示该声明可以在哪里被检查。

生成页脚是一个互操作层。
Sirno 可以将选定的元数据字段投影为条目底部的 Markdown 链接。
页脚有保护边界，属于 Sirno。
元数据仍是结构真值的来源。
页脚仅帮助编辑器和文档工具跟踪图。

本仓库现在将 `sirno-docs/` 视为设计源。
你正在阅读的介绍是穿越此 Lake 的第一条路线。
`methodology` 条目是在此 Lake 内行动的紧凑工作指南。
详细设计存在于条目本身中：
形式、结构字段、变换、存储、检查、见证和生成页脚。
先读此条目，
然后跟随 `belongs`、`refines` 和见证到你需要的局部设计。

[generated-footer-zh]: generated-footer-zh.md "Generated Footer"

---

> **Sirno generated links begin. Do not edit this section.**

- belongs (to): (none)
- belongs (from): (none)

> **Sirno generated links end.**
