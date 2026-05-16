---
name: 反射
desc: 从已变更的仓库工件回到 Sirno 条目的运动。
category:
  - concept-zh
  - narrative-zh
belongs:
  - transform-cycle-zh
refines:
  - transform-zh
---

`reflect` 从 `repo` 移动到 `sirno`。

反射记录在实现过程中学到的持久设计事实。
当仓库工作改变了表示、收窄了不变量、
引入了边界、使解释失效、
或揭示了比当前条目记录的更清晰的局部设计时进行反射。

反射应在仓库变更尚新时发生。
被反射的条目记录未来工作需要的设计事实，
而非编辑的完整叙述。

好的反射会问仓库现在知道了什么 Lake 也应该知道的东西。
函数重命名可能不重要。
新的存储边界、解析器不变量、CLI 契约或测试理由通常重要。
被反射的散文应命名稳定的设计事实，
然后在结构有用时通过元数据将其连接到现有条目。

反射应避免将 Lake 变成变更日志。
提交历史可以解释编辑的顺序。
条目 Lake 应该解释在编辑中幸存的设计。
如果一个变更是探索性的且后来被丢弃，
它可能不值得反射。
如果一个变更改变了未来工作应如何推理，
它应该在理由仍然清晰时被反射。

反射防止专著和 Lake 变成仪式性的。
它给实现一种改进设计模型的方式，
而不仅仅是遵从它。

[generated-footer-zh]: generated-footer-zh.md "Generated Footer"

---

> **Sirno generated links begin. Do not edit this section.**

- belongs (to):
  - [transform-cycle-zh](transform-cycle-zh.md)
- belongs (from): (none)

> **Sirno generated links end.**
