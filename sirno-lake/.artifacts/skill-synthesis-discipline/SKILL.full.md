---
name: sirno-skill-synthesizer
description: >-
  Rebuild packaged Sirno skill wrappers and MCP skill resources from the active project's lake,
  when that project owns such a skill surface.
---

# Sirno Skill Synthesizer

## Purpose

Use this skill when the active repository owns packaged Sirno skills or MCP skill resources
and those packages should be rebuilt from Sirno entries.
The active project's lake is the source of truth.
Generated skill files are reproducible surfaces.

If the active repository defines no skill roster, no skill artifacts, or no package locations,
say so instead of inventing a layout.

This full skill text is served as `sirno://skills/sirno-skill-synthesizer`.
It follows the project's skill-synthesis discipline.

## Project Binding

Bind the MCP server to the repository before calling project tools.
Call `sirno_cwd` with the repository root when the server may not already be there.
Project tools resolve `Sirno.toml` from the server current working directory on each call.
Call `sirno_cwd` again before switching projects in the same server process.

## Lake-First Rule

Skill synthesis is a repository edit.
Read and update the governing entries before changing artifacts,
installed packages, MCP resource lists, bundle constants, or tests.
Discover those surfaces from the active project.
Do not assume a particular repository layout unless the active project's entries say to use it.
When a skill resource, wrapper, and the lake disagree,
correct the artifact or wrapper, never the lake.

## Synthesis Workflow

1. Map the roster.
   Read `Sirno.toml` and query the active lake for the skill roster.
   A project may use `agent-skills` as the front door and rostered `*-discipline` entries
   as skill sources.
   Follow the active project's roster instead of assuming those ids exist.
   Treat other method entries as shared method unless the roster says they ship as skills.

2. Discover the package surface.
   Read the roster and the discipline entries for artifact paths,
   installed package paths, and MCP resource names.
   If the project has code that embeds or serves those resources, inspect that code before editing.
   Keep one source entry per package.

3. Inspect the current MCP tools.
   List the Sirno MCP tools available in the active server before writing tool names into a skill.
   A full resource that names a missing tool is worse than one that only names the procedure.

4. Render full resources.
   The full resource operationalizes its discipline plus the shared method it depends on.
   Add nothing the lake does not commit.
   Preserve shared documentation habits such as reader-aware bullets, diagrams,
   conceptual ordering, and declarative precision when the full resource touches prose.
   Include failure paths for missing sources, unavailable tools, blocked validation,
   absent evidence, and design changes that must be internalized into the lake.
   Defer project configuration maintenance to the editor skill rather than copying its checklist.

5. Render wrappers.
   The wrapper keeps the same frontmatter as the full resource.
   Its body only states that it is a wrapper and requires the matching MCP resource before work begins.
   Copy the wrapper artifact exactly into the installed package.

6. Update exposed surfaces.
   Update any MCP resource list, bundled wrapper list, tests, docs, or install metadata
   that the active project uses.
   Keep human CLI utility maintenance as a CLI surface, not an MCP procedure.
   When utility maintenance is needed, report the human CLI action rather than turning it into an
   agent step.

## Validation

Run `sirno_lake_render` after lake metadata changes.
Run `sirno_lake_check` in edit mode and review mode.
Confirm each generated `SKILL.md` and `SKILL.full.md` has valid frontmatter.
Compare installed wrappers against their artifacts byte for byte.
Confirm every rostered discipline has the artifacts, resources, and packages
that the active project expects.
Confirm every installed `sirno-*` package traces back to a rostered discipline.

Run the formatter, tests, and checks that fit the active repository.

If validation is blocked, report the blocker and the remaining risk.
If a resource would need behavior the lake does not commit,
leave it out and report the missing design instead of inventing it.
