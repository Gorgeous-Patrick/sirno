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

When using Sirno through MCP, call `cwd` with the repository root before project tools
if the server started without `--config`.
Project tools resolve `Sirno.toml` on every project tool call from the current server cwd.
Call `cwd` again before switching projects in the same server process.

## Workflow

Prefer Sirno commands for routine config writes:

```sh
cargo run -- init
cargo run -- lake init [PATH]
cargo run -- lake move PATH
cargo run -- frost init [PATH]
cargo run -- frost move PATH
cargo run -- util config --fix
```

Use manual TOML edits only when the current CLI cannot express the intended schema change.
Read the existing `Sirno.toml` before editing.
Preserve user-authored structural field order.

After writing `Sirno.toml`, run:

```sh
cargo run -- util config --fix
cargo run -- util config
cargo run -- check --mode review
```

If lake metadata changed, run `cargo run -- render` before the review check.

## Minimal Config

Prefer generated config output.
When writing `Sirno.toml` manually, start from this valid minimal form:

```toml
[lake]
path = "sirno-docs"

[witness]
[[witness.delimiters]]
begin = '(?m)^[ \t]*//[ \t]*sirno:witness:([^\x00-\x1F\x7F<>:"/\\|?*.,\r\n]+):begin'
end = '(?m)^[ \t]*//[ \t]*sirno:witness:([^\x00-\x1F\x7F<>:"/\\|?*.,\r\n]+):end'

[[witness.delimiters]]
begin = '(?m)^[ \t]*<!--[ \t]*sirno:witness:([^\x00-\x1F\x7F<>:"/\\|?*.,\r\n]+):begin[ \t]*-->'
end = '(?m)^[ \t]*<!--[ \t]*sirno:witness:([^\x00-\x1F\x7F<>:"/\\|?*.,\r\n]+):end[ \t]*-->'
```

Run `cargo run -- util config --fix` after writing it to add canonical comments
and default rendered sections.

## Required Tables

`[lake].path` is required.
It names the public Markdown entry lake.
Configured `lake` paths resolve relative to the config file unless absolute.

`[witness]` is required.
It must contain at least one `[[witness.delimiters]]` table.
Each delimiter table has `begin` and `end` regex fields.
Each regex must be non-empty, valid, capture the entry id as its first capture group,
and not match empty text.
Generated configs write the standard `//` line-comment delimiter syntax
and hidden Markdown HTML-comment delimiter syntax.

## Optional Tables

`[mono].path` names the optional Markdown monograph.
Configured `mono` paths resolve relative to the config file unless absolute.

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

`[check].render` controls generated-footer freshness checks.
It defaults to true.

`[tutorial]` enables tutorial text by table presence.
`frost_commit_tide` controls the tutorial for Frost commits blocked by open tide workitems.
`frost_bootstrap_tide` adds first-snapshot context to that tutorial.
Both fields default to true when the table is present.

## Structural Fields

Each structural field is configured by one `[structural.FIELD]` subtable.
`FIELD` must be non-empty, single-line, contain no comma,
and must not be `name`, `desc`, or `frozen`.
The field should also name the lake entry that documents it.

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
Run `cargo run -- util config` to check whether canonical comments are present.
Run `cargo run -- util config --fix` to rewrite the file through the canonical renderer
when comments are missing.

The Rust config types and TOML parser are the schema boundary.
Do not invent unknown fields.
Do not keep compatibility shims for old config spellings.

## Handoffs

Use `sirno-witness` when deciding whether a repository file should become evidence.
Use `sirno-editor` when the config change requires new or revised lake entries.
Use `sirno-skill-synthesizer` when adding this or another skill to the MCP resource roster
or installed wrapper set.
