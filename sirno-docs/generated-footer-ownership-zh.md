---
name: 生成页脚所有权
desc: Sirno 拥有的导航与用户拥有的散文之间的保护边界。
category:
  - concept-zh
belongs:
  - generated-navigation-zh
refines:
  - generated-footer-zh
---

生成页脚所有权是 Sirno 仅变更保护边界区域的规则。

开始和结束哨兵是被拥有区域的一部分。
创建、替换、检查或删除生成链接的命令首先验证该区域。
格式错误、缺失、重复或颠倒的哨兵是结构错误。

生成链接区域之外的散文保持为用户拥有。
变更生成链接的命令保留该散文。
frost 提交在写入快照前移除生成链接区域，
因此 Sirno Frost 保留规范元数据和散文，而非导航投影。

[generated-footer-zh]: generated-footer-zh.md "Generated Footer"

---

> **Sirno generated links begin. Do not edit this section.**

- belongs (to):
  - [generated-navigation-zh](generated-navigation-zh.md)
- belongs (from): (none)

> **Sirno generated links end.**
