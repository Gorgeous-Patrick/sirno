---
artifacts:
- SKILL.full.md
- SKILL.md
desc: The agent procedure for writing Sirno.toml.
lifecycle: Active
name: Config Writing Discipline
structural:
  category:
  - meta
  belongs:
  - agent-skills
---

Config writing keeps `Sirno.toml` valid, commented, and aligned with the lake model.
Its full MCP resource text lives in `.artifacts/config-writing-discipline/SKILL.full.md`
and is embedded by `src/mcp.rs` as `sirno://skills/sirno-config-writer`.
Its packaged wrapper lives in `.artifacts/config-writing-discipline/SKILL.md`
and renders to `.agents/skills/sirno-config-writer/SKILL.md`.

Use this discipline whenever work writes or repairs `Sirno.toml`.
Prefer Sirno commands for routine config changes:
`sirno init`, `sirno lake init`, `sirno lake move`, `sirno frost init`,
`sirno frost move`, and `sirno util config --fix`.
Manual edits are for schema changes that the current CLI cannot express.

The config schema is explicit.
`[lake].path` is required.
`[witness]` is required and must contain at least one `[[witness.delimiters]]` table.
Each delimiter table has `begin` and `end` regex fields.
Those regexes must be non-empty, valid, capture the entry id first, and not match empty text.
Generated configs write the standard line-comment and Markdown-comment witness syntax.
A valid minimal manual config is:

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

Optional tables describe configured surfaces and policy.
`[mono].path` names the optional monograph.
`[frost].path` names the optional private Frost path.
`[repo].members` names files, directories, or globs scanned for witness blocks.
`[lake].ignore` names lake-root-relative paths Sirno skips.
`[check].render` controls generated-footer freshness checks and defaults to true.
The presence of `[tutorial]` enables tutorial text,
with `frost_commit_tide` and `frost_bootstrap_tide` defaulting to true.

Paths have different roots.
Configured `mono`, `lake`, and `frost` paths resolve relative to the config file unless absolute.
`[lake].ignore` paths are relative to the lake root and cannot be absolute or escape upward.
`[repo].members` paths and globs are relative to the config file and cannot be absolute
or escape upward.
The Frost path must not equal, contain, or sit inside the public lake path.

Structural fields are `[structural.FIELD]` subtables.
`FIELD` must be non-empty, single-line, contain no comma,
and must not be `name`, `desc`, or `frozen`.
It should also name the lake entry that documents the field.
Each structural field may define `to`, `from`, and `clique` edge tables.
Each edge may set `render = true` and `ripple = { lake = true, frost = true }`.
Omitted structural edge values are false.
Structural field order is user-managed and must be preserved.

After writing `Sirno.toml`, run `sirno util config --fix`
so canonical comments are present.
Then run `sirno util config` and `sirno check --mode review`.
When lake metadata changed, run render maintenance before the review check.
