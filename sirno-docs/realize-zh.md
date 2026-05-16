---
name: 实现
desc: 从 Sirno 条目到仓库实现的运动。
category:
  - concept-zh
  - narrative-zh
belongs:
  - transform-cycle-zh
refines:
  - transform-zh
---

`realize` 从 `sirno` 移动到 `repo`。

实现使用条目来指导实现。
在编辑代码前，阅读治理该工作的条目，
遵循它们的 category、belongs 和 refines 结构，
并检查任何被见证的仓库区域。

一个实现步骤应该能够回答哪个条目解释了一个局部设计承诺。
不是每行都需要自己的条目，
但重要的承诺需要一个名义位置。

实现是命名设计变成行为的地方。
条目 Lake 应该告诉实现者什么重要：
哪个概念正在被具体化，
哪个字段或不变量必须保留，
以及在编辑前应该检查哪些现有见证。

仓库变更应保持对条目的诚实。
如果条目仍然正确，
实现可以在这个名称下进行。
如果实现揭示条目不完整或误导，
工作应包括反射，以便 Lake 从仓库中学习。

这让实现成为一种双向纪律。
Lake 指导仓库工作，
仓库工作可以暴露对 Lake 的压力。
重要之处在于局部实现不脱离设计意图。
未来的读者应该能够询问为什么一段代码存在，
并找到赋予该承诺名称的条目。

[generated-footer-zh]: generated-footer-zh.md "Generated Footer"

---

> **Sirno generated links begin. Do not edit this section.**

- belongs (to):
  - [transform-cycle-zh](transform-cycle-zh.md)
- belongs (from): (none)

> **Sirno generated links end.**
