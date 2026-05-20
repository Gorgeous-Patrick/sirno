---
name: Entry Artifact
desc: A lake-owned file attached to one entry and versioned with frost.
category:
  - concept
belongs:
  - entry
  - sirno-frost
prerequisite:
  - entry
  - sirno-frost
---

An *entry artifact* is a lake-owned file attached to one *entry*.

Entry artifacts live under `.artifacts/<entry-id>/...` inside the Sirno Lake.
The *entry* Markdown files remain flat at the *lake* root as `<entry-id>.md`.
The first component under `.artifacts` is the owner *entry* id.
It must name an existing *entry*.
The artifact path after that owner is a relative UTF-8 path with only normal components.
Artifact content is opaque bytes.

Entry artifacts are Sirno Lake state.
They are different from *repository witnesses*.
A *witness* stays in a configured *repository* artifact and points back to an *entry* claim.
An *entry artifact* belongs to the *entry* itself and moves with that *entry* when the id is renamed.

`sirno path ENTRY_ID` shows the lake and frost paths related to an *entry*.
It includes artifact paths by default and excludes *repository witness* paths.
The frost artifact store is sparse and versioned,
so the path command names the entry's frost root rather than inventing per-artifact backend paths.
`sirno artifact` manages owner-relative artifact paths as a top-level entry operation.
Its grouped form is `sirno entry artifact`.
Artifact mutation commands preserve the same protection rule as direct file edits:
a frozen *entry* blocks changes to its artifact tree.

When frost is configured,
artifacts are committed into the frost path with the *entries* they belong to.
The *entry* frost Markdown row stores the owner-relative artifact path list.
This manifest records which artifacts exist at that *entry* version.
Changed artifact bytes live beside the Markdown row in a matching version directory,
with the syntax `<16-hex-version>-<entry-id>/`.
Unchanged artifact bytes are inherited from older version directories.
`sirno frost gc` removes artifact byte files unreachable from the kept latest snapshot.
It preserves older byte files when the latest artifact manifest still inherits them.
A frozen lake *entry* protects its artifact tree as part of the same lake bundle.
Checkout restores both the flat Markdown *entries* and the `.artifacts` tree for the selected snapshot.

---

> **Sirno generated links begin. Do not edit this section.**

- belongs (to):
  - [entry](entry.md)
  - [sirno-frost](sirno-frost.md)
- belongs (from): (none)

> **Sirno generated links end.**
