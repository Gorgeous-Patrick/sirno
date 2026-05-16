---
name: 查询
desc: 通过模糊文本和精确结构谓词选择 Sirno 条目。
category:
  - concept-zh
belongs:
  - sirno-lake-zh
refines:
  - interfaces-zh
---

查询从公开 Lake 中选择已解析的条目，
当配置了 Sirno Frost 时，则从一个 frost 版本中选择。

它读取条目 ID、元数据和正文。
生成页脚是导航的投影，
不是查询的结构输入。
当未提供版本时，
查询读取公开 Lake。

默认查询模式是模糊文本查询。
它匹配条目的 ID、name、desc 和 body。
它同样匹配由结构字段命名的条目的 ID、name 和 `desc` 值。

模糊查询用于回忆。
用户可以在不先选择确切结构字段的情况下搜索附近语言。
每个文本词必须在展开的条目文本中某处匹配。

精确查询使用重复的 `--exact field=entry-id` 标志。
精确结构字段跨字段是合取的，在一个字段内是析取的。
两个 `--exact category=...` 值意味着任一类别。
一个 `category` 精确谓词加上一个 `refines` 精确谓词要求两个字段都匹配。

查询输出是呈现。
`sirno query --fields` 接受逗号分隔的字段列表。
可打印字段是 `id`、`name`、`path` 和 `desc`。
当未提供字段时，
查询选择 `id,path,name`。
`--format json` 打印一个 JSON 对象数组，包含选定字段。
`--format human` 将相同选定字段打印为对齐的表格供交互使用。
当未提供格式时，
查询使用 `json`。

[generated-footer-zh]: generated-footer-zh.md "Generated Footer"

---

> **Sirno generated links begin. Do not edit this section.**

- belongs (to):
  - [sirno-lake-zh](sirno-lake-zh.md)
- belongs (from): (none)

> **Sirno generated links end.**
