---
name: 叙事会话纪律
desc: 进行和具象化叙事会话的智能体程序。
category:
  - meta-zh
belongs:
  - agent-skills-zh
---

叙事会话为一位读者或一项任务构建穿越 Lake 知识的自适应路线。

先读取路线来源。
读取 `Sirno.toml` 获取 Lake 路径，
然后读取 Lake 的 narrative、introduction 和 methodology 条目，
以及用户命名或任务隐含的任何条目。

维护一个小型的私有会话状态。
它持有读者和任务、
使路线有用的设计压力、
使下一个概念值得相遇的拉力或张力、
已知术语和缺失的前置知识、
有序的条目路线、
用户反馈和更正、
推迟的细节、
余味短语或句柄，
以及具象化时预期的叙事条目 ID。

以短片段循环。
解释下一个概念或路线选择，
仅在答案改变下一步时才请求反馈，
当用户表现出困惑、厌倦、紧急或更清晰的目标时修订路线，
并命名什么前移了、什么后移了以及为什么。
优先选择能解锁更好排序的问题，而非仅感到交互性的问题；
当下一步清晰且用户想要势头时，继续并陈述假设。

设计路线使准确的概念在正确的时间到达。
在解释前展示张力，
在完整模型前给出干净的第一口，
通过示例、对比和后果添加质感，
保持序列紧凑，
尊重读者的能动性，
并留下读者可以复用的余味。

当路线应指导未来的上手或评审时进行具象化。
产物是按序列化器契约构建的叙事条目；参见 `narrative-serializer`。
具象化后，运行生成链接维护和编辑模式结构检查。
最后命名产物、条目路径和主要排序选择，
并确认路线保留了拉力、干净的第一口和余味。

[generated-footer-zh]: generated-footer-zh.md "Generated Footer"

---

> **Sirno generated links begin. Do not edit this section.**

- belongs (to):
  - [agent-skills-zh](agent-skills-zh.md)
- belongs (from): (none)

> **Sirno generated links end.**
