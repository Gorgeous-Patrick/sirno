---
name: Interfaces
desc: The CLI and MCP surfaces that operate on Sirno project storage.
category:
  - concept
belongs:
  - sirno
---

Sirno exposes configured project storage through CLI and MCP interfaces.
A lightweight GUI or Obsidian extension may later provide a direct editing experience.

The CLI is the first operational interface.
It can initialize *lakes*, create *entries*, query *entries*, check structure,
move configured storage paths, and maintain *generated footer* links.
The global `-C, --config PATH` option selects the Sirno project config file.
The global `-L, --lake-path PATH` option overrides the configured public *lake*
for commands that read or write the active *lake*.
Common command aliases keep terminal use compact:
`q` for `query`, `st` for `status`, and `w` or `wit` for `witness`.
Those commands should remain plain enough to use from a terminal
and stable enough for agents and skills to call.

`sirno status` summarizes the configured repository.
It reports the config path, *monograph* state, *lake* path, optional *frost* path,
*frost* lock state, *entry* count, check policy, structural policy, and current check result.

`sirno move PATH` changes the configured public *lake* path
and renames the current *lake* directory on the filesystem.
`sirno mv PATH` is its short form.

`sirno frost init` configures the private *frost* path and records empty version `0`.
`sirno frost init --frost-path PATH` chooses a non-default *frost* path.
`sirno frost move PATH` changes the configured *frost* path
and renames the current *frost* path on the filesystem.
`sirno frost mv PATH` is its short form.
`sirno frost commit` freezes the current public *lake*
and writes the resulting current snapshot reference to `Sirno.lock.toml`.
`sirno frost checkout --latest` materializes the latest version as a mutable public *lake*.
`sirno frost checkout VERSION` materializes one older version into the public *lake*.
Version checkout is immutable unless `--unsafe-mutable` is supplied.

`sirno new` creates one Markdown *entry* from typed command-line metadata.
The `-d`, `-n`, and `-b` flags are short forms for `--desc`, `--name`, and `--body`.
The `--structural FIELD=ENTRY_ID` option adds configured structural metadata targets.
It refuses to overwrite an existing *entry* file.

`sirno freeze ENTRY_ID` adds `frozen:` to one public *entry*
and removes write permission from that file.
`sirno melt ENTRY_ID` removes `frozen:` from one public *entry*
and restores write permission.
`sirno unfreeze ENTRY_ID` is an alias for `sirno melt ENTRY_ID`.

`sirno query` reads the configured Markdown *lake*.
Its default mode is vague text query.
Exact structural predicates use `-x, --exact FIELD=ENTRY_ID`.
The `-f, --fields` option selects output fields.
The `-o, --format` option selects the output format.

`sirno check` checks the active *lake*.
The `-m, --mode` option selects the check boundary.

`sirno rg` runs `rg` against the active *lake* path.
It forwards its arguments to the `rg` binary,
then appends the resolved *lake* path.
It preserves `rg` exit codes.
By default,
it asks `rg` to search Markdown *entries* through a preprocessor
that masks Sirno-owned *generated footer* regions.
The mask preserves paths, line breaks, and byte offsets outside those regions.
With `--with-generated-footer`,
it searches the full Markdown files including generated links.

`sirno witness ENTRY_ID` scans configured repo members through `mosaika`
and reports repository *witness* blocks for the selected *entry* id.
It first resolves `ENTRY_ID` in the active *lake*.
Missing *entries* fail before repo members are scanned.
`sirno witness ENTRY_ID -f, --full` also prints the full matched repository regions.
The *witness* output reports the opening and closing delimiter ranges.
Delimiter ranges start at the sentinel text and exclude leading indentation.
In full mode, the summary line contains only the range.
The displayed region is the complete set of lines spanned by the *witness* block.
Sirno preserves the matched indentation.
A blank line separates the summary from that region.
Multiple full regions are separated by a blank line, `---`, and another blank line.

`sirno gen-link` creates or replaces Sirno-owned *generated footer* regions.
`sirno gen-link -n, --dry` reports *generated footer* regions that would change without writing files.
`--dry-run` is an alias for `--dry`.
`sirno gen-link delete` removes those regions.
Generated-link commands operate on the active *lake* path.

`sirno util completion` emits shell completion scripts.
Completion generation is a utility interface,
not a *lake* operation.

The MCP interface serves interactive tooling.
It can expose the same *lake* model to agents and editors without asking them to shell out for every action.
Future GUI or Obsidian work should keep the same ownership rules:
metadata is structural,
*generated footer* regions are Sirno-owned,
and prose outside generated regions remains user-owned.

---

> **Sirno generated links begin. Do not edit this section.**

- belongs (to):
  - [sirno](sirno.md)
- belongs (from): (none)

> **Sirno generated links end.**
