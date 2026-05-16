---
name: Lake 探索纪律
desc: 探索 Sirno 管理仓库的智能体程序，从 Lake 向外进行。
category:
  - meta-zh
belongs:
  - agent-skills-zh
---

探索从 Lake 向外读取 Sirno 管理的仓库。

Lake 是项目地图，见证块是条目声明的证据。
目标是一条小而扎实的路线：条目 ID、它们为何重要、见证位置，
以及它们指向的代码或文档。

遵循固定顺序。
定位活跃的 `Sirno.toml` 并读取其 lake、structural 和 repo 设置。
查询 Lake，从模糊开始以进行发现，当知道结构字段或条目 ID 时使用精确查询，
在收窄前阅读 `desc` 字段。
阅读少数最高信号的条目并跟踪它们的结构字段。
用 `sirno witness ENTRY_ID --full` 向 Sirno 索取证据。
检查被见证的区域和附近的代码，在 Lake 内使用 `sirno rg`，对仓库代码使用普通的 `rg`。
最后综合路线。

将见证视为证据，而非证明。
见证说明在哪里检查声明；它不表示代码正确。
当见证宽泛时，通读一次，然后收窄到最小的相关函数、测试或配��节，
并注明它是否会从拆分中受益。
当条目没有见证时，检查相关条目，搜索 ID 和关键术语，
并说明结果是纯文档、无见证还是未找到。

保持路线窄。
除非问题要求概览，否则避免阅读整个 Lake 或整个仓库。
在探索时不要添加或编辑见证块；
当任务从读取变为变更时切换到见证或编辑器程序。

报告扎实的发现。
好的结果命名咨询的条目、
塑造路线的描述、
见证文件和行范围、
检查的代码符号或文档、
已知的、推断的和仍不确定的内容，
以及有用的下一步检查步骤。
如果检查失败，报告阻塞原因并继续使用仍可安全检查的证据。

[generated-footer-zh]: generated-footer-zh.md "Generated Footer"

---

> **Sirno generated links begin. Do not edit this section.**

- belongs (to):
  - [agent-skills-zh](agent-skills-zh.md)
- belongs (from): (none)

> **Sirno generated links end.**
