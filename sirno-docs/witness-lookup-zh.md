---
name: 见证查找
desc: 由 mosaika 支持的扫描，按条目 ID 解析见证块。
category:
  - concept-zh
refines:
  - witness-zh
---

见证查找通过用 `mosaika` 扫描已配置的仓库成员来解析仓库证据。
CLI 在扫描前先在活跃 Lake 中解析请求的条目 ID。
缺失的条目在读取仓库成员之前就失败了。

`[repo].members` 在配置了见证查找时定义仓库工件表面。
文件成员直接扫描。
目录成员递归扫描。
Glob 成员展开为匹配的文件。

Sirno 将每个成员文件投影为一个记录见证块的 `mosaika` 变换。
开始和结束定界符都捕获条目 ID。
当定界符 ID 不同时，Sirno 拒绝该见证块。
定界符正则对来自必需的 `[[witness.delimiters]]` 配置表。
生成的配置写入标准语法，
它接受 `//` 行注释和隐藏的 Markdown HTML 注释。
这些标准正则共享一个针对类似文件名条目 ID 的规范捕获组。
Sirno 将日志流解析为以条目 ID 为键的见证记录。
存储的定界符跨度排除前导缩进。
完整输出显示匹配块跨越的每一行，
并保留匹配的文本。

查找路径保持见证语法在条目散文之外。
条目保持为设计声明。
仓库工件承载见证这些声明的精确源跨度。

[generated-footer-zh]: generated-footer-zh.md "Generated Footer"

---

> **Sirno generated links begin. Do not edit this section.**

- belongs (to): (none)
- belongs (from): (none)

> **Sirno generated links end.**
