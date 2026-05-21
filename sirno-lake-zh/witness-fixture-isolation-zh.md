---
name: 见证夹具隔离
desc: 见证查找的测试夹具避免依赖自身的定界符文本。
category:
  - concept-zh
belongs:
  - witness-zh
refines:
  - witness-lookup-zh
---

见证夹具隔离防止见证查找的测试通过源字面量自我满足。

需要实际仓库见证夹具的测试在写入临时文件之前
从较小部分组装定界符文本。
扫描器仍然在夹具文件中看到一个真正的见证块。
Rust 测试源不将该夹具块暴露为独立的字符串字面量。

仅格式化见证记录的测试使用中性注释文本。
它们验证范围渲染、标记选择、正文保留
和记录间距，而不依赖见证语法。

仓库见证注释在测试模块中仍然是有效证据。
它们是仓库元数据，不是夹具数据。
隔离规则适用于测试为扫描器或格式化器创建的字符串和生成文件。

[generated-footer-zh]: generated-footer-zh.md "Generated Footer"

---

> **Sirno generated links begin. Do not edit this section.**

- belongs (to):
  - [witness-zh](witness-zh.md)
- belongs (from): (none)

> **Sirno generated links end.**
