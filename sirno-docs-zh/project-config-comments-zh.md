---
name: 项目配置注释
desc: Sirno 在生成的项���配置字段旁写入的精确注释。
category:
  - concept-zh
refines:
  - project-config-zh
---

项目配置注释是 Sirno 渲染 `Sirno.toml` 时写入的字段级注释。

每条注释紧邻其描述的字段之上。
可选表注释仅在写入可选表时出现。
解析仍通过普通 TOML 规则忽略注释。

生成的注释为：

- `Markdown monograph path, resolved relative to this config file.`
- `Markdown entry lake path, resolved relative to this config file.`
- `Paths in lake that Sirno skips while reading, checking, querying, and generating links.`
- `Sirno Frost path, kept outside the public lake.`
- `Repository files, directories, or globs scanned for witness blocks.`
- `Witness delimiter regex pairs; each first capture group is the entry id.`
- `Canonical filename entry-id capture: ([^\x00-\x1F\x7F<>:"/\\|?*\r\n]+)`
- `Require generated footers to match current metadata during checks.`
- `Structural metadata field; link.to, link.from, and link.clique default to false.`

注释解释用途，而非模式权威。
Rust 配置类型和 TOML 解析器仍是模式边界。

[generated-footer-zh]: generated-footer-zh.md "Generated Footer"

---

> **Sirno generated links begin. Do not edit this section.**

- belongs (to): (none)
- belongs (from): (none)

> **Sirno generated links end.**
