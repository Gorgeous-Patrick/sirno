---
name: 见证
desc: 设计条目的仓库证据。
category:
  - concept-zh
belongs:
  - sirno-zh
---

见证是条目声明的仓库证据。

见证条目是仓库证据的前门。
其局部条目涵盖查找行为和见证所在的仓库表面。

Sirno 机械地发现见证状态。
它通过 `mosaika` 按条目 ID 查询见证。
智能体应使用 `sirno witness ENTRY_ID --full` 来检查证据，
而不是从散文或生成链接中推断。

见证可以是源代码、测试、配置、生成文件、资源，
或任何 `mosaika` 可以界定和查询的仓库工件。
当测试本身就是相关代码时，测试可以见证一个条目。

仓库工件由 `[repo].members` 选择。
目录成员被递归扫描。
仓库见证块以 `sirno:witness:<entry-id>:begin` 打开，
以 `sirno:witness:<entry-id>:end` 关闭。
开头和结尾的条目 ID 必须匹配。
Rust 和其他行注释文件可以用 `//` 写入哨兵。
Markdown 文件可以将其写为隐藏的 HTML 注释。
这是生成配置所写的标准语法。
标准分隔符正则表达式使用一个规范捕获组来匹配类似文件名的条目 ID。
它捕获所有可以放入哨兵冒号之间的合法 ID。
解析出的条目 ID 然后应用剩余的跨平台文件名检查。
项目可以通过 `[[witness.delimiters]]` 覆盖分隔符正则对，
当另一个仓库表面需要不同的标记形状时。

条目正文可以解释如何找到或解释证据，作为后备指导。
约定是条目 ID 加上仓库见证块。

仓库见证将散文连接到工件，而不合并二者。
条目用项目语言陈述设计声明。
见证块标识应当检查的工件区域。
条目 ID 将它们联系在一起。

当声明应该在仓库中可评审时，证据是有用的。
一个实现模块可以见证一个接口决策。
一个测试可以见证一个行为属性。
一个配置文件可以见证一个存储或工具边界。
一个生成资源可以见证一个可见或打包的结果。

当仓库证据支持一个相关但不同的声明时，
创建一个新条目并见证那个确切的声明。
复用一个近似条目 ID 会让评审不够精确。

如果一个条目描述了一个还没有仓库证据的想法，
不添加见证更清晰。
如果证据存在但难以解释，
条目正文可以解释评审者应该寻找什么。
条目 ID 仍然是查询键。

本条目的仓库见证应该展示 Sirno 如何表示见证记录、跨度和
在 `mosaika` 找到界定的仓库区域后接受的定界符样式。

[generated-footer-zh]: generated-footer-zh.md "Generated Footer"

---

> **Sirno generated links begin. Do not edit this section.**

- belongs (to):
  - [sirno-zh](sirno-zh.md)
- belongs (from):
  - [repo-zh](repo-zh.md)
  - [witness-fixture-isolation-zh](witness-fixture-isolation-zh.md)

> **Sirno generated links end.**
