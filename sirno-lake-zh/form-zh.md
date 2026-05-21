---
name: 形式
desc: 项目知识在 Sirno 中所采取的一种形态。
category:
  - concept-zh
  - narrative-zh
belongs:
  - sirno-zh
---

Sirno 通过三种形式工作。

`mono` 是一份可选配置的 Markdown 文档。
它以可读的专著形式承载整个项目设计。

`sirno` 是一个配置的条目 Lake。
它包含带有精确元数据的紧凑 Markdown 条目。
当配置了 Sirno Frost 时，它通过一个单独的 `eter` frost 路径进行版本化，
因此一个 Lake 版本命名一个不可变的条目集。

`repo` 是仓库。
它包含源代码、测试、配置、生成文件、资源，
以及任何可以实现或见证设计的工件。
Sirno 仅在配置了 `[repo].members` 时扫描仓库见证。

形式不仅仅是存储位置。
它们是设计工作流中的角色。
专著为连续性而优化，
让读者可以按精心安排的顺序构建心智模型。
Lake 为可寻址性而优化，
让人或工具可以找到对局部变更重要的命名对象。
仓库为执行和证据而优化，
让设计承诺有具体的工件可供检查。

在 Lake 存在之前，
用户选择仓库还是专著承载更多权威。
一旦 Lake 建立，
Sirno 将其视为结构化的中间形式。

这种权威仍然可以通过深思熟虑的工作来修订。
降维让专著播种 Lake。
反射让实现发现更新 Lake。
提升让 Lake 重建整个项目叙事。
实现让条目指导实现。

保持三种形式分离防止一份文档试图同时服务所有读者。
专著可以保持流畅。
条目可以保持紧凑和命名。
仓库工件可以保持聚焦于行为，同时仍有地方指向意图。

[generated-footer-zh]: generated-footer-zh.md "Generated Footer"

---

> **Sirno generated links begin. Do not edit this section.**

- belongs (to):
  - [sirno-zh](sirno-zh.md)
- belongs (from): (none)

> **Sirno generated links end.**
