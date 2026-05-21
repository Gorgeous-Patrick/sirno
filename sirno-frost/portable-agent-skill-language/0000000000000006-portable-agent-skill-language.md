---
desc: A rule that packaged Sirno skills speak to any active Sirno-managed repository.
lifecycle: Active
name: Portable Agent Skill Language
structural:
  category:
  - meta
  belongs:
  - agent-skills
  prerequisite:
  - agent-skills
---

Packaged Sirno skills are portable instructions for any active Sirno-managed repository.

A skill should speak from the active project perspective.
Use `the active repository`, `the configured lake`, and `the current project` for runtime work.
Do not assume this source repository, its `sirno-lake/` path, or its self-hosted entries exist
in the user's project.

Skill examples should use configured paths, placeholders, or discovered entry addresses.
A resource may name Sirno source files only when it describes how the Sirno source repository
bundles and serves the skill itself.
The procedure the user follows should remain useful in a fresh repository with a different lake path,
different entries, and a different documentation method.

When a skill needs project-specific guidance,
it should discover that guidance through `Sirno.toml`, repository instructions,
entry queries, and direct reads of the active lake.
