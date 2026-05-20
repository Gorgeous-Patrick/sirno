---
name: Interfaces
desc: The CLI and MCP surfaces that operate on Sirno project storage.
category:
  - concept
belongs:
  - sirno
prerequisite:
  - project-config
---

Sirno exposes configured project storage through CLI and MCP interfaces.
A lightweight GUI or Obsidian extension may later provide a direct editing experience.

The CLI is the human-facing operational interface.
Human interaction with Sirno should happen through the CLI.
It can initialize *lakes*, create *entries*, query *entries*, check structure,
move configured storage paths, maintain *generated footer* links,
manage optional Frost snapshots, and manage the active *tide*.
The `sirno::surface` module is the shared command surface behind the CLI.
The binary `main.rs` delegates process startup to that module.
`sirno --version` prints the package version from `Cargo.toml` before command dispatch.
Reusable helpers in `sirno::surface` return typed query, path, tide, and witness data
before the CLI renders human text or JSON.
Human CLI output prints records, tables, or diagnostics before command summary lines.
Terminal human output may color semantic labels such as setup choices,
diagnostic severity, check state, tide state, and wrapper status.
JSON output remains structured data and carries no terminal styling.
Commands with no detail may print only their summary.
MCP tools should call those typed helpers and prefer JSON rendering through the shared serializer.
Human-facing usage and mechanism documents should spell Sirno operations as CLI commands.
Agent-facing discipline entries, packaged skill resources, and MCP documentation
should spell Sirno operations as MCP tools when the agent performs them.
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
`sirno init` prompts for public *lake*, private *frost* store,
and packaged skill wrapper setup.
`sirno init --all` initializes those parts together without prompts.
`sirno init --claude-skills` also links installed skill packages into `.claude/skills`.
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
and stable enough to share behavior with MCP tools.

`sirno status` summarizes the configured *repository*.
It reports the config path, *lake* path, optional *frost* path,
*frost* lock state, *entry* count, check policy, structural policy, and current check result.

`sirno init` opens an interactive setup flow.
It asks which setup parts to run, asks for default paths when no path flag supplies them,
asks whether installed wrappers should be linked into Claude skills,
shows the init plan, and applies it after confirmation.
`sirno init --all` creates a Sirno config, ordinary seed entries,
an empty Frost version `0`, and packaged skill wrappers without prompts.
By default, it names those paths from the directory that contains `Sirno.toml`:
`<repo>-lake` for the public *lake* and `<repo>-frost` for the private *frost* path.
`sirno init --lake PATH` chooses a non-default public *lake* path.
`sirno init --frost PATH` chooses a non-default private *frost* path.
`sirno init --no-lake`, `--no-frost`, and `--no-skills`
skip their corresponding setup parts.
`sirno init --claude-skills` creates `.claude/skills/sirno-*` links
to the installed `.agents/skills/sirno-*` package directories.
The config is still written when another selected setup part needs it.
When a setup part is skipped, its path option is not accepted.
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
Path moves create missing destination parents and refuse existing destinations.
A destination inside the moved path is handled through temporary staging.
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
and applies local file protection to that file and its artifact tree.
`sirno melt ENTRY_ID` removes `frozen:` from one public *entry*
and clears local file protection from its file and artifact tree.
`sirno unfreeze ENTRY_ID` is an alias for `sirno melt ENTRY_ID`.
The grouped forms are `sirno entry freeze`, `sirno entry melt`, and `sirno entry unfreeze`.

`sirno query` reads the configured Markdown *lake*.
`sirno entry query` is its grouped form.
Its default mode is vague text query.
Structural filters use `--has FIELD=ENTRY_ID[,ENTRY_ID]`.
Field state filters use `--is FIELD=present`, `--is FIELD=empty`, or `--is FIELD=missing`.
Distinct fields narrow results.
Same-field target filters and state filters are alternatives.
The `--columns` option selects output columns.
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
The MCP `sirno_entry_witness` tool returns full matched *repository* regions by default.
Each record includes the full `region` span and matched `body`.
When `verbose` is true,
records also include `opening` and `closing` delimiter spans.

`sirno tide status` reports entry ids that need dependency review,
grouped by review entry in one table.
The reason column lists the ripple entry whose change created the review obligation.
It prints a one-sentence summary after the table.
`sirno tide status --by wave` groups the same output by wave.
`sirno tide status --show full` reports open dependency review obligations
in the same grouped table.
`sirno tide status --show all` also reports resolved obligations.
`sirno tide status --by entry` selects the default review-entry grouping explicitly.
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
Render commands print changed paths or blocking diagnostics before their summary line.
`--dry-run` is an alias for `--dry`.
`sirno render --override-json JSON` uses JSON structural render settings for that run,
instead of the configured settings.
The JSON uses structural field and edge names,
such as `{"belongs":{"to":{"render":true}}}`.
It does not write `Sirno.toml`.
`sirno render delete` removes those regions.
Render commands operate on the active *lake* path.

`sirno util completion` emits shell completion scripts.
Completion generation is a utility interface,
not a *lake* operation.

`sirno util config` opens an interactive terminal UI for `Sirno.toml`.
Each row is a top-level config section with its presence status
and canonical-comment status.
`j`, `k`, Up, and Down move the selected row.
`i` inserts the selected section with canonical comments.
`f` repairs comments for the selected non-empty section.
`q` and Esc exit.
`sirno util config --dry` keeps the old non-interactive comment check.
It prints missing comments before the summary line,
does not write `Sirno.toml`,
and exits with failure when comments are missing.
`sirno util config --fix` keeps the old non-interactive comment repair.
It rewrites `Sirno.toml` through the canonical config renderer when comments are missing.
`--dry` and `--fix` are mutually exclusive.
Config utility commands reject `--lake-path` and `--frost-path`.

`sirno util skills init` installs bundled Sirno skill wrappers
to their `.agents/skills/sirno-*` package targets.
`sirno util skills check` reports whether installed wrappers match those bundled constants.
`sirno util skills list` lists the bundled skill names and package targets.
The `--claude-skills` option includes `.claude/skills/sirno-*` links in `init`, `check`,
and `list` output.
Skill utility commands print wrapper records as a table,
followed by a summary line.
Skill utility commands reject `--lake-path` and `--frost-path`.

`sirno util mcp --config PATH` starts the MCP server over stdio.
When `--config` is omitted, the server uses the default `Sirno.toml` path.
Project tools resolve that config path on each tool call.
If the config path is relative, the server process current working directory controls the project.
`sirno util mcp` rejects `--lake-path` and `--frost-path`;
the configured project selects its *lake* and optional *frost* path.

The MCP interface exposes grouped project command tools and skill resources.
It does not expose top-level CLI aliases, shortcut commands, prompts, or CLI utility commands.

Skill resources are `text/markdown` payloads embedded by `src/mcp.rs`
from the lake-owned `SKILL.full.md` artifacts.
Packaged `.agents/skills/sirno-*` wrappers tell agents to read these resources.

The `sirno util` command family is the local operator and integration-maintenance surface.
It prepares or repairs the environment around a Sirno project.
Humans perform that operator work through the CLI.

MCP is the agent-facing project interface:

- it serves stable project operations
- it serves lake-owned skill instructions as resources
- it keeps host setup and package maintenance as explicit human CLI actions

MCP resources are:

- `sirno://skills/design-doc-writer`
- `sirno://skills/sirno-config-writer`
- `sirno://skills/sirno-editor`
- `sirno://skills/sirno-explorer`
- `sirno://skills/sirno-narrative-session`
- `sirno://skills/sirno-skill-synthesizer`
- `sirno://skills/sirno-witness`
- `sirno://entries/{id}` through the entry resource template

Reading one entry resource returns the full stored Markdown source as `text/markdown`.

MCP tool names are stable snake-case names prefixed with `sirno_`:

- project binding: `sirno_cwd`
- entries: `sirno_entry_new`, `sirno_entry_rename`, `sirno_entry_freeze`,
  `sirno_entry_melt`, `sirno_entry_path`, `sirno_entry_read`, `sirno_entry_query`,
  `sirno_entry_rg`, and `sirno_entry_witness`
- entry artifacts: `sirno_entry_artifact_list`, `sirno_entry_artifact_add`,
  `sirno_entry_artifact_rename`, and `sirno_entry_artifact_remove`
- lake: `sirno_lake_init`, `sirno_lake_move`, `sirno_lake_check`,
  `sirno_lake_render`, `sirno_lake_render_delete`, and `sirno_lake_status`
- frost: `sirno_frost_init`, `sirno_frost_move`, `sirno_frost_commit`,
  `sirno_frost_checkout`, and `sirno_frost_defrost`
- tide: `sirno_tide_status`, `sirno_tide_resolve`, `sirno_tide_unresolve`,
  and `sirno_tide_reset`

MCP tools accept typed JSON arguments.

- `sirno_cwd` accepts optional `{ path }`.
  With `path`, it changes the process current working directory before returning it.
  Without `path`, it returns the current working directory without changing it.
- Relative config paths are resolved against the process current working directory
  on every project tool call.
- `sirno_entry_read` returns parsed metadata, body text, and the full stored Markdown source.
- Structural filters may use `{ field, targets }` objects
  or compact `FIELD=ENTRY_ID[,ENTRY_ID]` strings.
- Structural states may use `{ field, state }` objects
  or compact `FIELD=present`, `FIELD=empty`, and `FIELD=missing` strings.
- Tide selectors use neighbor id arrays and existing JSON-shaped workitem objects.
- `sirno_tide_status` returns review entry ids by default.
  Its `show` argument selects `review`, `full`, or `all`.
- `sirno_entry_rg` accepts `args: string[]` and returns captured `exit_code`, `stdout`, and `stderr`.
- Successful tool calls return structured JSON content.
  They also include the same JSON as pretty text content for clients that read only text.
- Domain failures such as failed checks, dirty query preconditions,
  and nonzero `rg` exits return structured results with `ok: false`.
- Command failures return MCP tool errors with concise text.

The MCP adapter calls `sirno::surface` methods for command behavior.
Public request and result DTOs live in `sirno::surface`.
The adapter only converts JSON parameters into surface requests
and surface DTOs into MCP tool results.
This keeps CLI JSON and MCP JSON aligned without duplicating command logic.

The MCP interface serves interactive tooling.
It can expose the same *lake* model to agents and editors
without asking them to shell out for every action.
Future GUI or Obsidian work should keep the same ownership rules:
metadata is structural,
*generated footer* regions are Sirno-owned,
and prose outside generated regions remains user-owned.

MCP does not expose `sirno util` commands.
This keeps the MCP capability set small enough for hosts and agents to audit.

---

> **Sirno generated links begin. Do not edit this section.**

- belongs (to):
  - [sirno](sirno.md)
- belongs (from): (none)

> **Sirno generated links end.**
