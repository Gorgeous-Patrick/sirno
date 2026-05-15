---
name: Storage And Interfaces
description: The design commitment to eter storage and CLI or MCP access interfaces.
category:
  - concept
belongs:
  - sirno
---

Sirno uses one required storage surface and several optional surfaces.

The public Markdown *lake* is the editable working form.
The configured *monograph* is optional.
Configured repo members are optional and enable *witness* lookup.
The private *frost* root is optional and managed through `eter`.
`eter` provides durable storage, indexing, immutable snapshots,
field history, version retirement, and garbage collection.

Sirno exposes the *lake* through CLI and MCP interfaces.
A lightweight GUI or Obsidian extension may later provide a direct editing experience.

Repository *witnesses* are managed through `mosaika` when repo members are configured.
The *entry* id is the query key Sirno uses when locating *witness* blocks.

The storage design separates the public Markdown form from the durable substrate.
Markdown *entries* are the human-facing form.
They are easy to read, review, diff, and edit.
`eter` provides the storage and indexing foundation beneath that form,
so Sirno can grow more capable without making the *entry* files opaque.
It also gives Sirno Lake snapshots in `sirno-frost`
without adding version fields to *entry* metadata.

The CLI is the first operational interface.
It can initialize *lakes*, create *entries*, query *entries*, check structure,
move configured storage paths, and maintain *generated footer* links.
The global `--lake-path PATH` option overrides the configured public *lake*
for commands that read or write the active *lake*.
Those commands should remain plain enough to use from a terminal
and stable enough for agents and skills to call.

`sirno status` summarizes the configured repository.
It reports the config path, *monograph* state, *lake* path, optional *frost* path,
*frost* lock state, *entry* count, check policy, link policy, and current check result.

`sirno mv PATH` changes the configured public *lake* path
and renames the current *lake* directory on the filesystem.

`sirno frost init` configures the private *frost* root and freezes the current public *lake*.
`sirno frost mv PATH` changes the configured *frost* path
and renames the current *frost* root on the filesystem.
`sirno frost commit` freezes the current public *lake*
and writes the resulting current snapshot reference to `Sirno.lock.toml`.
`sirno frost checkout VERSION` materializes one version into the public *lake*.
The checkout is immutable unless `--unsafe-mutable` is supplied.

`sirno new` creates one Markdown *entry* from typed command-line metadata.
It refuses to overwrite an existing *entry* file.

`sirno freeze ENTRY_ID` adds `frozen:` to one public *entry*
and removes write permission from that file.
`sirno melt ENTRY_ID` removes `frozen:` from one public *entry*
and restores write permission.
`sirno unfreeze ENTRY_ID` is an alias for `sirno melt ENTRY_ID`.

`sirno query` reads the configured Markdown *lake*.
Its default mode is vague text query.
Exact structural predicates live behind explicit exact flags.

`sirno witness ENTRY_ID` scans configured repo members through `mosaika`
and reports repository *witness* blocks for the selected *entry* id.
`sirno witness ENTRY_ID --full` also prints the full matched repository regions.
The *witness* output reports the opening and closing delimiter ranges.
Delimiter ranges start at the sentinel text and exclude leading indentation.
In full mode, the summary line contains only the range.
The displayed region is the complete set of lines spanned by the *witness* block.
Sirno preserves the matched indentation.
A blank line separates the summary from that region.
Multiple full regions are separated by a blank line, `---`, and another blank line.

`sirno gen-link` creates or replaces Sirno-owned *generated footer* regions.
`sirno gen-link --dry` reports *generated footer* regions that would change without writing files.
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

The *witness* lookup stays separate through `mosaika`.
That lets *witness* blocks evolve with repository navigation needs
while Sirno keeps the *entry* id as the shared nominal handle.

---

> **Sirno generated links begin. Do not edit this section.**

Belongs (from): (none)

Belongs (to):
- [sirno](sirno.md)

> **Sirno generated links end.**
