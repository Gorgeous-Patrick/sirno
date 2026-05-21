---
name: 生成导航
desc: Sirno 拥有的生成页脚导航的评审邻域。
category:
  - concept-zh
---

生成导航是 Sirno 拥有的页脚机制的评审前门。

它汇集了产生和界定生成链接的部件：
`generated-footer` 是 Sirno 从元数据投影的页脚，
`generated-footer-ownership` 是 Sirno 可以变更的保护边界区域，
`generated-link-policy` 选择哪些结构字段变为链接。

这些部件一起评审。
对页脚渲染、所有权边界或链接选择的变更
通常会约束其他部件，因此此条目为它们提供一个邻域。

`generated-footer` 仍是所有权和策略细化的更广泛声明。
该邻域是独立水平视图：
`refines` 说明一个部件特化什么，
这里的 `belongs` 说明哪些部件一起评审。

[generated-footer-zh]: generated-footer-zh.md "Generated Footer"

---

> **Sirno generated links begin. Do not edit this section.**

- belongs (to): (none)
- belongs (from):
  - [generated-footer-ownership-zh](generated-footer-ownership-zh.md)
  - [generated-footer-zh](generated-footer-zh.md)
  - [generated-link-policy-zh](generated-link-policy-zh.md)

> **Sirno generated links end.**
