# Sirno

*Semantic Intermediate Representation of Nominal Objects*

Sirno compiles between design forms for design-aware programming work,
moving among one long-form project narrative, a store of compact named Markdown entries, and the repository codebase.

Design needs a form that humans can read, tools can index,
and agents can manipulate without carrying a whole project in context.
Sirno gives design that form by naming it,
and the resulting names are readable by humans, stable for tools, and small enough to circulate.

<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="assets/sirno-20260401.png" width="40%">
    <source media="(prefers-color-scheme: light)" srcset="assets/sirno-nb-20260401.png" width="40%">
    <img src="assets/sirno-nb-20260401.png" width="40%">
  </picture>
</p>

---

## What Is Sirno?

Sirno works through three forms.

`mono` is one configured Markdown document, often `DESIGN.md`,
and it serves as the project monograph:
a readable narrative for a person who wants the whole design in one sitting.

`sirno` is one configured entry store, often `docs/`,
holding named Markdown documents with metadata blocks and bodies of prose.

`code` is the repository implementation form,
containing source files, tests, configuration, generated files, assets,
and other artifacts that realize design decisions.

The `sirno` store is the human-readable intermediate representation:
text first, structured enough for tools, and compact enough for humans and agents to inspect locally.
When history is configured, committed store state is versioned through a separate `eter` history root,
so one version names one immutable snapshot of the whole entry set.

Before the store exists, the user chooses which form currently carries more authority —
the working codebase, or the monograph that best describes the intended project —
and once the store exists, Sirno treats it as the structured intermediate form.

Sirno maintains structure, giving people, agents, skills, and tools stable nominal objects
through which design and implementation can be revised.
Judging design quality and proving that code satisfies a claim are left to the people and tools that use it.

---

## Project Configuration

A repository is Sirno-managed when it contains `Sirno.toml`.
The file configures the forms and the operational policy that Sirno applies to them.

`[mono].path` names the monograph.
`[store].path` names the public Markdown entry store.
`[history].path` optionally names the private `eter` history root,
with `sirno-history` as the default convention when history is initialized.
`[code].members` lists config-relative repository paths or globs that Sirno scans through `mosaika`.
All configured paths are resolved relative to the config file when they are not absolute.

A project can use Sirno without history.
`sirno init` creates the config and public entry store.
`sirno history init` adds the history config and commits the current public store
into the private history root.

`Sirno.lock` records the public store's history state when history is configured.
It is TOML and lives next to `Sirno.toml`.
The lock says whether the public store is the current history version
or a checked-out historical version.

`[store].ignore` is a list of store-root-relative paths that Sirno does not read.
An ignored path excludes that path and its descendants from store checks and generated-link operations.
Ignored paths do not change the entry model;
they define filesystem items that belong to adjacent tools rather than to Sirno.

`[check].link` controls generated-link freshness checks.
It is enabled by default.
Malformed generated-link sentinels remain structural errors,
because Sirno cannot safely identify its owned region when the sentinels are malformed.

`[links]` controls generated-footer projection.
The structural fields `category`, `clustee`, and `refiner` each accept either a boolean
or a table shaped as `{ to = boolean, from = boolean }`.
A boolean applies to both link sides.
`to` renders links from an entry to its metadata targets.
`from` renders links from an entry to entries that name it as a metadata target.

The default link policy renders only direct `clustee` links.
`links.clique` is a boolean policy that adds clique-derived links.
It does not change direct `clustee` projection.
When it is enabled, each named clustee closure induces a clique:
the closure entry links to members,
and each member links to the closure and the other members.

---

## Entries

The `sirno` store is a set of named Markdown documents called *entries*,
each carrying a YAML metadata block and a body of prose.

The filename stem is the entry id, globally unique within the store and case-sensitive,
and it serves as the stable nominal handle used by structural fields, generated footers, and witness lookup.
In principle, ids can follow the filesystem;
in practice, they are filename stems in lowercase ASCII kebab-case, possibly with digits,
reading like `concept-driven-development`.

An entry is sized to be read in about five minutes or less,
stating a concept, category, clique closure, refinement, invariant, interface,
implementation commitment, witnessable claim, or narrative route with local prose.
The prose body carries the design content,
while the metadata block carries structure that tools must read exactly.

---

## Concept-Driven Development

Sirno is cultivated from one elementary principle: *compression*.

Spec-driven development, intent-driven development, test-driven development —
each is effective in its own way, and yet each still leaves the same piece of the puzzle untouched.
Compression saves bandwidth, reduces communication overhead,
and is what allows understanding to scale across codebase evolution and across time.
Sirno responds with *concept-driven development*.

A *concept* is a named idea that compresses knowledge:
a design intention, an algorithmic detail, a local vocabulary, a behavior, a test rationale,
or the shared reason behind several decisions.
Cognition starts from naming things,
and it is token-efficient to keep those names stable as anchors for reference and understanding.

LZ77 uses an adaptive dictionary to replace repeated data with compact references,
and Sirno gives project knowledge a similar dictionary,
except each reference remains human-readable
and each concept entry gathers the specifications, decisions, implementation notes,
and tests that share the same name.

Concepts serve three roles at once:
they cluster behavioral specifications under one named object,
they keep intent portable across levels of detail,
and they organize tests so that properties and constraints become easier to see.

The initialized `concept` entry is ordinary, created by `init` and not privileged by the system.

---

## Narrative

Concepts alone are inert. A reader follows a route.

A *narrative* records a cognitive route through concepts.
Understanding is a process rather than a state,
and a concept unfolds over time as the reader's mental model grows and refines;
narrative shapes that unfolding.

The monograph is the primary narrative form.
It is the `mono` document — normal Markdown outside the store,
configured by path with `DESIGN.md` as the usual convention —
and it introduces terms once and trusts them afterward.

Materialized narratives may also live in the store as guides.
They state prerequisites, choose a base language,
and refer to concept entries at the end or along the way,
and these prerequisites and base-language choices belong in prose.

An *interactive narrative* presents an entry through dialogue,
asking positioning questions, observing responses,
and generating the next paragraph or quiz from the reader's current state.
The generated narrative is ephemeral;
canonical knowledge remains in entries and metadata,
while the narrative serves as a reading interface for onboarding and knowledge transfer.

The initialized `narrative` entry is ordinary, created by `init` and not privileged by the system.

---

## Structural Fields

Entries connect through their metadata,
and four named fields carry the structure that Sirno treats as operational.

- `category` classifies an entry by other entries.
- `clustee` groups an entry into a named clique.
- `refiner` points from a refinement to the broader entries it refines.
- `witness:` declares that an entry's claim is evidenced in the repository.

The first three fields refer to entries by id and are list-valued.
`witness:` is a canonical marker without a value.
Operational structure is formed only from metadata,
and although Markdown links in prose may help readers and external tools, they do not define Sirno structure.

The next four sections introduce the fields in turn.

---

## Category

A project's vocabulary should not be fixed by the tool that records it.

`category` is a metadata field of the classified entry.
It classifies that entry by other entries.
Categories are themselves entries, so entry kinds form an open, project-defined vocabulary.
Meta-classification reuses the same mechanism:
the category id `meta` classifies entries that themselves define categories,
and the `concept` and `narrative` entries may be categorized by `meta` in this way.

The `locked` field is reserved for later design.
It may eventually protect entries or regions that a project wants to treat as controlled.

---

## Clustee

Tags, scopes, namespaces, and domains all approximate the same structure —
a named clique of related entries — and that named clique is itself an entry.

`clustee` is a metadata field of the clique member.
It groups the entry by shared subject, local vocabulary, or design neighborhood.
The clique name provides a short route into a region of the store
without changing the entries' nominal identities.

The named entry used in `clustee` is the clique closure,
an ordinary entry that gives the group a name and a place for explanation.
The clique closure is a module-like review unit.
A local design or program change should often be reviewable by visiting that closure,
its members, their refiners, and their witnesses.
Splitting an entry should preserve the same `clustee` when the new entries remain in that unit.
Creating a new clique closure means creating a new review boundary,
not just a smaller tag.
The mechanism can describe a two-member clique,
whose entry records why the two members belong together.

---

## Refiner

A concept names a compressed idea. A *refinement* unfolds it.

Refinement turns a high-level idea into lower-level design, implementation, and tests;
the refined entry keeps the meaning of the concept intact
while making its consequences local and concrete.
A refinement chain is a path of increasing specificity
that starts from a compressed concept and ends near repository text,
preserving the reason that a local decision exists.
If the programming language itself is expressive and clean enough
that the design is clearest when expressed in code,
the final step of refinement may be a Markdown code block.

`refiner` is a metadata field of the refined entry.
It points to the entry it refines.
The field is list-valued,
so an entry may refine several entries when the local design realizes several broader claims.

---

## Witness

Design that never meets code drifts.
The `witness:` marker closes the distance.

The `witness:` marker is a metadata field of the witnessed entry,
declaring that the entry's claim is evidenced in the repository.
The marker is canonical and has no value.

Sirno queries witnesses through `mosaika` by entry id.
The witness may be source code, tests, configuration, generated files, assets,
or any repository artifact that `mosaika` can delimit and query,
and a test may witness an entry when the test itself is the relevant code.
Repository artifacts are selected by `[code].members`.
Directory members are scanned recursively.
The repository witness block opens with `sirno:witness:start <entry-id>`
and closes with `sirno:witness:end`.

Sirno uses the entry id itself as the witness query key,
which keeps the witness convention nominal and the witness block separate from entry prose and metadata.
The entry body may describe how to search for or interpret an artifact as fallback guidance,
while the structural convention remains the metadata marker plus the witness block.

---

## Transforms

Metadata is static. Work between forms moves.

Sirno names four transforms between its forms:

```text
mono ─────lower───▶ sirno ────realize──▶ code
mono ◀────raise──── sirno ◀───reflect─── code
```

The direct names are also useful:
`mono-to-sirno`, `sirno-to-mono`, `sirno-to-code`, and `code-to-sirno`.
These names are conceptual operations —
vocabulary for humans, LLMs, skills, CLI interfaces, and MCP tools —
and they need not all be implemented as one-shot commands.

Lowering gives broad design compact nominal form,
splitting a long narrative into entries without losing the design route that made the narrative readable,
and it creates named objects that future work can cite without retelling the whole design.

Raising composes entries into a readable monograph rather than concatenating them:
the monograph introduces terms once, trusts them afterward,
and omits detail that belongs in local entries.

Realizing uses entries to guide implementation,
and a realization step should be able to answer which entry explains a local design commitment.
Not every line of code needs its own entry, but important commitments need a nominal place.

Reflecting records durable design facts learned during implementation.
Reflect when code changes a representation, narrows an invariant, introduces a new boundary,
invalidates an old explanation, or reveals a clearer local design than the current entries record.

---

## Metadata

Structural fields rest on a small, exact schema.
Every entry has a YAML metadata block whose required fields are `name` and `description`, both plain strings.
The optional structural fields are `category`, `clustee`, and `refiner`,
always lists when present, and their values are entry ids.
The optional `witness:` marker is canonical and has no value;
no other witness spelling is accepted.

```yaml
---
name: Witness
description: An entry whose claim is evidenced by repository artifacts.
category:
  - concept
witness:
---
```

Operational structure is formed only from metadata,
and Markdown links in prose may help readers and external tools without defining Sirno structure.

---

## Query

Query selects parsed entries from the public store or from one history version.
It reads entry ids, metadata, and bodies;
it does not read generated footers as structural truth.
When no version is supplied, query reads the public store.

The default CLI query is vague.
Vague query matches text against an entry's id, name, description, and body,
and also against the ids, names, and descriptions of entries named by its structural fields.
Each text term must match somewhere in that expanded text.

Exact query is available through explicit exact flags.
Exact structural fields are conjunctive across distinct fields
and disjunctive inside one field.
A query for two categories matches entries in either category,
while a query for a category and a refiner requires both predicates.

Query output is presentation.
The selected entries remain ordinary parsed entries,
and callers may print summaries, ids, or paths without changing the store.

---

## Generated Footers

Sirno may generate and maintain a footer at the bottom of entries,
bounded by sentinels that state Sirno owns the region
and that humans and tools should leave it untouched.

The sentinels are human-visible Markdown block quotes.
They are separated from the generated list by blank lines,
so Markdown renderers do not nest the closing sentinel under the list.
When Sirno appends a generated region to a non-empty entry body,
it inserts a horizontal divider immediately before the region unless one is already present.

The generated body is grouped by configured structural field.
Each enabled group appears in the region.
A group with links begins with a plain label such as `Category (from)`, `Clustee (to)`,
or `Clique`, followed by a Markdown list of entry links.
A group with no links is rendered inline, such as `Clustee (from): (none)`.
If no generated-link group is enabled, the region contains `(none)`.

The footer format is a projection of metadata-derived structure,
maintained for external tools that navigate links.
Changing a generated link by hand does not change metadata.
Changing metadata and regenerating the footer is the correct edit path.
History commits remove generated regions before writing entry snapshots,
so history stores canonical metadata and prose rather than navigation projections.

`sirno check` reports stale generated regions when link checking is enabled.
`sirno gen-link` creates or replaces the generated region.
`sirno gen-link --dry` reports the generated regions that would change without writing files.
`sirno gen-link delete` removes the generated region.
Deleting generated links does not edit prose outside the guard-bounded region.

---

## Versioning

When history is configured,
Sirno versions the `sirno` form by committing the public Markdown store
into a separate `eter` history root.

A Sirno version is an `Eterator`:
an immutable global snapshot of the entry store.
It identifies the whole store state, not a single entry revision.
The value is a storage handle with ordering;
entry metadata does not store it,
and entry ids remain stable across versions.

The public store is always the editable working form.
The history root is private storage,
conventionally `sirno-history`.
It is not read as part of the entry store,
and it must not be placed where store discovery can treat it as entries.
`Sirno.lock` is the project-local state file for this relation.
It records one `[history]` table with `status`, `version`, and an optional `mutable` flag.

`sirno history init` configures the history root and creates the first history commit.
A history commit imports the selected public entry set and writes one `eter` transaction.
The transaction may touch one entry or many entries,
and all changed rows receive the same version.
Before writing the transaction,
Sirno removes every guard-bounded generated-link region from the committed entry bodies.
Generated links remain a public-store projection;
history stores metadata and prose without generated navigation regions.
A successful commit returns the new `Eterator`.
If the public store matches the current history snapshot,
the commit returns the current version without writing a new snapshot.
If an entry exists in the current history snapshot but is absent from the public store,
the commit writes an `eter` lifecycle deletion marker for that entry.
After a commit, `Sirno.lock` records `status = "current"`
and the committed `Eterator` version.

Direct edits to the public store are working-state edits.
They become history only after a commit.
Reading interfaces without a version selector read the public store.
A version selector reads from the history root and changes the observed store state
without changing query or check semantics.

Checkout materializes one history version into a public Markdown directory.
It resolves live entries at the selected `Eterator` and renders canonical entry files.
Checkout uses an explicit conflict policy;
the conservative policy writes only into an absent or empty target directory.
CLI checkout replaces managed Markdown entry files in the configured public store
while preserving ignored paths.
After checkout, `Sirno.lock` records `status = "checked-out"` and the selected version.

A normal checkout is immutable.
Sirno removes write permission from the public store root and managed entry files
so ordinary file edits fail at the filesystem boundary.
It also writes a visible Markdown blockquote at the start of each checked-out entry body
that marks the file as read-only and says not to edit it by hand.
`sirno history checkout VERSION --unsafe-mutable` leaves the checkout writable
and records `mutable = true` in `Sirno.lock`.
Committing an unsafe mutable checkout creates a new current history version
and rewrites the lock to `status = "current"`.
Sirno refuses to commit an immutable checkout.

History is field-level in `eter` and entry-level in Sirno.
Sirno may expose entry history, diffs, and restore operations by reading fields at successive snapshots,
but it presents those results as changes to entries and structural fields.
The public entry schema remains unchanged.

Restoring a version writes a snapshot back to the public store.
Committing the restored public store creates a new current history snapshot,
so later work stays ordered and old snapshots remain immutable.
Undo-tree branching belongs to git or another outer repository mechanism;
Sirno's own version line is linear.

Retention is policy.
Sirno may keep named versions, recent versions, versions tied to exported reviews, or all versions.
Unkept versions can be retired and garbage-collected through `eter`
only when no retained version needs their rows.
The filesystem backend does not persist retired-version state,
so Sirno must provide the live set when it performs collection.

---

## Storage And Interfaces

The entry storage model has one required surface and one optional surface.
The public Markdown store is the required working form.
The private history root is optional.
When configured, it uses `eter` for durable storage, indexing, immutable snapshots,
field history, version retirement, and garbage collection.

Sirno exposes the store through CLI and MCP interfaces,
and a lightweight GUI or Obsidian extension may later provide a direct editing experience.
Repository witnesses are managed through `mosaika`,
with the entry id serving as the query key Sirno uses when locating witness blocks.

The CLI is the first operational interface.
It can initialize stores, create entries, query entries, report status,
check structure, generate or delete link footers,
and emit shell completions.
The commands operate on the configured store by default
and accept explicit paths where a local operation needs a different entry directory.

`sirno status` summarizes the configured project.
It reports the config file, monograph, store, optional history root, entry count,
check policy, link policy, and current check result.
When history is configured, it also reports the `Sirno.lock` state.

`sirno new` creates one entry file from typed command-line metadata.
It refuses to overwrite an existing entry file.

`sirno history commit` commits the current public store into the configured history root.
It updates `Sirno.lock` to the resulting current version.
`sirno history checkout VERSION` materializes one version into the configured public store.
The checkout is immutable unless `--unsafe-mutable` is supplied.

`sirno query` is the reading interface over the Markdown store.
It defaults to vague text query and keeps exact structural predicates behind explicit exact flags.

`sirno witness ENTRY_ID` scans the configured code members
and reports repository witness blocks for the selected entry id.
`sirno witness ENTRY_ID --full` also prints the full matched code regions.
Witness output reports the opening and closing delimiter ranges.
In full mode, the summary line contains only the range.
A blank line separates the summary from the dedented region.
Multiple full regions are separated by a blank line, `---`, and another blank line.

`sirno util completion` emits shell completion scripts.
Utility commands do not read or mutate the store unless their own subcommand says so.

---

## Checks

Sirno checks structure, not semantic truth.

Structural checks include required metadata fields, accepted field shapes, reference existence,
generated footer boundaries, and witness lookup validity when requested.
When `[code].members` is configured,
review checks require each `witness:` entry to have at least one repository witness block.
They also report repository witness blocks that name missing entries.

Generated-link checking has two layers.
Sentinel structure is always checked.
Freshness is controlled by `[check].link`,
which is enabled by default.

Sirno validates references lazily:
dangling `category`, `clustee`, and `refiner` ids may warn during edits but become errors during an explicit check,
and witness validity is checked only during an explicit check.
Checks are light during editing and strict at explicit review boundaries,
so local movement stays fast while the final state stays sound.

---

## Planning

Planning lives in skills built on top of Sirno.

Entries are durable, named, and structured by metadata,
so Sirno can support persistent planning as an application of its core primitives.
A skill may use entries to represent a worklist,
treating the worklist as ordinary entries with categories, refiners, clustees, and witnesses.

---

## Future Work

The `locked` field is reserved for later design,
where it may define how entries or generated regions resist accidental edits.

Long-term version retention policy is reserved for later design.
The core model already treats versions as `eter` snapshots;
future work decides which snapshots Sirno keeps by default,
which snapshots can be named,
and how review interfaces expose them.

The exact naming of the four transforms may be refined;
the current names are `lower`, `raise`, `realize`, and `reflect`.

Planning skills are future work;
they may use Sirno entries to leave durable work artifacts without changing Sirno's core fields.

---

## Project Self-Application

This repository uses Sirno's own model.

`DESIGN.md` is the monograph,
`sirno-docs/` is the configured public store,
and `sirno-history/` is the configured history root.
The store contains compact entries for the concepts, structural fields, interfaces,
and implementation commitments described here.
The codebase will witness those entries through `mosaika`.

The document may grow long, but it should remain ordered as one narrative.
Local details that become too dense should be lowered into entries,
and raised back only when the monograph needs them.
