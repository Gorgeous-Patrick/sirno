---
name: sirno-config-writer
description: >-
  Write, repair, or validate Sirno.toml. Use for creating or editing a Sirno project config,
  adding repository witness members, configuring structural fields, repairing missing generated
  comments, or explaining the exact Sirno.toml schema and validation workflow.
---

# Sirno Config Writer

## Purpose

Use this skill when writing, repairing, or validating `Sirno.toml`.
The config marks a Sirno-managed repository and records the configured lake,
witness syntax, optional storage surfaces, and policy tables.

This full skill text is served as the MCP resource `sirno://skills/sirno-config-writer`.
It follows the `config-writing-discipline` lake entry.

## MCP Project Resolution

When using Sirno through MCP, call `sirno_cwd` with the repository root before project tools
if the server started without `--config`.
Project tools resolve `Sirno.toml` on every project tool call from the current server cwd.
Call `sirno_cwd` again before switching projects in the same server process.

## Workflow

Prefer Sirno MCP tools for routine project config writes:

- `sirno_lake_init`
- `sirno_lake_move`
- `sirno_frost_init`
- `sirno_frost_move`

Use manual TOML edits only when the current MCP tools cannot express the intended schema change.
Read the existing `Sirno.toml` before editing.
Preserve user-authored structural field order.

This skill may call CLI `sirno util config --fix` directly.
This is a narrow exception for deterministic `Sirno.toml` comment canonicalization.
It does not expose utility commands through MCP
or authorize other skills to call `sirno util` commands.

After writing `Sirno.toml`, run:

```sh
sirno util config --fix
```

Then run `sirno_lake_check mode=review`.
If lake metadata changed, also run `sirno_lake_render` before the review check.

## Minimal Config

Prefer generated config output.
When writing `Sirno.toml` manually, start from this valid minimal form:

```toml
[lake]
path = "sirno-docs"

[witness]
```

After writing a manual config,
run `sirno util config --fix` to add canonical comments and default rendered sections.

## Required Tables

`[lake].path` is required.
It names the public Markdown entry lake.
Configured `lake` paths resolve relative to the config file unless absolute.

`[witness]` is required.
It may contain zero or more `[[witness.delimiters]]` tables.
An empty delimiter list disables repository witness lookup.
Each delimiter table has `begin` and `end` regex fields.
Each regex must be non-empty, valid, capture the entry id as its first capture group,
and not match empty text.
Generated configs write the standard `//` line-comment delimiter syntax
and hidden Markdown HTML-comment delimiter syntax.

## Optional Tables

`[frost].path` names the optional private Frost path.
Configured `frost` paths resolve relative to the config file unless absolute.
The Frost path must not equal, contain, or sit inside the public lake path.

`[repo].members` lists files, directories, or globs scanned for witness blocks.
Members are relative to the config file.
They must not be absolute and must not escape upward.
Add repo members only for artifacts that are truly part of the witness surface.

`[lake].ignore` lists lake-root-relative paths Sirno skips while reading, checking,
querying, and rendering generated footers.
Ignored paths must not be absolute and must not escape upward.

`[check]` is optional.
Omitting `[check]` or an individual check flag leaves that check enabled.
`[check].render` controls generated-footer freshness checks.
`[check].structural-inhabitance` controls the check that every configured structural field
has a matching entry.
It follows the same default-on rule.
When a check flag is present,
`sirno util config --fix` writes its canonical comment.

`[tutorial]` enables tutorial text by table presence.
`frost_commit_tide` controls the tutorial for Frost commits blocked by open tide workitems.
`frost_bootstrap_tide` adds first-snapshot context to that tutorial.
Both fields default to true when the table is present.

## Structural Fields

Each structural field is configured by one `[structural.FIELD]` subtable.
`FIELD` must be non-empty, single-line, contain no comma,
and must not be `name`, `desc`, or `frozen`.
The field should also name the lake entry that documents it.
When `[check].structural-inhabitance` is true,
checks require that documentation entry to exist.

Each structural field may define `to`, `from`, and `clique` edge tables.
Each edge may set:

```toml
render = true
ripple = { lake = true, frost = true }
```

Omitted structural edge values are false.
`to` follows outgoing metadata targets.
`from` follows incoming entries that name the current entry as a target.
`clique` follows entries that share a target in that field.
Structural field order is user-managed and must be preserved.

## Comments

Generated config files include concise comments that describe each written field.
When canonical comments are missing,
run `sirno util config --fix` to rewrite the file through the canonical renderer.

The Rust config types and TOML parser are the schema boundary.
Do not invent unknown fields.
Do not keep compatibility shims for old config spellings.

## Handoffs

Use `sirno-witness` when deciding whether a repository file should become evidence.
Use `sirno-editor` when the config change requires new or revised lake entries.
Use `sirno-skill-synthesizer` when adding this or another skill to the MCP resource roster
or installed wrapper set.
