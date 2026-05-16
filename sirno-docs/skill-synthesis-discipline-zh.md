---
name: 技能合成纪律
desc: 从 meta 分类的 Lake 条目重建打包智能体技能的智能体程序。
category:
  - meta-zh
belongs:
  - agent-skills-zh
---

技能合成从 Lake 的 `meta` 分类条目重建打包的智能体技能。

先读取来源。
读取 `Sirno.toml` 获取 Lake 路径，
然后读取 `agent-skills` 获取技能名册和技能间的交接，
然后通过 `sirno query` 读取每个 `meta` 分类的条目。
Lake 是权威的；打包技能是其可复现表面。

将 discipline 与共享方法分开。
一个 `meta` 条目如果有 `belongs: agent-skills` 并陈述一个智能体程序，则是一个技能来源。
其他 `meta` 条目承载词汇、原则、视角和设计权威。
它们是每个技能必须尊重的横切方法，
而非技能本身。

将每个 discipline 映射到一个包。
一个技能 discipline 渲染恰好一个 `.agents/skills/sirno-<role>/SKILL.md`。
保留现有的技能目录名，不要发明新角色，
除非 `agent-skills` 将其添加到名册中。
每个 `belongs: agent-skills` discipline 应该有一个包，
每个包应该追溯到一个 discipline。

渲染，而非重新解释。
打包技能将其 discipline 及其依赖的共享 `meta` 方法操作化。
前言 `name` 是技能目录 ID；
`description` 陈述何时使用该技能以及应触发它的触发条件。
正文将持久程序转化为具体步骤和当前命令。
不添加 Lake 未承诺的内容，不遗漏 discipline 要求的内容。

在将命令写入技能之前检查当前 Sirno CLI。
命名了不存在命令的技能比只命名程序的技能更糟糕。

保持 Lake 为真值来源。
当打包技能与 Lake 不一致时，
纠正技能，绝不纠正 Lake。
此 discipline 本身也是一个技能来源；
合成器以与重建其他技能相同的方式重建自己的包。

写入后验证。
如果 Lake 元数据变更，运行生成链接维护，
然后运行评审模式结构检查。
确认每个 SKILL.md 有有效的前言，
并且 discipline 和包仍然一一对应。

[generated-footer-zh]: generated-footer-zh.md "Generated Footer"

---

> **Sirno generated links begin. Do not edit this section.**

- belongs (to):
  - [agent-skills-zh](agent-skills-zh.md)
- belongs (from): (none)

> **Sirno generated links end.**
