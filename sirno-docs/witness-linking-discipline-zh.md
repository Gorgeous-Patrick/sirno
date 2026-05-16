---
name: 见证链接纪律
desc: 将 Lake 条目链接到仓库证据的智能体程序。
category:
  - meta-zh
belongs:
  - agent-skills-zh
---

见证链接将条目声明连接到使其可检查的仓库区域。

在接触代码之前阅读目标条目。
理解声明、其结构字段以及正文中关于证据应意味着什么的任何指引。
如果没有现有条目精确匹配需求，
先创建或提议一个紧凑条目，并将见证 ID 绑定到该确切声明。
复用一个近似条目 ID 让评审更不精确。

选择支持声明的最小区域。
在添加新见证之前用 `sirno witness ENTRY_ID --full` 检查当前见证。
优先选择单个项、测试用例、配置节或小的内聚块。
如果区域太宽，将其拆分为共享同一条目 ID 的更小块。
将块放在已配置的仓库成员内，
并为文件类型使用已配置的定界符语法。
当需要时更新条目散文，使其简要说明该区域展示什么，
并保持生成页脚不被触碰。

不要重复 `mosaika` 的工作。
Sirno 调用 `mosaika` 进行定界符匹配、区域提取和跨度处理；
Sirno 侧的工作消费该结构化输出并将其格式化为评审用。

变更证据后验证。
再次运行直接见证查询，
运行评审模式结构检查，
如果 Lake 元数据或链接变更则运行生成链接维护。
然后像人类一样阅读完整的见证输出：
它应显示简洁的范围、字面匹配的区域，且没有宽泛无关的代码。

[generated-footer-zh]: generated-footer-zh.md "Generated Footer"

---

> **Sirno generated links begin. Do not edit this section.**

- belongs (to):
  - [agent-skills-zh](agent-skills-zh.md)
- belongs (from): (none)

> **Sirno generated links end.**
