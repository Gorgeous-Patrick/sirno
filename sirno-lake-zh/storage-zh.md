---
name: 存储
desc: 持有 Sirno 条目、快照、配置和仓库证据的存储表面。
category:
  - concept-zh
belongs:
  - sirno-zh
---

Sirno 存储是持有设计知识和操作状态的仓库表面集合。

公开的 Markdown Lake 是必需的可编辑工作形式。
已配置的专著是可选的。
已配置的仓库成员是可选的，启用见证查找。
私有的 frost 路径是可选的，通过 `eter` 管理。
`eter` 提供持久存储、索引、不可变快照、
字段历史、版本退役和垃圾回收。

存储表面保持分离。
Markdown 条目是人类面向的形式。
它们易于阅读、评审、diff 和编辑。
Sirno Frost 在该形式下提供持久的快照基底，
而不向条目元数据添加版本字段。
仓库见证保留在仓库工件中，
在那里它们可以展示实现条目声明的代码、测试、配置、生成文件或资源。

`Sirno.toml` 命名已配置的存储路径和策略。
`Sirno.lock.toml` 在配置了 Sirno Frost 时记录公开 Lake 的 frost 状态。
公开 Lake、私有 frost 路径、可选专著和仓库工件
保持为具有独立所有权规则的独立表面。

存储模型使 Sirno 具有持久状态而不使公开条目文件变得不透明。
接口在这些已配置的表面上操作。
它们应该保持可编辑的公开 Markdown、
私有快照存储和仓库证据之间的区分。

[generated-footer-zh]: generated-footer-zh.md "Generated Footer"

---

> **Sirno generated links begin. Do not edit this section.**

- belongs (to):
  - [sirno-zh](sirno-zh.md)
- belongs (from): (none)

> **Sirno generated links end.**
