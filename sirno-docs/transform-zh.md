---
name: 变换
desc: 命名 Sirno 形式之间移动的概念。
category:
  - concept-zh
  - narrative-zh
---

变换命名 Sirno 形式之间的一类工作。

Sirno 使用四个变换名称：
`lower`、`raise`、`realize` 和 `reflect`。

它们的直接名称同样有用：
`mono-to-sirno`、`sirno-to-mono`、`sirno-to-repo` 和 `repo-to-sirno`。

变换是为人类、LLM、技能、CLI 接口和 MCP 工具准备的词汇。
它们描述连贯的工作，而不要求每次变换都是一次性命令。

变换名称让设计工作更易于请求和评审。
不需要每次都去说"把这个文档拆成小块"，
用户可以要求将专著降维到 Lake 中。
不需要说"根据这次仓库变更更新设计笔记"，
用户可以要求将仓库反射回条目中。

四个变换形成一个循环：
`mono` 降维到 `sirno`，
`sirno` 实现到 `repo`，
`repo` 反射回 `sirno`，
`sirno` 提升到 `mono`。
循环是概念性的，不是自动权威。
每次变换仍应在判断当前真值来源后执行。

这个词汇也帮助技能保持聚焦。
降维技能应在创建条目时保留叙事意图。
实现技能应在编辑代码前检查条目。
反射技能应记录从实现中学到的持久设计事实。
提升技能应组成可读的专著。

[generated-footer-zh]: generated-footer-zh.md "Generated Footer"

---

> **Sirno generated links begin. Do not edit this section.**

- belongs (to): (none)
- belongs (from): (none)

> **Sirno generated links end.**
