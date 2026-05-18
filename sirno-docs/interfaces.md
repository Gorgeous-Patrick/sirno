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
move configured storage paths, maintain *generated footer* links,
manage optional Frost snapshots, and manage the active *tide*.
The global `-C, --config PATH` option selects the Sirno project config file.
The global `-L, --lake-path PATH` option overrides the configured public *lake*
for commands that read or write the active *lake*.
The global `-F, --frost-path PATH` option selects a Frost path for direct Frost checks.
Common command aliases keep terminal use compact:
`q` for `query`, `st` for `status`, `w` or `wit` for `witness`,
and `defrost` for latest Frost checkout.
Entry-centric operations also live under `sirno entry`.
Storage-wide lake operations also live under `sirno lake`.
Entry artifact operations also have the top-level `sirno artifact` form.
Frost operations also live under `sirno frost`.
When a top-level command delegates to a group,
the grouped spelling uses the same subcommands and aliases.
For example, `sirno query`, `sirno q`, `sirno entry query`, and `sirno entry q`
select the same entry operation.
Likewise, `sirno status`, `sirno st`, `sirno lake status`, and `sirno lake st`
select the same lake operation.
`sirno init` initializes the public *lake* and private *frost* store together.
`sirno commit` and `sirno checkout` select the same Frost operations
as their grouped `sirno frost ...` forms.
`sirno defrost` selects latest Frost checkout.
Public lake setup and path moves also live under `sirno lake`.
Top-level `sirno move` groups the three mutation moves:
`sirno move entry OLD_ID NEW_ID`, `sirno move lake PATH`, and `sirno move frost PATH`.
`sirno mv ...` is its short form.
Each wrapper delegates to the corresponding grouped move command.
For artifact mutation,
`sirno artifact ...` and `sirno entry artifact ...` select the same entry operation.
Those commands should remain plain enough to use from a terminal
and stable enough for agents and skills to call.

`sirno status` summarizes the configured *repository*.
It reports the config path, *monograph* state, *lake* path, optional *frost* path,
*frost* lock state, *entry* count, check policy, structural policy, and current check result.

`sirno init` creates a Sirno config, ordinary seed entries,
and an empty Frost version `0`.
By default, it names those paths from the directory that contains `Sirno.toml`:
`<repo>-lake` for the public *lake* and `<repo>-frost` for the private *frost* path.
`sirno init --lake PATH` chooses a non-default public *lake* path.
`sirno init --frost PATH` chooses a non-default private *frost* path.
`sirno lake init [PATH]` creates a Sirno config and ordinary seed entries without configuring Frost.
`sirno lake move PATH` changes the configured public *lake* path
and renames the current *lake* directory on the filesystem.
`sirno lake mv PATH` is its short form.
`sirno move lake PATH` and `sirno mv lake PATH` select the same path move.

`sirno frost init [PATH]` configures the private *frost* path
and records empty version `0`.
`sirno frost move PATH` changes the configured *frost* path
and renames the current *frost* path on the filesystem.
`sirno frost mv PATH` is its short form.
`sirno move frost PATH` and `sirno mv frost PATH` select the same path move.
`sirno commit` freezes the current public *lake*
and writes the resulting current snapshot reference to `Sirno.lock.toml`.
`sirno frost commit` is its grouped form.
It fails while open *tide* workitems remain.
When `[tutorial]` is present,
this failure can include tutorial text controlled by `[tutorial].frost_commit_tide`
and `[tutorial].frost_bootstrap_tide`.
`sirno commit --unsafe-resolve-all` bypasses that gate for the current commit.
`sirno checkout --latest` materializes the latest version as a mutable public *lake*.
`sirno defrost` is shorthand for `sirno checkout --latest`.
`sirno checkout VERSION` materializes one older version into the public *lake*.
The grouped checkout command is `sirno frost checkout`.
The grouped latest shortcut is `sirno frost defrost`.
Version checkout is immutable unless `--unsafe-mutable` is supplied.

`sirno new` creates one Markdown *entry* from typed command-line metadata.
`sirno entry new` is its grouped form.
The `-d`, `-n`, and `-b` flags are short forms for `--desc`, `--name`, and `--body`.
The `--structural FIELD=ENTRY_ID` option adds configured structural metadata targets.
It refuses to overwrite an existing *entry* file.
`sirno entry rename OLD_ID NEW_ID` renames one *entry* id.
It updates structural metadata targets,
existing *generated footer* regions,
and configured *witness* sentinels that reference `OLD_ID`.
`sirno entry mv` and `sirno entry move` are aliases for `sirno entry rename`.
`sirno move entry OLD_ID NEW_ID` and `sirno mv entry OLD_ID NEW_ID`
select the same entry rename.
Authored prose outside *generated footer* regions remains user-owned.

`sirno path ENTRY_ID` prints filesystem paths related to one *entry*.
Its default output includes the public Markdown *entry* path,
the public `.artifacts/<entry-id>/` tree,
and the private Frost entry root when Frost is configured.
It excludes *repository witness* paths.
The grouped form is `sirno entry path ENTRY_ID`.
The `--entry`, `--artifact`, and `--frost` flags select one or more path classes.
The `--absolute` flag prints absolute paths.
The `-o, --format` option selects `human`, `json`, or `paths`.

`sirno artifact list ENTRY_ID` lists owner-relative artifact paths for one *entry*.
`sirno artifact add ENTRY_ID SOURCE [ARTIFACT_PATH]`
copies a file into `.artifacts/<entry-id>/...`.
When `ARTIFACT_PATH` is omitted,
the source file name becomes the owner-relative artifact path.
`sirno artifact rename ENTRY_ID OLD_PATH NEW_PATH` renames one artifact path.
`sirno artifact mv` and `sirno artifact move` are aliases for `rename`.
`sirno artifact remove ENTRY_ID ARTIFACT_PATH` removes one artifact.
`sirno artifact rm` and `sirno artifact delete` are aliases for `remove`.
The grouped forms live under `sirno entry artifact`.
Artifact mutation commands refuse to change artifacts owned by a frozen *entry*.

`sirno freeze ENTRY_ID` verifies that one public *entry* matches current Frost,
adds `frozen:`,
and removes write permission from that file and its artifact tree.
`sirno melt ENTRY_ID` removes `frozen:` from one public *entry*
and restores write permission to its file and artifact tree.
`sirno unfreeze ENTRY_ID` is an alias for `sirno melt ENTRY_ID`.
The grouped forms are `sirno entry freeze`, `sirno entry melt`, and `sirno entry unfreeze`.

`sirno query` reads the configured Markdown *lake*.
`sirno entry query` is its grouped form.
Its default mode is vague text query.
Exact structural predicates use `-x, --exact FIELD=ENTRY_ID`.
The `-f, --fields` option selects output fields.
The `-o, --format` option selects the output format.

`sirno check` checks the active *lake*.
The `-m, --mode` option selects the check boundary.

`sirno rg` runs `rg` against the active *lake* path.
`sirno entry rg` is its grouped form.
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
and reports *repository witness* blocks for the selected *entry* id.
`sirno entry witness ENTRY_ID` is its grouped form.
It first resolves `ENTRY_ID` in the active *lake*.
Missing *entries* fail before repo members are scanned.
`sirno witness ENTRY_ID -f, --full` also prints the full matched *repository* regions.
The *witness* output reports the opening and closing delimiter ranges.
Delimiter ranges start at the sentinel text and exclude leading indentation.
In full mode, the summary line contains only the range.
The displayed region is the complete set of lines spanned by the *witness* block.
Sirno preserves the matched indentation.
A blank line separates the summary from that region.
Multiple full regions are separated by a blank line, `---`, and another blank line.

`sirno tide status` reports open dependency review obligations.
`sirno tide status --all` also reports resolved obligations.
`sirno tide status -o, --format` selects `human` or `json` output.
The canonical review command forms are `sirno tide resolve` and `sirno tide unresolve`.
The top-level forms `sirno resolve` and `sirno unresolve` select the same operations.
`sirno resolve ENTRY_ID` resolves open workitems whose neighbor is that *entry*.
`sirno resolve RIPPLE,FIELD,DIRECTION,NEIGHBOR` resolves one full workitem tuple.
`sirno resolve --infer` resolves open workitems whose neighbor also appears in the ripple set.
`sirno resolve --json JSON` resolves full workitem tuples encoded as JSON.
`sirno unresolve ENTRY_ID` removes resolutions whose neighbor is that *entry*.
`sirno unresolve RIPPLE,FIELD,DIRECTION,NEIGHBOR` removes one full workitem resolution.
`sirno reopen` is an alias for `sirno unresolve`.
`sirno tide reopen` is an alias for `sirno tide unresolve`.
`sirno tide reset` clears tide resolution state.

`sirno render` creates or replaces Sirno-owned *generated footer* regions.
`sirno render -n, --dry` reports *generated footer* regions that would change without writing files.
`--dry-run` is an alias for `--dry`.
`sirno render delete` removes those regions.
Render commands operate on the active *lake* path.

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
