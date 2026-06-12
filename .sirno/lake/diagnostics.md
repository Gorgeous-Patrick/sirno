---
name: Diagnostics
desc: Structured findings reported while inspecting Sirno project material.
category:
  - concept
belongs:
  - interfaces
refines:
  - structural-check
---

Diagnostics are structured findings collected while Sirno inspects project material.

A diagnostic belongs to a report.
It has a stable code, severity, message, optional source location,
optional domain fields such as entry, metadata field, and target,
and optional repair help.

Diagnostics accumulate while an operation can keep inspecting related material.
The selected command boundary decides whether the collected findings block success.
For example, review-mode checks treat selected findings as errors,
while edit-mode checks may keep them as warnings.

Command errors remain hard-stop failures.
They cover unreadable files, failed writes, invalid command input,
failed process execution, and other cases where the operation cannot continue safely.
A command error states what went wrong through its message and cause chain,
and carries a concise repair hint when a next step is known.
The message names the cause; the hint names the fix.
Human and MCP output render the hint as a `help` line after the cause chain.

Human CLI output renders diagnostics as concise lines with codes, locations, and help when known.
JSON and MCP output keep diagnostics as structured data.
This lets scripts and agents inspect the same findings that a human sees without parsing prose.
