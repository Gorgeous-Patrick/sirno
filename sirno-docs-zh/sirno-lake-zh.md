---
name: Sirno Lake
desc: 由紧凑命名的设计条目组成的已配置目录。
category:
  - concept-zh
belongs:
  - sirno-zh
refines:
  - form-zh
---

Sirno Lake 是由 Markdown 条目组成的已配置目录。
常见约定是 `docs/`。

这个名字让 Sirno 的 Cirno 影响可见。
它呼应了 Misty Lake，并将项目设计源重塑为一座知识雾之湖：
静谧可读，但又足够结构化，使变更留下可见的涟漪。

`Sirno.toml` 在 `[lake].path` 下记录 lake 路径。
`sirno move PATH` 重命名已配置的 lake 目录，
并将新路径写回 `[lake].path`。
`sirno mv PATH` 是其短形式。
move 拒绝替换已有的目标目录。

Lake 是人类可读的中间表示：
文本优先，足够结构化以供工具使用，
足够紧凑以供人和智能体局部检查。
当配置了 Sirno Frost 时，
其冻结状态通过单独的 `eter` frost 路径进行版本化，
因此一个版本命名一个不可变的条目集。

每个条目是一个普通的 Markdown 文件，带有 YAML 元数据块和散文正文。
文件名主干是结构字段、生成页脚和见证查找使用的稳定 ID。
按定义，ID 类似文件名。
小写 kebab-case 是可读 Lake 的约定，而非验证边界。

一旦建立，Lake 就是首选的结构化设计源。

Lake 应该像一组命名良好的设计卡片。
每张卡片有足够的散文使其单独有用，
但也通过元数据参与更大的图。
这个图有意保持小型：
分类、归属、细化和见证。
这个小集合足以导航，而不会将 Lake 变成一个独立的数据库语言。
`frozen:` 标记添加文件级保护状态，
使一个公开条目可以保持只读并从 Frost 提交中排除。

Lake 也是一个协作边界。
人可以直接编辑条目。
CLI 可以检查其元数据和链接。
智能体可以在变更代码前查询几个相关条目。
编辑器可以使用生成页脚来暴露导航。
所有这些形式使用相同的文件名和元数据。

Lake 是工作形式。
只有当配置了 Sirno Frost 并且 Sirno 将 Lake 冻结到 frost 路径时，
直接编辑才会变成冻结版本。

Lake 根目录下的一些文件可能属于相邻工具。
`[lake].ignore` 列出相对于 Lake 根目录的路径。
被忽略的路径覆盖该路径本身及其后代。
这让 Lake 可以包含编辑器状态如 `.obsidian`，
而不使该状态成为 Sirno 条目集的一部分。

Lake 应避免变成词汇表或积压清单。
词汇表条目可能定义了一个词但没有承载设计压力。
积压项可能描述了工作但没有保留其背后的概念。
条目应该命名持久的项目知识：
为什么一个承诺存在，
它如何连接到更广泛的设计，
以及当证据存在时，实现证据应该在哪里找到。

[generated-footer-zh]: generated-footer-zh.md "Generated Footer"

---

> **Sirno generated links begin. Do not edit this section.**

- belongs (to):
  - [sirno-zh](sirno-zh.md)
- belongs (from):
  - [design-source-authority-zh](design-source-authority-zh.md)
  - [entry-zh](entry-zh.md)
  - [metadata-zh](metadata-zh.md)
  - [project-config-zh](project-config-zh.md)
  - [query-zh](query-zh.md)
  - [ripple-zh](ripple-zh.md)
  - [structural-check-zh](structural-check-zh.md)

> **Sirno generated links end.**
