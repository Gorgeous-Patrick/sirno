---
name: 类别
desc: 一个结构字段，通过其他条目对条目进行分类。
category:
  - category-zh
  - meta-zh
  - concept-zh
belongs:
  - structural-field-zh
---

`category` 通过其他条目对条目进行分类。

类别本身是条目。
这让项目词汇保持开放和文档化，而非由 Sirno 固定。

类别目标必须是可作为种类使用的，
而可作为种类使用本身就是一个文档化的属性。
`category` 条目分类那些可作为类别目标使用的条目。
因此，类别目标应被 `category` 分类。
这包括 `category` 本身以及初始化的 `concept`、`narrative` 和 `meta` 条目。
该标记是自应用的，这使得类别词汇在其自身规则下封闭。

保留的 `locked` 字段未来可能保护项目视为受控的条目或区域。

当被分类的条目应被读作某命名种类的一个实例时，使用 `category`。
被 `meta` 分类的条目应定义项目的原则、词汇或文档方法。
被 `category` 分类的条目本身可作为类别目标使用。
被 `concept` 分类的条目应定义一个压缩的思想。
被 `narrative` 分类的条目应记录或命名一条穿越概念的路线。

因为类别是条目，
它们的含义可以在它们所分类的同一个 Lake 中记录。
这避免了实现中的隐藏枚举成为唯一的真值来源。
项目可以通过添加条目来增长词汇。

类别应保持语义的而非装饰的。
如果一个标签仅帮助按主题浏览，
`belongs` 可能更合适。
如果一个条目使另一个条目更具体，
`refines` 是更精确的字段。
类别字段在告诉读者他们正在看什么种类的对象时最有用。

[generated-footer-zh]: generated-footer-zh.md "Generated Footer"

---

> **Sirno generated links begin. Do not edit this section.**

- belongs (to):
  - [structural-field-zh](structural-field-zh.md)
- belongs (from):
  - [concept-zh](concept-zh.md)
  - [meta-zh](meta-zh.md)
  - [narrative-zh](narrative-zh.md)

> **Sirno generated links end.**
