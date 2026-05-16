---
name: 专著
desc: 承载整个设计叙事的已配置专著。
category:
  - concept-zh
  - narrative-zh
belongs:
  - sirno-zh
refines:
  - form-zh
---

`mono` 是可选的已配置长篇 Markdown 文档。
常见约定是 `DESIGN.md`。
本仓库目前将其第一条叙事路线保存在 Lake 中，即 `introduction`。

它是项目专著：
为想要一次性读完整个设计的人提供一条可读的叙事。

专著是 Sirno Lake 之外的普通 Markdown。
它不携带 Sirno 条目元数据。

当配置时，专著成为当前条目的提升叙事视图。
它应该保留一条穿越项目的路线，
而不是变成条目散文的目录列表。

专著通过编排思想来获得其位置。
它可以先引入问题，
然后是项目模型，
然后让模型可用的模式和操作。
这种顺序对人类读者很重要。
条目可以按多种顺序浏览，
但专著应该让一条好的路线感觉自然。

健康的专著是选择性的。
它命名重要的概念，
解释它们如何配合，
将密集的局部细节留在条目中。
当一个条目的局部设计内容增长到足以打断主要叙事时，
专著可以总结该条目并信任 Lake 承载细节。

这让 `mono` 在降维前后都有用。
降维前，它可能是预期设计的最佳陈述。
降维后，它成为 Lake 上的组合视图。
在两种情况下，它都应该读起来像为人写成的文档，
而不是每个已知事实的机械导出。

[generated-footer-zh]: generated-footer-zh.md "Generated Footer"

---

> **Sirno generated links begin. Do not edit this section.**

- belongs (to):
  - [sirno-zh](sirno-zh.md)
- belongs (from): (none)

> **Sirno generated links end.**
