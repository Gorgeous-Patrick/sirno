---
name: Narrative Serializer
desc: The deterministic contract that turns a narrative session into a lake entry.
category:
  - concept
belongs:
  - interactive-narrative-session
prerequisite:
  - interactive-narrative-session
---

The narrative serializer turns a finished session into a lake entry by a deterministic contract.

A session first records compact notes, not a transcript.
The notes name the reader and task,
the design pressure that makes a route useful,
the pull or tension that makes the next concept worth meeting,
known and missing terms,
the ordered route of steps,
durable feedback,
deferred detail,
and an aftertaste phrase, handle, or entry address.
Each route step records an entry address or proposed address,
its role,
the prerequisite it satisfies,
and the detail deferred to an entry body.

The serializer input is a separate, smaller shape.
It carries an `id`,
an optional `intrinsic` map from field name to scalar string,
a `structural` map from field name to a list of entry addresses,
and a `body` as a list of paragraph strings.
The notes are scaffolding for the route; the input is what becomes the file.

The contract holds these invariants.
The entry address uses lowercase kebab-case atoms.
Intrinsic fields are written exactly as supplied,
so the caller must use the active project's discovered intrinsic registry.
Intrinsic fields and Sirno-managed fields are never written as structural links.
Structural links are written exactly as supplied, in the order given,
because their order is user-managed and Sirno renders configured surfaces in that order.
Empty fields are omitted, and `witness:` is added only when repository evidence exists.
Serialization is deterministic and refuses to overwrite an existing entry unless overwrite is
explicitly requested; a dry run can preview the entry without writing.

The materialized entry body answers a fixed set of questions:
who the route serves,
what design pressure makes it useful,
what pull makes the next concept worth meeting,
what must be understood first,
which entries carry the ordered route,
what local detail is deferred,
what phrase or handle remains afterward,
and what durable feedback shaped the route.
The body names entries and explains their order; it does not copy their definitions.

The serializer is an implementation of this contract.
The contract is the durable design fact and lives here so a session tool can be rebuilt from it.
