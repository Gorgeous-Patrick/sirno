---
name: Entry Artifact
desc: A lake-owned file attached to one entry and versioned with Sirno Frost.
category:
  - concept
belongs:
  - sirno-lake
  - frost-versioning
---

An *entry artifact* is a lake-owned file attached to one *entry*.

Entry artifacts live under `.artifacts/<entry-id>/...` inside the public *lake*.
The *entry* Markdown files remain flat at the *lake* root as `<entry-id>.md`.
The first component under `.artifacts` is the owner *entry* id.
It must name an existing *entry*.
The artifact path after that owner is a relative UTF-8 path with only normal components.
Artifact content is opaque bytes.

Entry artifacts are public *lake* state.
They are different from *repository witnesses*.
A *witness* stays in a configured *repository* artifact and points back to an *entry* claim.
An *entry artifact* belongs to the *entry* itself and moves with that *entry* when the id is renamed.

`sirno path ENTRY_ID` shows the public and Frost paths related to an *entry*.
It includes artifact paths by default and excludes *repository witness* paths.
`sirno artifact` manages owner-relative artifact paths as a top-level entry operation.
Its grouped form is `sirno entry artifact`.
Artifact mutation commands preserve the same protection rule as direct file edits:
a frozen *entry* blocks changes to its artifact tree.

When Sirno Frost is configured,
artifacts are committed into the private *frost* path with the *entries* they belong to.
Frost stores each artifact as a separately versioned backend object,
so an artifact-only change can produce a new snapshot without rewriting the *entry* row.
A frozen public *entry* protects its artifact tree as part of the same public bundle.
Checkout restores both the flat Markdown *entries* and the `.artifacts` tree for the selected snapshot.

---

> **Sirno generated links begin. Do not edit this section.**

- belongs (to):
  - [frost-versioning](frost-versioning.md)
  - [sirno-lake](sirno-lake.md)
- belongs (from): (none)

> **Sirno generated links end.**
