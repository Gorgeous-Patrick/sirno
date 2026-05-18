---
name: sirno-skill-synthesizer
description: >-
  Rebuild the packaged Sirno agent skill wrappers and MCP skill resources from the lake. Use for
  regenerating or auditing `.agents/skills/sirno-*/SKILL.md`, editing a `meta`-categorized
  discipline entry, finding a skill resource or wrapper that has drifted from the Sirno Lake,
  or adding a new skill to the `agent-skills` roster.
---

# Sirno Skill Synthesizer

## Purpose

Use this skill to rebuild the packaged agent skill wrappers and MCP skill resources
from the Sirno Lake.
The lake's `meta`-categorized entries are the source of truth.
The full `SKILL.full.md` artifact is the resource text served by MCP.
The `SKILL.md` artifact is the installed wrapper that tells an agent to read that resource.

The procedure this skill encodes lives in the `skill-synthesis-discipline` entry.
This full skill text is served as the MCP resource `sirno://skills/sirno-skill-synthesizer`.
It follows the `skill-synthesis-discipline` lake entry.

## Core Principles

Read the lake before writing any skill.
Read `Sirno.toml` for the lake path,
`agent-skills` for the skill roster and the handoffs between skills,
and every `meta`-categorized entry for the method the skills render.

Separate disciplines from shared method.
A `meta` entry named by the Sirno skill roster and ending in `-discipline`
is a skill source.
The other `meta` entries carry vocabulary, principles, perspective, and design authority.
They are cross-cutting method every skill must respect, not skills in their own right.
Repository-specific design-document skills or documented prose methods are the first method input
when Sirno skill work touches design prose.
If a repository has none,
default to `sirno://skills/design-doc-writer` from `design-doc-writer-skill`.
Their reusable content is reader evaluation,
conceptual ordering,
declarative precision,
and whole-document coherence.
They do not become skill sources unless the Sirno skill roster adds them.
They may own full MCP resource artifacts without rendering installed Sirno wrappers.

Map each discipline to one MCP resource and one wrapper package.
A skill discipline owns one `SKILL.full.md` resource artifact,
one `SKILL.md` wrapper artifact,
and one `.agents/skills/sirno-<role>/SKILL.md` installed wrapper package.
The target package path is written in the discipline body until the project defines
a structural field for skill packages.
Keep the existing skill directory name.
Do not invent a new role unless `agent-skills` adds one to the roster.
Every rostered Sirno discipline should have both artifacts and a package,
and every `sirno-*` package should trace back to a discipline.

Split full procedure from wrapper.
The full artifact operationalizes its discipline plus the shared `meta` method it depends on.
Add nothing the lake does not commit, and drop nothing the discipline requires.
Include the discipline's failure paths in the full artifact.
The wrapper artifact keeps the same frontmatter and only instructs the agent to read
the corresponding MCP resource before working.
When a skill resource, wrapper, and the lake disagree, correct the artifact or wrapper,
never the lake.

## Synthesis Workflow

Read the roster and the method.

```sh
cargo run -- query --has category=meta --columns id,desc
cargo run -- query --has belongs=agent-skills --columns id,path,desc
```

Read `agent-skills` and each discipline entry in full before rendering.
Read a matched entry rather than working from the `desc` line alone.

Classify the `meta` entries.
The rostered `*-discipline` entries are skill sources, one resource and wrapper each.
Adjacent skill entries in the same neighborhood are shared method inputs.
The remaining `meta` entries are shared method.
Fold the shared method into the skills that depend on it,
for example design authority and the structural-field model into the editor skill.

Render each full resource artifact.
Write `sirno-docs/.artifacts/<discipline>/SKILL.full.md` with valid frontmatter:

```yaml
---
name: skill-directory-id
description: >-
  When to use the skill and the triggers that should invoke it.
---
```

Then write the body as direct procedure:
purpose, core principles, an ordered workflow, and validation.
Turn durable procedure into concrete steps and current commands.
Include failure paths for missing sources, unavailable commands, blocked validation,
absent evidence, and design changes that must reflect back into the lake.

Render each wrapper artifact.
Write `sirno-docs/.artifacts/<discipline>/SKILL.md` with the same frontmatter as the full artifact.
The body should state that it is a wrapper,
name the matching `sirno://skills/sirno-*` MCP resource,
and require the agent to read that resource before doing the work.
Do not duplicate the full procedure in the wrapper.

Routine project initialization installs wrappers by default.
Use the Sirno utility command to refresh wrappers after initialization.

```sh
cargo run -- util skills init
```

Use `cargo run -- util skills check` at review boundaries.
Use `cargo run -- util skills list` when auditing bundled wrapper constants and targets.
The command uses compile-time constants from the `SKILL.md` artifacts.
It does not parse the lake at runtime.

Inspect the current Sirno CLI before writing commands into a skill.

```sh
cargo run -- --help
```

A full skill resource that names a missing command is worse than one that only names the procedure.
Use `cargo run -- ...` or `target/debug/sirno ...` according to the repository state.
Keep config-writing procedure in `sirno://skills/sirno-config-writer`.
Other resources should hand off `Sirno.toml` edits instead of copying the schema checklist.

## Validation

If lake metadata or links changed, run render maintenance:

```sh
cargo run -- render
```

Then run the review-mode structural check:

```sh
cargo run -- check --mode review
```

Confirm each `SKILL.full.md` and `SKILL.md` has valid frontmatter.
Confirm the disciplines, resources, wrappers, and packages still correspond one to one:
no `belongs: agent-skills` discipline without artifacts and a package,
and no `sirno-*` package without a discipline.
Run `cargo run -- util skills check` to verify installed wrappers match artifacts.

If a package exists without a discipline,
either add the missing discipline to the lake or report the package as outside the reproducible set.
If a discipline exists without a package,
create the package only when the roster says the skill should ship.
If a full skill resource would need behavior the lake does not commit,
leave that behavior out and report the missing design instead of inventing it.

Report the resources and wrappers rebuilt,
the entries they were rendered from,
and any discipline, resource, wrapper, or package that no longer has a counterpart.
