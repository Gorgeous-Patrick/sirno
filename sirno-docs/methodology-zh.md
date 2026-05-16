---
name: 方法论
desc: 在 Sirno Lake 内行动的工作指南。
category:
  - concept-zh
  - narrative-zh
belongs:
  - narrative-zh
refines:
  - concept-driven-development-zh
  - transform-zh
  - witness-zh
---

Sirno 是一种让设计和实现保持对话的方法。

在写作前选择视角。
用 `Sirno` 指代工具和项目模型。
用 `a Sirno-managed project` 指代任何应用 Sirno 的项目。
用 `this repository` 指代 Sirno 的实现工作区。
用 `this lake` 指代 `sirno-docs/`，
即描述 Sirno 的自托管 Sirno Lake。

方法是纪律性的簿记。
它给人、智能体、工具和编辑器相同的命名设计对象来检查。
它不决定一个设计是否好。
它不证明代码满足一个条目。
它使相关对象更易于命名、连接、修订和见证。

从 Lake 开始。
本仓库将其设计源保存在 `sirno-docs/` 中。
当你需要穿越项目的第一条路线时先读 `introduction`。
然后跟随类别、`belongs`、`refines` 和见证到局部设计。

在工作变得局部之前命名事物。
条目应该足以就地阅读，
且足够持久以在使其有用的编辑之后存活。
它可以命名一个概念、结构字段、细化、不变量、
实现承诺或叙事路线。

用 `category` 表示种类。
用 `belongs` 表示评审局部性。
用 `refines` 表示语义收窄。
当仓库包含条目声明的证据时，使用仓库见证块。
当结构字段不能改善导航、评审或问责时，将其省略。
运行 `sirno witness ENTRY_ID --full` 并阅读条目散文以了解证据应意味着什么。

用 `meta` 表示项目面向 Sirno 的文档方法。
`meta` 条目应回答项目希望其文档如何发展：
术语、拆分规则、叙事习惯、评审期望或面向智能体的指导。

当意图对下一个局部变更来说过于宽泛时进行降维。
降维将叙事设计转化为紧凑的 Lake 条目。
它应该在赋予未来工作稳定句柄的同时，
保留使叙事可读的路线。

从命名对象实现。
在编辑代码前，
阅读治理该工作的条目。
检查它们的 `belongs`、`refines` 和见证。
实现应能回答哪个条目解释了一项重要的承诺。

在仓库变更尚新时反射。
当实现改变了一个表示、
收窄了一个不变量、
引入了一个边界、
使解释失效、
或揭示了更清晰的局部设计时进行反射。
被反射的散文应记录持久的设计事实，
而非叙述整个编辑。

仅在项目需要 Lake 之外的长篇叙事时进行提升。
提升将条目组成可读的专著。
它不是拼接。
它选择一条路线，
一次引入术语，
将密集的局部细节留在条目中。

见证重要的声明。
见证可以是源代码、测试、配置、生成文件或资源。
Sirno 通过 `mosaika` 按条目 ID 查询见证。
条目陈述设计声明。
见证块标识要检查的仓库区域。
条目散文应简要说明该仓库区域预期展示什么。

让 Sirno 维护生成页脚。
生成区域由哨兵界定并属于 Sirno。
元数据仍是结构真值的来源。
页脚存在是为了导航和互操作。

在评审边界检查。
在编辑期间，一些结构问题可以保持为警告。
在评审边界，悬挂的结构 ID 和命名了缺失条目的见证块应为错误。
检查确认结构。
它们不替代关于含义的判断。

将规划视为 Sirno 的一种用途，而非核心原语。
工作表在有用时可以表示为普通条目。
这些条目可以像 Lake 的其他部分一样使用类别、`belongs`、`refines` 和见证。

习惯很简单。
命名事物。
写条目。
仅当分类有帮助时才分类。
当共享主题值得一个前门时将其置于评审邻域中。
当宽泛设计需要局部形式时细化它。
当仓库包含其证据时见证它。

Sirno 保持结构就绪。
人和智能体保持意义鲜活。

[generated-footer-zh]: generated-footer-zh.md "Generated Footer"

---

> **Sirno generated links begin. Do not edit this section.**

- belongs (to):
  - [narrative-zh](narrative-zh.md)
- belongs (from): (none)

> **Sirno generated links end.**
