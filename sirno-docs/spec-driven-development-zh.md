---
name: 规范驱动开发
desc: 一种从显式行为承诺开始的开发风格。
category:
  - concept-zh
belongs:
  - development-style-zh
refines:
  - concept-driven-development-zh
---

规范驱动开发始于显式的行为承诺。

规范说明在代码选择如何使之成立之前必须成立什么。
它可以描述接口、状态转换、不变量、文件形状、命令契约
或可观察行为。
有用的部分是精确性：
工作有一个实现可以满足、修订或拒绝的命名声明。

在概念驱动开发中，
规范锐化一个概念。
概念给规范在项目词汇中一个稳定的位置。
规范给概念一个局部的行为边界。

规范驱动的工作应保持足够小以便测试或检查。
如果规范变成了一篇宽泛的文章，
将其降维为更小的条目或在其使之具体化的概念下细化它。
如果实现揭示了一个更好的边界，
将该边界反射回 Lake，使规范保持为一个活的设计对象。

[generated-footer-zh]: generated-footer-zh.md "Generated Footer"

---

> **Sirno generated links begin. Do not edit this section.**

- belongs (to):
  - [development-style-zh](development-style-zh.md)
- belongs (from): (none)

> **Sirno generated links end.**
