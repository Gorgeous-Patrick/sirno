---
name: 元数据
desc: 承载 Sirno 条目结构的精确 YAML 模式。
category:
  - concept-zh
belongs:
  - sirno-lake-zh
---

元数据是承载 Sirno 结构的精确模式。

每个条目都有一个 YAML 元数据块。
必需字段是 `name` 和 `desc`，
两者都是纯字符串。

已配置的结构字段是可选的。
本仓库配置了 `category`、`belongs` 和 `refines`。
当存在时，它们始终是列表，
其值是条目 ID。
它们的字段顺序是用户编写的元数据。
Sirno 在解析、渲染和通过 Sirno Frost 移动条目时保留它。

`frozen:` 声明条目文件为只读，
在 Sirno Frost 可以提交它之前必须融化。
它在写入时不带值。

操作结构仅由元数据形成。
散文链接可能对读者和外部工具有帮助，
但它们不定义 Sirno 结构。

元数据块应该小而稳定。
它是条目中工具必须无需解释即可读取的部分。
这就是为什么必需字段是纯字符串，
而结构字段是 ID 列表。

正文可以解释细微差别，
但元数据不得要求解析散文。
如果工具需要知道一个条目细化了另一个，
已配置的结构元数据必须说明。
如果智能体需要检查条目的仓库证据，
它应该运行 `sirno witness ENTRY_ID --full`。

规范的条目形状如下：

```yaml
---
name: 概念
desc: 一个命名的思想，用于压缩项目知识。
category:
  - concept-zh
---
```

模式保持必需的标量字段小巧。
当 `[structural]` 配置了一个字段时，新的列表值元数据可以成为结构的。
未配置的列表值元数据字段作为检查警告保持可见。

[generated-footer-zh]: generated-footer-zh.md "Generated Footer"

---

> **Sirno generated links begin. Do not edit this section.**

- belongs (to):
  - [sirno-lake-zh](sirno-lake-zh.md)
- belongs (from): (none)

> **Sirno generated links end.**
