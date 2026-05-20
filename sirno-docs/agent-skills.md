---
name: Agent Skills
desc: The Sirno agent skill set and the handoffs between its skills.
category:
  - meta
belongs:
  - sirno
prerequisite:
  - methodology
  - interfaces
---

Sirno ships an agent skill set that renders this lake's method into operational procedure.

There are six Sirno skills.
The config-writer skill writes, repairs, and validates `Sirno.toml`.
The editor skill creates, revises, and reorganizes entries,
and internalizes durable design knowledge into the lake.
The explorer skill reads a Sirno-managed repository from the lake outward to locate design
and its evidence.
The witness skill links lake entries to repository evidence and keeps that evidence precise.
The narrative-session skill conducts an adaptive route through lake knowledge
and materializes it as a narrative entry when the route should persist.
The skill-synthesizer skill rebuilds the MCP skill resources and packaged wrappers
from discipline entries,
so the skill set stays a reproducible surface of the method rather than prose that can drift.
`src/surface/context.rs` bundles the packaged wrappers as compile-time constants
from the lake-owned `SKILL.md` artifact files.
Project initialization installs those bundled wrappers by default.
Human CLI operator maintenance can refresh wrappers and detect installed wrapper drift.
Each rostered Sirno discipline owns two skill artifacts.
`SKILL.full.md` is the full Markdown skill text embedded by `src/mcp.rs`
and served as a `sirno://skills/sirno-*` MCP resource.
`SKILL.md` is the packaged wrapper copied word-for-word into `.agents/skills/sirno-*/SKILL.md`.
The wrapper tells an agent to read the MCP resource before doing skill work.
When Claude skill integration is selected,
Sirno links each installed `.agents/skills/sirno-*` package directory into `.claude/skills`.
The link is an adjacent integration point.
The packaged wrapper under `.agents/skills` remains the owned skill package.
MCP-hosted Sirno skills must bind the active project through the server current working directory.
When a Sirno MCP server starts without `--config`,
call its `sirno_cwd` tool with the repository root before project tools.
Project tools resolve `Sirno.toml` on every project tool call from the current server cwd.
Call `sirno_cwd` again before switching projects in the same server process.

This entry is the review front door for those skills.
The durable procedure each skill encodes lives in its own discipline entry,
so a skill can be rebuilt from the lake rather than only from its packaged wrapper.
Each Sirno discipline entry names its target `.agents/skills/sirno-*/SKILL.md` package path.
The discipline entries are `config-writing-discipline`, `lake-editing-discipline`,
`lake-exploration-discipline`, `witness-linking-discipline`,
`narrative-session-discipline`, and `skill-synthesis-discipline`.

The skills hand off rather than overlap.
Exploration switches to the witness skill when the task changes from reading evidence
to creating or refining it.
Exploration switches to the editor skill when the task changes from reading entries to editing them.
Skills switch to the config-writer skill when the task changes to writing `Sirno.toml`.
The editor skill defers to the repository documentation-writing skills for `README`, `DESIGN`,
and `METHODOLOGY` prose, because those documents have their own roles and style.
The `design-doc-writer-skill` entry documents the adjacent meta-management skill
for design documents.
It is documented in this lake as a method input,
not as part of the six packaged Sirno skills.
`design-doc-writer-skill` contributes reusable design-document habits:
read the whole design document,
order sections by conceptual dependency and scope,
write declarative, dry, precise prose,
prefer positive definitions over defensive framing,
and evaluate the result as a reader before and after editing.
It also owns `.artifacts/design-doc-writer-skill/SKILL.full.md`,
copied exactly from `.agents/skills/design-doc-writer/SKILL.md`.
`src/mcp.rs` embeds that artifact as `sirno://skills/design-doc-writer`.
Sirno skill work uses those habits only as the default design-document method.
When a repository has its own design-document skill or documented manner,
use that instead.
The synthesis skill checks the Sirno skill roster
and reports any discipline, MCP resource, wrapper, or package that no longer has a counterpart.
Repository skill maintenance is a local human CLI utility surface.
MCP serves skill resources for agents.
It does not expose any `sirno util` commands,
including wrapper listing, checking, or installation.
Agents maintain skill artifacts and installed wrappers as explicit repository files.
Utility commands copy exact bundled wrapper constants;
they do not ask a model to rewrite the skill text.
When an agent discovers that utility maintenance is needed,
it should report the needed human CLI action instead of treating it as an MCP operation.
The config-writer skill is the only exception:
it may call CLI `sirno util config --fix` to canonicalize `Sirno.toml` comments.
That exception belongs to config ownership and does not add utility commands to MCP.

A full skill resource is an operational rendering of lake method, not a separate authority.
When a resource or wrapper and the lake disagree, the lake and `Sirno.toml` win,
and the artifact should be corrected.
Failure handling belongs in the full resource.
A wrapper should only direct the agent to the MCP resource,
so installed skills stay small while the full method remains source-controlled.

---

> **Sirno generated links begin. Do not edit this section.**

- belongs (to):
  - [sirno](sirno.md)
- belongs (from):
  - [config-writing-discipline](config-writing-discipline.md)
  - [design-doc-writer-skill](design-doc-writer-skill.md)
  - [lake-editing-discipline](lake-editing-discipline.md)
  - [lake-exploration-discipline](lake-exploration-discipline.md)
  - [narrative-session-discipline](narrative-session-discipline.md)
  - [skill-synthesis-discipline](skill-synthesis-discipline.md)
  - [witness-linking-discipline](witness-linking-discipline.md)

> **Sirno generated links end.**
