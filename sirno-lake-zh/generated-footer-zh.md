---
name: 生成页脚
desc: Sirno 拥有的页脚，将选定的元数据字段投影为链接。
category:
  - concept-zh
belongs:
  - generated-navigation-zh
---

Sirno 可以在条目底部生成和维护一个页脚。

生成页脚条目是生成导航区域的前门。
其局部细化定义了所有权边界和链接选择策略。

页脚由声明 Sirno 拥有该区域的哨兵界定。
人和工具应保持该区域不被触碰。

哨兵是人类可见的 Markdown 引用块。
生成的列表与两个哨兵之间用空行分隔。
这种形状防止 Markdown 渲染器将结束哨兵嵌套在列表之下。

当 Sirno 向非空正文追加生成页脚时，
它在生成区域之前插入一个水平分隔线，
除非正文已以分隔线结尾。

页脚为导航链接的外部工具投影元数据派生的结构。
它不是结构真值的来源。

生成页脚是一个互操作层。
一些编辑器和文档工具比元数据字段更自然地导航 Markdown 链接。
Sirno 可以将选定字段投影为链接，使这些工具能够参与 Lake。

生成的正文按已配置的结构字段分组。
每个启用的组出现在该区域中。
在一个字段内，
组按 `to`、`from`、`clique` 的顺序渲染。
每组是一个顶级 Markdown 列表项，
如 `- category (from):`、`- belongs (to):` 或 `- belongs (clique):`。
组的链接是缩进在组项下的子列表项。
没有链接的组内联渲染，如 `- belongs (from): (none)`。
如果没有生成链接组被启用，该区域包含 `(none)`。

页脚派生自元数据。
手动更改生成链接不会改变元数据。
更改元数据并重新生成页脚是正确路径。
哨兵使所有权边界在条目文件本身中可见。
frost 提交在写入条目快照之前移除生成页脚。
Sirno Frost 保留规范元数据和散文，
而非导航投影。

`[structural]` 链接策略控制哪些结构字段出现。

`sirno check` 在链接检查启用时报告陈旧的生成页脚区域。
`sirno gen-link` 创建或替换生成页脚区域。
`sirno gen-link --dry` 报告将变更的生成页脚区域而不写入文件。
`sirno gen-link delete` 移除它们。
变更命令将保护边界区域之外的散文保留在用户所有权下。
`sirno rg` 搜索 Lake 时将这些保护边界区域视为仅包含空白。
这让文字搜索聚焦于编写的元数据和散文。
`sirno rg --with-generated-footer` 在它们作为搜索目标时包括投影链接。

生成页脚应保持乏味。
它们的工作是使页面的边缘对工具有用，
而不是成为设计散文的另一个地方。

[generated-footer-zh]: generated-footer-zh.md "Generated Footer"

---

> **Sirno generated links begin. Do not edit this section.**

- belongs (to):
  - [generated-navigation-zh](generated-navigation-zh.md)
- belongs (from): (none)

> **Sirno generated links end.**
