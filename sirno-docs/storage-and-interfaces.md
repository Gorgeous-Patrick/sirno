---
name: Storage And Interfaces
description: The design commitment to eter storage and CLI or MCP access interfaces.
category:
  - concept
clustee:
  - sirno
witness:
---

Sirno uses one required storage surface and several optional surfaces.

The public Markdown store is the editable working form.
The configured monograph is optional.
Configured code members are optional and enable witness lookup.
The private history root is optional and managed through `eter`.
`eter` provides durable storage, indexing, immutable snapshots,
field history, version retirement, and garbage collection.

Sirno exposes the store through CLI and MCP interfaces.
A lightweight GUI or Obsidian extension may later provide a direct editing experience.

Repository witnesses are managed through `mosaika` when code members are configured.
The entry id is the query key Sirno uses when locating witness blocks.

The storage design separates the public Markdown form from the durable substrate.
Markdown entries are the human-facing form.
They are easy to read, review, diff, and edit.
`eter` provides the storage and indexing foundation beneath that form,
so Sirno can grow more capable without making the entry files opaque.
It also gives Sirno store-wide snapshots in `sirno-history`
without adding version fields to entry metadata.

The CLI is the first operational interface.
It can initialize stores, create entries, query entries, check structure,
move configured storage paths, and maintain generated link footers.
Those commands should remain plain enough to use from a terminal
and stable enough for agents and skills to call.

`sirno status` summarizes the configured repository.
It reports the config path, monograph state, store path, optional history path,
history lock state, entry count, check policy, link policy, and current check result.

`sirno mv PATH` changes the configured public store path
and renames the current store directory on the filesystem.

`sirno history init` configures the private history root and commits the current public store.
`sirno history mv PATH` changes the configured history path
and renames the current history root on the filesystem.
`sirno history commit` commits the current public store into history
and writes the resulting current version to `Sirno.lock`.
`sirno history checkout VERSION` materializes one version into the public store.
The checkout is immutable unless `--unsafe-mutable` is supplied.

`sirno new` creates one Markdown entry from typed command-line metadata.
It refuses to overwrite an existing entry file.

`sirno query` reads the configured Markdown store.
Its default mode is vague text query.
Exact structural predicates live behind explicit exact flags.

`sirno witness ENTRY_ID` scans configured code members through `mosaika`
and reports repository witness blocks for the selected entry id.
`sirno witness ENTRY_ID --full` also prints the full matched code regions.
Witness output reports the opening and closing delimiter ranges.
In full mode, the summary line contains only the range.
A blank line separates the summary from the dedented region.
Multiple full regions are separated by a blank line, `---`, and another blank line.

`sirno gen-link` creates or replaces Sirno-owned generated footer regions.
`sirno gen-link --dry` reports generated footer regions that would change without writing files.
`sirno gen-link delete` removes those regions.
Generated-link commands operate on the configured store unless an explicit entry directory is given.

`sirno util completion` emits shell completion scripts.
Completion generation is a utility interface,
not a store operation.

The MCP interface serves interactive tooling.
It can expose the same store model to agents and editors without asking them to shell out for every action.
Future GUI or Obsidian work should keep the same ownership rules:
metadata is structural,
generated footer regions are Sirno-owned,
and prose outside generated regions remains user-owned.

Witness lookup stays separate through `mosaika`.
That lets witness blocks evolve with code navigation needs
while Sirno keeps the entry id as the shared nominal handle.

---

> **Sirno generated links begin. Do not edit this section.**

Clustee (to)
- [sirno](sirno.md)

> **Sirno generated links end.**
