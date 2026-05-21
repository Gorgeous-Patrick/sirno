---
name: 叙事序列化器
desc: 将叙事会话转化为 Lake 条目的确定性契约。
category:
  - concept-zh
refines:
  - interactive-narrative-session-zh
---

叙事序列化器通过确定性契约将完成的会话转化为 Lake 条目。

会话首先记录紧凑的笔记，而非转录。
笔记命名读者和任务、
使路线有用的设计压力、
使下一个概念值得相遇的拉力或张力、
已知和缺失的术语、
步骤的有序路线、
持久反馈、
推迟的细节，
以及余味短语、句柄或条目 ID。
每个路线步骤记录一个条目 ID 或提议的 ID、其角色、其满足的前置知识，
以及被推迟到条目正文的细节。

序列化器的输入是一个独立、更小的形状。
它携带一个 `id`、`name`、`desc`、一个从字段名到条目 ID 列表的 `structural` 映射，
以及作为段落字符串列表的 `body`。
笔记是路线的脚手架；输入是变成文件的内容。

契约持有以下不变量。
条目 ID 是小写 kebab-case。
`name`、`desc` 和 `frozen` 是保留元数据，从不作为结构字段写入。
结构字段严格按提供的样子写入，以给定顺序，
因为它们的顺序是用户管理的，Sirno 按该顺序渲染已配置的表面。
空字段被省略，`witness:` 仅在仓库证据存在时添加。
序列化是确定性的，除非显式请求覆盖否则拒绝覆盖已有条目；
干运行可以预览条目而不写入。

具象化条目正文回答一组固定的问题：
路线服务谁，
什么设计压力使其有用，
什么拉力使下一个概念值得相遇，
什么必须先理解，
哪些条目承载有序路线，
什么局部细节被推迟，
之后留下什么短语或句柄，
以及什么持久反馈塑造了路线。
正文命名条目并解释其顺序；它不复制它们的定义。

序列化器是这个契约的实现。
契约是持久的设计事实，存在于此以使会话工具能从中重建。

[generated-footer-zh]: generated-footer-zh.md "Generated Footer"

---

> **Sirno generated links begin. Do not edit this section.**

- belongs (to): (none)
- belongs (from): (none)

> **Sirno generated links end.**
