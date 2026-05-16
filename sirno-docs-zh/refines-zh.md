---
name: 细化
desc: 一个结构字段，从具体条目指向它使之更具体的更广泛条目。
category:
  - concept-zh
belongs:
  - structural-field-zh
---

`refines` 从更具体的条目指向它使之更具体的更广泛条目。

细化将高层设计转化为低层设计、
实现细节和可测试行为。
当前条目保持更广泛的声明附着其上，同时使其后果局部化。

细化链是一条递增具体性的路径。
它从一个压缩的概念开始，可以终止于仓库文本附近。

如果编程语言最清楚地表达了设计，
最终细化可能是一个 Markdown 代码块。

当条目回答"这种更广泛的设计在这里意味着什么？"这个问题时，使用 `refines`。
该字段保留了局部选择存在的理由。
一个低层条目可以细化一个概念，
一个元数据规则可以细化条目模型，
一个可测试行为可以细化一个宽泛的不变量。

更具体的条目指回更广泛的条目。
这让局部工作更容易：
从局部条目，读者可以爬回意图。
从广泛条目，生成或查询的元数据可以揭示阐述它的条目。

使用最接近的能解释当前条目设计压力的更广泛目标。
不要用 `refines` 来分组仅仅一起评审的条目。
对这种水平关系使用 `belongs`。

一个条目可以细化多个更广泛的条目。
这应在局部设计真正结合了几个思想时发生，
而非作者想要额外的交叉链接。
散文应解释组合的责任，以便未来读者能分辨为什么该字段存在。

[generated-footer-zh]: generated-footer-zh.md "Generated Footer"

---

> **Sirno generated links begin. Do not edit this section.**

- belongs (to):
  - [structural-field-zh](structural-field-zh.md)
- belongs (from): (none)

> **Sirno generated links end.**
