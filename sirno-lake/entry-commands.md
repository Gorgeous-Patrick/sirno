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

`sirno new` creates one Markdown entry from typed command-line metadata.
`sirno entry new` is its grouped form.
The `-d`, `-n`, and `-b` flags are short forms for `--desc`, `--name`, and `--body`.
The `--structural FIELD=ENTRY_ADDRESS` option adds configured structural link targets.
It refuses to overwrite an existing entry file.

`sirno entry rename OLD_ENTRY_ADDRESS NEW_ENTRY_ADDRESS` renames one entry address.
It updates structural link targets, existing generated footer regions,
and configured witness sentinels that reference `OLD_ENTRY_ADDRESS`.
`sirno entry mv` and `sirno entry move` are aliases for `sirno entry rename`.
`sirno move entry OLD_ENTRY_ADDRESS NEW_ENTRY_ADDRESS`
and `sirno mv entry OLD_ENTRY_ADDRESS NEW_ENTRY_ADDRESS`
select the same entry rename.
Authored prose outside generated footer regions remains user-owned.

`sirno entry path ENTRY_ADDRESS` prints filesystem paths related to one entry.
Its default output includes the lake Markdown entry file path,
the lake `.artifacts/<entry-address>/` tree,
and the private frost entry root when frost is configured.
It excludes repository witness paths.
The `--entry`, `--artifact`, and `--frost` flags select one or more path classes.
The `--absolute` flag prints absolute paths.
The `-o, --format` option selects `human`, `json`, or `paths`.

`sirno artifact list ENTRY_ADDRESS` lists owner-relative artifact paths for one entry.
`sirno artifact add ENTRY_ADDRESS SOURCE [ARTIFACT_PATH]`
copies a file into `.artifacts/<entry-address>/...`.
When `ARTIFACT_PATH` is omitted,
the source file name becomes the owner-relative artifact path.
`sirno artifact rename ENTRY_ADDRESS OLD_PATH NEW_PATH` renames one artifact path.
`sirno artifact mv` and `sirno artifact move` are aliases for `rename`.
`sirno artifact remove ENTRY_ADDRESS ARTIFACT_PATH` removes one artifact.
`sirno artifact rm` and `sirno artifact delete` are aliases for `remove`.
The grouped forms live under `sirno entry artifact`.
Artifact mutation commands refuse to change artifacts owned by a frozen entry.

`sirno freeze ENTRY_ADDRESS` verifies that one lake entry matches current frost,
adds the `reviewed` frozen reason,
and applies local file protection to that file and its artifact tree.
`sirno melt ENTRY_ADDRESS` removes the `reviewed` frozen reason.
It clears local file protection only when no other frozen reason remains.
`sirno unfreeze ENTRY_ADDRESS` is an alias for `sirno melt ENTRY_ADDRESS`.
The grouped forms are `sirno entry freeze`, `sirno entry melt`, and `sirno entry unfreeze`.
They accept the same direct-entry and TUI spellings.
`sirno freeze`, `sirno melt`, `sirno freeze tui`, and `sirno melt tui`
open one terminal UI for selected-entry freeze and melt work.
`Space` applies the command's default operation.
`f` freezes, `m` melts, `c` refreshes, and `Tab` switches the default operation.
`sirno melt --unsafe-all` clears all Sirno local protection in the active lake
without editing metadata or deleting files.
It prints a danger warning and the selected paths.
`sirno freeze --fix-all` reapplies local protection from `meta.frozen` reasons
and immutable frost checkout state.
`--dry-run` reports selected paths for either all-project operation.

`sirno query` reads the configured Markdown lake.
`sirno entry query` is its grouped form.
Its default mode is vague text query.
Structural filters use `--has FIELD=ENTRY_ADDRESS[,ENTRY_ADDRESS]`.
Field state filters use `--is FIELD=present`, `--is FIELD=empty`, or `--is FIELD=missing`.
Distinct fields narrow results.
Same-field target filters and state filters are alternatives.
The `--columns` option selects built-in output columns and configured link relations.
When omitted,
query selects the default `id` and `name` columns.
When present with no value,
query prints selectable column names without selecting entries.
The `-o, --format` option selects the output format.

`sirno rg` runs `rg` against the active lake path.
`sirno entry rg` is its grouped form.
It forwards its arguments to the `rg` binary,
then appends the resolved lake path.
It preserves `rg` exit codes.
By default,
it asks `rg` to search Markdown entries through a preprocessor
that masks Sirno-owned generated footer regions.
The mask preserves paths, line breaks, and byte offsets outside those regions.
With `--with-generated-footer`,
it searches the full Markdown files including generated links.

`sirno witness ENTRY_ADDRESS` scans configured repo members through `mosaika`
and reports repository witness blocks for the selected entry address.
`sirno entry witness ENTRY_ADDRESS` is its grouped form.
It first resolves `ENTRY_ADDRESS` in the active lake.
Missing entries fail before repo members are scanned.
`sirno witness ENTRY_ADDRESS -f, --full` also prints the full matched repository regions.
The witness output reports the opening and closing delimiter ranges.
Delimiter ranges start at the sentinel text and exclude leading indentation.
In full mode, the summary line contains only the range.
The displayed region is the complete set of lines spanned by the witness block.
Sirno preserves the matched indentation.
A blank line separates the summary from that region.
Multiple full regions are separated by a blank line, `---`, and another blank line.

---

> **Sirno generated links begin. Do not edit this section.**

- belongs (to):
  - [interfaces](interfaces.md)
- belongs (from): (none)

> **Sirno generated links end.**
