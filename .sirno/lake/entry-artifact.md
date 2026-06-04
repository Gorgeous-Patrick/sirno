---
name: Entry Artifact
desc: A lake-owned file attached to one entry and tracked with the lake.
category:
  - concept
  - implemented
belongs:
  - entry
  - sirno-anchor
  - storage
  - sirno-lake
prerequisite:
  - entry
  - sirno-anchor
---

An *entry artifact* is a lake-owned file attached to one *entry*.

Entry artifacts live under `.artifacts/<entry-address>/...` inside the Sirno Lake.
The `.artifacts` component uses the built-in `.<id>` path form.
The owner directory under `.artifacts` is the dot-joined owner *entry address*.
It must resolve to an existing *entry*.
The artifact path after that owner is a relative UTF-8 path with only normal components.
Artifact content is opaque bytes.

Entry artifacts are Sirno Lake state.
They are different from *repository witnesses*.
A *witness* stays in a configured *repository* artifact and points back to an *entry* claim.
An *entry artifact* belongs to the *entry* itself
and moves with that *entry* when the path is renamed.

`sirno entry path ENTRY_ADDRESS` shows the lake paths related to an *entry*.
It includes artifact paths by default and excludes *repository witness* paths.
`sirno artifact` manages owner-relative artifact paths as a top-level entry operation.
Its grouped form is `sirno entry artifact`.
Artifact mutation commands preserve the same protection rule as direct file edits:
a frozen *entry* blocks changes to its artifact tree.

Git stores artifact bytes as ordinary lake files.
Anchor stores one owner artifact-tree fingerprint for an entry when that entry owns artifacts.
This lets Tide detect artifact changes without copying artifact bytes into Anchor.
