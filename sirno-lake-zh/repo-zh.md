---
name: 仓库
desc: 实现和见证设计条目的仓库工件。
category:
  - concept-zh
belongs:
  - witness-zh
refines:
  - form-zh
---

`repo` 形式是仓库。

它包括源文件、测试、配置、生成文件、资源，
以及其他实现设计决策的工件。

仓库工件可以通过 `mosaika` 见证条目。
Sirno 使用条目 ID 作为见证查询键，
保持设计名称和见证块连接而不将块语法嵌入条目散文。
`[repo].members` 定义 Sirno 在配置时扫描的仓库工件表面。
文件成员直接扫描，
目录成员递归扫描。
见证块以 `sirno:witness:<entry-id>:begin` 打开，
以 `sirno:witness:<entry-id>:end` 关闭。
两个哨兵命名同一个条目 ID。
行注释工件可以用 `//` 携带哨兵。
Markdown 工件可以将其携带为隐藏的 HTML 注释。
当项目需要不同的定界符时，可以通过 `[[witness.delimiters]]` 替换该标准语法。

仓库是设计变得昂贵的地方——这是在有用的意义上。
名称、不变量、解析器选择、存储边界、用户界面、
测试和生成资源都做出了未来工作必须尊重或修订的承诺。
Sirno 不要求每一行代码都携带一个设计条目。
它要求重要的承诺有一个可以在引入它们的编辑之后继续存在的名字。

仓库见证使这个名字变得具体。
一个条目可以陈述一个声明，
见证块可以显示声明在哪里实现、测试、配置或生成。
见证块属于仓库工件。
条目保留设计语言。
共享的键是条目 ID。

这使仓库工件和文档耦合而不让任何一方变得尴尬。
源代码不需要为每个设计概念写长篇叙事注释。
条目不需要复制会漂移的源代码片段。
评审可以通过询问哪个条目解释了一段代码承诺，
以及哪个仓库工件见证了一个条目，在两者之间移动。

[generated-footer-zh]: generated-footer-zh.md "Generated Footer"

---

> **Sirno generated links begin. Do not edit this section.**

- belongs (to):
  - [witness-zh](witness-zh.md)
- belongs (from): (none)

> **Sirno generated links end.**
