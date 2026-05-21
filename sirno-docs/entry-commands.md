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
The `--structural FIELD=ENTRY_ID` option adds configured structural metadata targets.
It refuses to overwrite an existing entry file.

`sirno entry rename OLD_ID NEW_ID` renames one entry id.
It updates structural metadata targets, existing generated footer regions,
and configured witness sentinels that reference `OLD_ID`.
`sirno entry mv` and `sirno entry move` are aliases for `sirno entry rename`.
`sirno move entry OLD_ID NEW_ID` and `sirno mv entry OLD_ID NEW_ID`
select the same entry rename.
Authored prose outside generated footer regions remains user-owned.

`sirno path ENTRY_ID` prints filesystem paths related to one entry.
Its default output includes the lake Markdown entry path,
the lake `.artifacts/<entry-id>/` tree,
and the private frost entry root when frost is configured.
It excludes repository witness paths.
The grouped form is `sirno entry path ENTRY_ID`.
The `--entry`, `--artifact`, and `--frost` flags select one or more path classes.
The `--absolute` flag prints absolute paths.
The `-o, --format` option selects `human`, `json`, or `paths`.

`sirno artifact list ENTRY_ID` lists owner-relative artifact paths for one entry.
`sirno artifact add ENTRY_ID SOURCE [ARTIFACT_PATH]`
copies a file into `.artifacts/<entry-id>/...`.
When `ARTIFACT_PATH` is omitted,
the source file name becomes the owner-relative artifact path.
`sirno artifact rename ENTRY_ID OLD_PATH NEW_PATH` renames one artifact path.
`sirno artifact mv` and `sirno artifact move` are aliases for `rename`.
`sirno artifact remove ENTRY_ID ARTIFACT_PATH` removes one artifact.
`sirno artifact rm` and `sirno artifact delete` are aliases for `remove`.
The grouped forms live under `sirno entry artifact`.
Artifact mutation commands refuse to change artifacts owned by a frozen entry.

`sirno freeze ENTRY_ID` verifies that one lake entry matches current frost,
adds `frozen:`,
and applies local file protection to that file and its artifact tree.
`sirno melt ENTRY_ID` removes `frozen:` from one lake entry
and clears local file protection from its file and artifact tree.
`sirno unfreeze ENTRY_ID` is an alias for `sirno melt ENTRY_ID`.
The grouped forms are `sirno entry freeze`, `sirno entry melt`, and `sirno entry unfreeze`.
They accept the same direct-entry and TUI spellings.
`sirno freeze`, `sirno melt`, `sirno freeze tui`, and `sirno melt tui`
open one terminal UI for selected-entry freeze and melt work.
`Space` applies the command's default operation.
`f` freezes, `m` melts, `c` refreshes, and `Tab` switches the default operation.
`sirno melt --unsafe-all` clears all Sirno local protection in the active lake
without editing metadata or deleting files.
It prints a danger warning and the selected paths.
`sirno freeze --fix-all` reapplies local protection from `frozen:` markers
and immutable frost checkout state.
`--dry-run` reports selected paths for either all-project operation.

`sirno query` reads the configured Markdown lake.
`sirno entry query` is its grouped form.
Its default mode is vague text query.
Structural filters use `--has FIELD=ENTRY_ID[,ENTRY_ID]`.
Field state filters use `--is FIELD=present`, `--is FIELD=empty`, or `--is FIELD=missing`.
Distinct fields narrow results.
Same-field target filters and state filters are alternatives.
The `--columns` option selects output columns.
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

`sirno witness ENTRY_ID` scans configured repo members through `mosaika`
and reports repository witness blocks for the selected entry id.
`sirno entry witness ENTRY_ID` is its grouped form.
It first resolves `ENTRY_ID` in the active lake.
Missing entries fail before repo members are scanned.
`sirno witness ENTRY_ID -f, --full` also prints the full matched repository regions.
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
