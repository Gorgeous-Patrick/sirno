---
name: Anchor File
desc: The tracked TOML file that records the accepted lake baseline.
category:
  - concept
  - implemented
belongs:
  - sirno-anchor
  - storage
prerequisite:
  - sirno-anchor
  - entry-artifact
  - structural-edge-policy
---

The Anchor file is `.sirno/anchor.toml`.
It stores the minimum shared state needed to compare the current waterline
against the accepted Sirno Lake.

```toml
schema = 1
lake = "sirno-lake"

[entries.methodology]
fingerprint = "sha256:..."
belongs = ["narrative"]
prerequisite = ["introduction", "sirno-witness"]

[entries.entry-with-artifacts]
fingerprint = "sha256:..."
artifact_fingerprint = "sha256:..."
```

`schema` selects both the TOML shape and the fingerprint semantics.
`lake` names the lake path that the file describes.
`entries` stores one record for each live entry,
keyed by entry address.

Each entry record stores:

- `fingerprint`, the canonical entry fingerprint;
- `artifact_fingerprint`, when the entry owns artifacts;
- structural link fields needed for Anchor-side Tide traversal.

Schema 1 entry fingerprints use Sirno's canonical entry Markdown.
Sirno parses the entry, deletes generated-link regions,
normalizes line endings,
renders the entry through the canonical Markdown renderer,
and hashes that source with a `sirno-entry-v1` domain prefix.
The public value is `sha256:` followed by lowercase hexadecimal digest text.

Artifact fingerprints hash the owner artifact tree.
The tree source orders artifacts by path
and includes each artifact path, byte length, and content bytes.
The Anchor file stores only the resulting fingerprint.

The Anchor file contains no authoritative Git object ids.
Rebase, squash, and garbage collection cannot make Anchor references dangle.
Any future Git hint must be non-authoritative
and safe to ignore when stale.
