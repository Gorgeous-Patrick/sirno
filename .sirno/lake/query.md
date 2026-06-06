---
name: Query
desc: Selection of Sirno entries through vague text and structural filters.
category:
  - concept
belongs:
  - lake-projection
prerequisite:
  - interfaces
  - metadata
---

Query selects parsed *entries* from the lake or,
when anchor is configured,
from one *anchor* version.

It reads *entry* paths, metadata, and bodies.
The *generated footers* are projections for navigation,
not structural input to query.
When no version is supplied,
query reads the lake.

The mist design strengthens query into the shared selector mechanism.
`sirno query` prints selected entries.
A *mist* uses the same selector to render selected reservoir entries into a *misty lake*.
Filters and projection should not become separate languages.

The default query mode is vague text query.
It matches an *entry*'s address,
intrinsic field values,
and body.
It also matches the addresses and intrinsic field values
of *entries* named by structural links.

Vague query is for recall.
A user can search for nearby language without choosing the exact link relation first.
Each text term must match somewhere in the expanded *entry* text.

Target filters use `--has FIELD=ENTRY_ADDRESS[,ENTRY_ADDRESS]`.
Field state filters use `--is FIELD=present`, `--is FIELD=empty`, or `--is FIELD=missing`.
`empty` means the field is present with no targets.
`missing` means the field is absent.
Query combines text terms and distinct link relations with all-of logic.
Inside one relation,
comma-separated values, repeated `--has` flags, and `--is` states use any-of logic.
`--has category=concept,meta` means either category.
`--has category=concept --has refines=interfaces` requires both fields to match.
`--has refines=query --is refines=empty` matches either a `query` refinement
or a present empty `refines` field.

Query output is presentation.
The default output columns are `id` and `path`.
`sirno query --columns COLUMNS` accepts a comma-separated list of columns.
The built-in columns are `id` and `path`.
Discovered intrinsic metadata fields are printable scalar columns.
Configured structural link relation names are printable relation columns.
`name` and `desc` are selectable in this lake
because they are discovered intrinsic fields.
Query does not define them as built-in columns.
Intrinsic columns print plain strings when present in an *entry*'s ownership scope.
When a selected intrinsic field is absent from that scope,
JSON prints `null` and human tables print a blank cell.
Structural link columns print target entry addresses in metadata order.
When `sirno query --columns` has no value,
query prints every selectable column name and does not select entries.
Run query again with `--columns` to select entries.
`--format json` prints a JSON array of objects with the selected columns;
structural link column values are arrays,
and missing link relations are `null`.
`--format human` prints the same selected columns as a bordered Unicode table.
In a terminal, the table detects the available width and wraps cell content.
When the selected columns cannot fit, Sirno keeps the leftmost columns that fit
and appends a `...` column to show that columns were omitted.
When no format is supplied,
query uses `human`.
