---
name: 生成链接策略
desc: 选择哪些结构链接出现在生成页脚中的配置。
category:
  - concept-zh
belongs:
  - generated-navigation-zh
refines:
  - generated-footer-zh
---

生成链接策略决定哪些已配置的结构部分出现在生成页脚中。

`[structural]` 列出 Sirno 视为结构的元数据字段。
每个字段键有一个 `link` 策略。
`link.to` 生成到目标的出站链接。
`link.from` 生成来自来源的入站链接。
`link.clique` 通过该字段中的共享目标生成团链接。
所有三个布尔值是可选的，
缺失的布尔值意味着 false。

团投影不改变直接的 `from` 或 `to` 投影。
当为一个字段启用时，每个目标引发团边：
目标链接到其成员，
成员链接到目标并互相链接。
禁用时，仅渲染已配置的直接结构字段部分。

此策略是配置，不是条目数据。
更改它会变更生成的导航表面而不改变结构元数据。

[generated-footer-zh]: generated-footer-zh.md "Generated Footer"

---

> **Sirno generated links begin. Do not edit this section.**

- belongs (to):
  - [generated-navigation-zh](generated-navigation-zh.md)
- belongs (from): (none)

> **Sirno generated links end.**
