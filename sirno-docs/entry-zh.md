---
name: 条目
desc: Sirno Lake 中的一个命名 Markdown 文档。
category:
  - concept-zh
belongs:
  - sirno-lake-zh
---

一个条目是 Sirno Lake 中的一个 Markdown 文件。

文件名主干是条目 ID。
ID 在 Lake 内全局唯一，
区分大小写，并被验证为跨平台文件名主干。
默认将条目 ID 写为小写 ASCII kebab-case。
这种风格易于在工具之间键入、引用、链接和比较。
当这些字符在常见文件系统中安全时，它可以使用空格、大写字母、标点和 Unicode。
它不得使用路径分隔符、控制字符、Windows 保留标点、
保留设备名称，或尾部空格或句点。
最多可以使用 252 个 UTF-8 字节，
以使最终的 `.md` 文件名保持在常见组件限制内。

每个条目有一个 YAML 元数据块和一个散文正文。
必需的元数据字段是 `name` 和 `desc`。
本仓库推荐 `category`、`belongs` 和 `refines`。
活跃的结构字段集配置在 `Sirno.toml` 中。
`frozen:` 字段通过 `sirno freeze ENTRY_ID` 将条目文件设为只读。
条目文件可以使用 LF 或 CRLF 行尾。
每个文件应只使用一种行尾风格，
以便保留字节位置的工具能让文件保持可预测。

条目应该足够聚焦以便就地阅读。
它可以陈述一个概念、类别、评审邻域、细化、不变量、
接口、实现承诺、可见证声明或叙事路线。

条目的正文应该是有用的散文，而不仅仅是一个标签。
它应该告诉未来的读者这个条目意味着什么，
为什么它值得一个稳定的名字，
以及它如何参与项目模型。
当条目描述一个局部实现承诺时，
正文应该解释持久的设计事实，而不是叙述最近的编辑。

元数据块承载工具必须精确读取的结构。
正文承载判断、示例和解释。
这种分工让 Sirno 保持简单。
它可以验证 ID 和结构字段，而无需假装理解散文的完整含义。

好的条目紧凑但不晦涩。
它们避免重复整篇专著，
但也给出足够的上下文，让读者可以在不打开十个文件的情况下跟踪查询结果。
如果一个概念依赖其他几个概念，
结构字段应该承载导航结构，
散文应该用普通语言解释局部含义。

当条目有仓库证据时，
其散文可以简要说明见证预期要展示什么。

[generated-footer-zh]: generated-footer-zh.md "Generated Footer"

---

> **Sirno generated links begin. Do not edit this section.**

- belongs (to):
  - [sirno-lake-zh](sirno-lake-zh.md)
- belongs (from): (none)

> **Sirno generated links end.**
