---
desc: A principle for writing the next project state from an immutable snapshot of the old one.
name: Immutable Update
category:
  - meta
  - concept
belongs:
  - methodology
prerequisite:
  - methodology
  - design-source-authority
  - min-cut
  - semantic-locality
refines:
  - documentation-driven-development
  - min-cut
  - semantic-locality
---

Immutable update is the project-writing principle that treats the previous repository and *lake*
as an immutable snapshot.

The snapshot is source material, not a shape that must survive.
Before editing, describe the desired next state as if writing it fresh from the approved facts
in the snapshot.
Then carry forward only the definitions, boundaries, evidence, and code paths
that still earn their place.

This principle makes change pay the cost of reuse.
Existing prose and code are easy for an agent to keep because they are already present.
That inertia can turn local edits into layered special cases.
Immutable update asks the agent to compare two costs:
the cost of replacing a local shape
and the cost of preserving it with another wrapper.

In documentation, write the new canonical entry or section around the current idea.
Use the old wording to recover stable commitments, examples, and constraints.
Let the new idea choose its own structure.
When the old entry owns the same commitment,
rewriting it completely can be clearer than patching sentences.

In code, use the old implementation as evidence and test material.
When the desired behavior has a simpler structure than the current implementation,
prefer replacing the local structure over adding compatibility logic around it.
Keep transitional layers only when the task names them as a requirement.

The previous state remains available through git, Anchor, Tide, witnesses, and review history.
Its job is to make the next state accountable.
The current codebase and *lake* should read as the best present design,
not as a sediment of every path taken to reach it.

A practical check:

- What would the desired state look like if the current files were read-only?
- Which old facts are still true and must be carried forward?
- Which old structures exist only because editing around them was cheaper than replacing them?
- Would a replacement lower total code or prose complexity?
- Does the new state explain the current design without narrating obsolete paths?
