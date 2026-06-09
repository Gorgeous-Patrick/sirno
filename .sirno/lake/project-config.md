---
name: Project Config
desc: The Sirno.toml file that marks and configures a Sirno-managed repository.
category:
  - concept
belongs:
  - sirno
prerequisite:
  - repo
  - reservoir
---

`Sirno.toml` marks a *repository* as Sirno-managed.

The file configures the reservoir path and the project-level policies
that Sirno needs before it can read the lake.
It may also configure repository witness lookup,
Git-backed upstream lakes,
check flags,
and optional tutorial output.

The main tables are:

| Table or field | Meaning |
|---|---|
| `[lake].path` | Names the reservoir path. |
| `[lake].ignore` | Lists adjacent reservoir paths Sirno should skip. |
| `[upstreams.DOMAIN]` | Declares one Git-backed upstream lake. |
| `[repo].members` | Lists repository paths or globs scanned for witness blocks. |
| `[[witness.delimiters]]` | Configures one witness delimiter syntax. |
| `[check]` | Overrides project check flags. |
| `[tutorial]` | Enables extra CLI tutorial text for recoverable failures. |

Path selection follows three rules:

- Relative paths are resolved from the directory that contains `Sirno.toml`.
- The CLI `--lake-path PATH` option can override the reservoir path for one command.
- Sirno-owned control files live in `.sirno/` next to `Sirno.toml`.

`[lake].ignore` lists paths relative to the reservoir root.
Sirno skips those paths and their descendants while reading, checking,
querying, and rendering reservoir entries.
Ignored paths are for adjacent tool state, not for entries.

`[upstreams.DOMAIN]` declares a Git-backed upstream lake under an explicit local domain.
`DOMAIN` is an entry atom and becomes the glacier entry-address prefix.
Each upstream has `git = "SOURCE"` and exactly one of `branch`, `tag`, or `rev`.
`SOURCE` is a remote Git URL or local Git repository source accepted by Git.
`project` optionally names the upstream project root inside the Git tree;
it defaults to `.`.
`manifest` optionally names the upstream project config manifest relative to `project`;
it defaults to `Sirno.toml`.
`mist` optionally names the upstream mist to crystallize.

`[repo].members` lists paths and globs relative to `Sirno.toml`.
File members are scanned directly.
Directory members are scanned recursively.
Glob members may match files or directories.
Only intended witness surfaces should be listed.

`[[witness.delimiters]]` configures one witness delimiter syntax.
Each delimiter table has `begin` and `end` regex fields.
Each regex should capture the entry address as its first capture group.
Sirno rejects empty, invalid, captureless, or empty-matching delimiter regexes.
An empty delimiter list disables repository witness lookup.

`[check]` is optional.
Omitting the table or an individual check flag leaves that check enabled.
`[check].render` controls generated-footer freshness checks for checked entry directories.
Malformed generated-link sentinels remain errors,
because malformed sentinels make Sirno ownership ambiguous.

`[tutorial]` is optional.
When present,
Sirno shows enabled tutorials after matching recoverable command failures.
`[tutorial].anchor_update_tide` explains an anchor update blocked by open Tide workitems.
`[tutorial].anchor_bootstrap_tide` adds first-anchor context to that tutorial.
Removing the table silences all tutorial text.

`Sirno.toml` does not register intrinsic or structural metadata fields.
Sirno discovers those fields from lake entries that define `meta.type`.
An intrinsic or structural entry address is the metadata field name.
Structural relation order is entry-address order.

Config comments, setup commands, utility commands, control files,
generated navigation, and Tide review state have their own entry homes.
`project-config-comments` owns the exact generated comments.
`project-setup-commands`, `utility-commands`, and `lake-commands` own command behavior.
`control-files` owns shared `.sirno/` control-file placement.
`mist` owns projection render settings.
Structural relation entries own Tide policy.
