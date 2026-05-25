---
name: Entry Commands
desc: CLI commands for entries, artifacts, queries, ripgrep, and witnesses.
category:
  - concept
belongs:
  - interfaces
prerequisite:
  - project-config
---

Entry commands operate on Markdown entries and their owner-relative artifact trees.

## Entry Files

| Command | Behavior |
|---|---|
| `sirno new` | Creates one Markdown entry from typed command-line metadata. |
| `sirno entry new` | Grouped form of `sirno new`. |
| `sirno entry rename OLD NEW` | Renames one entry address. |
| `sirno entry mv OLD NEW` | Alias for `sirno entry rename`. |
| `sirno entry move OLD NEW` | Alias for `sirno entry rename`. |
| `sirno move entry OLD NEW` | Top-level wrapper for the same rename. |
| `sirno mv entry OLD NEW` | Short top-level wrapper. |

Entry creation options:

- `-d`, `-n`, and `-b` are short forms for `--desc`, `--name`, and `--body`.
- `--structural FIELD=ENTRY_ADDRESS` adds configured structural link targets.
- Creation refuses to overwrite an existing entry file.

Entry rename updates structural link targets,
existing generated footer regions,
and configured witness sentinels that reference the old address.
Authored prose outside generated footer regions remains user-owned.

## Paths And Artifacts

`sirno entry path ENTRY_ADDRESS` prints filesystem paths related to one entry.
Its default output includes:

- the lake Markdown entry file path;
- the lake `.artifacts/<entry-address>/` tree;
- the private frost entry root when frost is configured.

It excludes repository witness paths.
The `--entry`, `--artifact`, and `--frost` flags select one or more path classes.
The `--absolute` flag prints absolute paths.
The `-o, --format` option selects `human`, `json`, or `paths`.

| Command | Behavior |
|---|---|
| `sirno artifact list ENTRY_ADDRESS` | Lists owner-relative artifact paths for one entry. |
| `sirno artifact add ENTRY_ADDRESS SOURCE [PATH]` | Copies a file into the entry artifact tree. |
| `sirno artifact rename ENTRY_ADDRESS OLD NEW` | Renames one artifact path. |
| `sirno artifact mv ENTRY_ADDRESS OLD NEW` | Alias for `rename`. |
| `sirno artifact move ENTRY_ADDRESS OLD NEW` | Alias for `rename`. |
| `sirno artifact remove ENTRY_ADDRESS PATH` | Removes one artifact. |
| `sirno artifact rm ENTRY_ADDRESS PATH` | Alias for `remove`. |
| `sirno artifact delete ENTRY_ADDRESS PATH` | Alias for `remove`. |

When `PATH` is omitted from `artifact add`,
the source file name becomes the owner-relative artifact path.
The grouped forms live under `sirno entry artifact`.
Artifact mutation commands refuse to change artifacts owned by a frozen entry.

## Freeze And Melt

| Command | Behavior |
|---|---|
| `sirno freeze ENTRY_ADDRESS` | Verifies current frost, adds `reviewed`, and protects files. |
| `sirno melt ENTRY_ADDRESS` | Removes the `reviewed` frozen reason. |
| `sirno unfreeze ENTRY_ADDRESS` | Alias for `sirno melt ENTRY_ADDRESS`. |
| `sirno entry freeze` | Grouped form. |
| `sirno entry melt` | Grouped form. |
| `sirno entry unfreeze` | Grouped form. |
| `sirno freeze tui` | Opens selected-entry freeze and melt work. |
| `sirno melt tui` | Opens selected-entry freeze and melt work. |

`sirno melt` clears local file protection only when no other frozen reason remains.
The grouped forms accept the same direct-entry and TUI spellings.
Plain `sirno freeze` and `sirno melt` also open the selected-entry terminal UI.

The freeze and melt TUI keys are:

| Key | Action |
|---|---|
| `Space` | Applies the command's default operation. |
| `f` | Freezes. |
| `m` | Melts. |
| `c` | Refreshes. |
| `Tab` | Switches the default operation. |

All-project protection commands are explicit:

- `sirno melt --unsafe-all` clears all Sirno local protection in the active lake.
- `sirno freeze --fix-all` reapplies local protection from `meta.frozen` reasons and frost state.
- `--dry-run` reports selected paths for either all-project operation.

`sirno melt --unsafe-all` does not edit metadata or delete files.
It prints a danger warning and the selected paths.

## Query And Search

`sirno query` reads the configured Markdown lake.
`sirno entry query` is its grouped form.
Its default mode is vague text query.

Query filters and output follow these rules:

- Structural filters use `--has FIELD=ENTRY_ADDRESS[,ENTRY_ADDRESS]`.
- Field state filters use `--is FIELD=present`, `--is FIELD=empty`, or `--is FIELD=missing`.
- Distinct fields narrow results.
- Same-field target filters and state filters are alternatives.
- `--columns` selects built-in output columns and configured link relations.
- Without `--columns`, query selects the default `id` and `name` columns.
- `--columns` with no value prints selectable column names without selecting entries.
- `-o, --format` selects the output format.

`sirno rg` runs `rg` against the active lake path.
`sirno entry rg` is its grouped form.
It forwards its arguments to the `rg` binary,
then appends the resolved lake path.
It preserves `rg` exit codes.

By default,
`sirno rg` asks `rg` to search Markdown entries through a preprocessor
that masks Sirno-owned generated footer regions.
The mask preserves paths, line breaks, and byte offsets outside those regions.
With `--with-generated-footer`,
it searches the full Markdown files including generated links.

## Witnesses

`sirno witness ENTRY_ADDRESS` scans configured repo members through `mosaika`
and reports repository witness blocks for the selected entry address.
`sirno entry witness ENTRY_ADDRESS` is its grouped form.
It first resolves `ENTRY_ADDRESS` in the active lake.
Missing entries fail before repo members are scanned.

Full witness output follows these rules:

- `sirno witness ENTRY_ADDRESS -f, --full` prints the full matched repository regions.
- The witness output reports the opening and closing delimiter ranges.
- Delimiter ranges start at the sentinel text and exclude leading indentation.
- In full mode, the summary line contains only the range.
- The displayed region is the complete set of lines spanned by the witness block.
- Sirno preserves the matched indentation.
- A blank line separates the summary from that region.
- Multiple full regions are separated by a blank line, `---`, and another blank line.

---

> **Sirno generated links begin. Do not edit this section.**

- belongs (to):
  - [interfaces](interfaces.md)
- belongs (from): (none)

> **Sirno generated links end.**
