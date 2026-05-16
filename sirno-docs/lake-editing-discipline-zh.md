---
name: Lake 编辑纪律
desc: 编辑 Sirno Lake 条目的智能体程序。
category:
  - meta-zh
belongs:
  - agent-skills-zh
---

Lake 编辑遵循固定程序，以保持 Lake 精确且可评审。

先读取。
读取仓库指令、`Sirno.toml`、已配置的专著（当存在时）和现有 Lake。
在修改任何东西之前决定设计权威；参见 `design-source-authority`。
在假设哪些命令存在之前检查当前 Sirno CLI。

编辑前先映射。
对每个候选条目，了解其 ID、name、desc、结构字段和见证状态。
使用 `sirno query` 查找概念和邻域，
使用 `sirno rg` 查找条目内的文字文本。
在重写匹配条目之前阅读它；不要仅从孤立的匹配行编辑。

通过工具创建。
用当前的条目创建命令创建缺失条目，以使 ID 验证和脚手架正确。
然后用直接、对读者友好的散文扩展或修订正文。
根据各自条目选择 `category`、`belongs` 和 `refines`，
当结构字段纯粹装饰性时将其省略。

区分原则与其应用。
元级原则陈述项目应如何被理解或开发。
应用它产生结构事实：
`category` 命名哪些条目，条目加入哪个 `belongs` 邻域。
这些事实存在于元数据和生成页脚中，而非新条目中。
仅当条目给未来工作一个原则及其结构边界尚未提供的句柄时才创建它。

保持生成页脚不被触碰。
元数据稳定后，运行生成链接维护，
然后运行编辑模式结构检查和评审模式检查。

验证可能被部分阻塞。
如果评审模式检查仅因本地编辑器或工具目录存在于 Lake 中而失败，
保留这些文件除非用户要求移除、
报告阻塞原因，
并仍然尽可能验证条目解析和元数据引用。

提交时窄范围暂存。
暂存已配置的 Lake、指向它的配置变更和直接相关的文档，
将无关代码或生成的编辑器状态单独放置。

[generated-footer-zh]: generated-footer-zh.md "Generated Footer"

---

> **Sirno generated links begin. Do not edit this section.**

- belongs (to):
  - [agent-skills-zh](agent-skills-zh.md)
- belongs (from): (none)

> **Sirno generated links end.**
