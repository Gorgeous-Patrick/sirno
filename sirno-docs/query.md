---
name: Query
desc: Selection of Sirno entries through vague text and structural filters.
category:
  - concept
belongs:
  - lake
prerequisite:
  - interfaces
  - metadata
---

Query selects parsed *entries* from the public *lake* or,
when Sirno Frost is configured,
from one *frost* version.

It reads *entry* ids, metadata, and bodies.
The *generated footers* are projections for navigation,
not structural input to query.
When no version is supplied,
query reads the public *lake*.

The default query mode is vague text query.
It matches an *entry*'s id, name, desc, and body.
It also matches the ids, names, and `desc` values of *entries* named by structural fields.

Vague query is for recall.
A user can search for nearby language without choosing the exact *structural field* first.
Each text term must match somewhere in the expanded *entry* text.

Target filters use `--has FIELD=ENTRY_ID[,ENTRY_ID]`.
Field state filters use `--is FIELD=present`, `--is FIELD=empty`, or `--is FIELD=missing`.
`empty` means the field is present with no targets.
`missing` means the field is absent.
Query combines text terms and distinct *structural fields* with all-of logic.
Inside one structural field,
comma-separated values, repeated `--has` flags, and `--is` states use any-of logic.
`--has category=concept,meta` means either category.
`--has category=concept --has refines=interfaces` requires both fields to match.
`--has refines=query --is refines=empty` matches either a `query` refinement
or a present empty `refines` field.

Query output is presentation.
`sirno query --columns` accepts a comma-separated list of columns.
The printable columns are `id`, `name`, `path`, and `desc`.
When no columns are supplied,
query selects `id,path,name`.
`--format json` prints a JSON array of objects with the selected columns.
`--format human` prints the same selected columns as a bordered Unicode table.
In a terminal, the table detects the available width and wraps cell content.
When the selected columns cannot fit, Sirno keeps the leftmost columns that fit
and appends a `...` column to show that columns were omitted.
When no format is supplied,
query uses `human`.

---

> **Sirno generated links begin. Do not edit this section.**

- belongs (to):
  - [lake](lake.md)
- belongs (from): (none)

> **Sirno generated links end.**
