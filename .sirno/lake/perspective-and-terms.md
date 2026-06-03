---
name: Perspective and Terms
desc: The reader-facing distinction between Sirno as tool and this self-hosted lake.
category:
  - meta
  - concept
belongs:
  - narrative
prerequisite:
  - project-self-application
---

Sirno's self-application creates a perspective problem.

`Sirno` can name the tool and model.
It can also appear inside this repository,
where Sirno uses its own model to describe itself.
Both readings are true,
but mixed language can make a reader wonder whether a sentence describes the general tool
or this self-hosted documentation.

This *lake* resolves the ambiguity through stable perspective labels.
Use `Sirno` for the tool and project model.
Use `a Sirno-managed project` for any project that applies Sirno.
Use `this repository` for the implementation workspace for Sirno.
Use `this lake` for the self-hosted Sirno Lake that describes Sirno.
Its tracked reservoir lives at `.sirno/lake`.
Its default misty workspace renders to `sirno-lake/`.

Sirno-specific terms are capitalized when they appear as part of a Sirno proper name,
such as Sirno Lake.
When lake and anchor appear together,
keep both lowercase.
Otherwise, lowercase and italicize them when they carry Sirno-specific meaning:
*lake*, *entry*, *witness*, *anchor*, *ripple*, *transform*, and *repository*.
Generic words stay plain when they are not carrying Sirno structure:
project, documentation, reader, history, change, review, and work.

Agent-procedure entries are exempt from the italic rule.
The discipline entries that `belongs` to `agent-skills`,
and the `agent-skills` roster itself,
are written as terse operational instructions.
There the markup adds noise instead of clarity,
so they name Sirno terms in plain prose.
The exemption is for instructional procedure, not for concept or narrative *entries*.

This distinction lets ordinary project prose stay ordinary.
When a document says "the project documentation",
it can mean documentation in a normal repository.
When it says "the *lake*",
the reader is inside Sirno's model.
When it says "this *lake*",
the reader is looking at Sirno's self-application in `.sirno/lake`.

This repository now applies that convention in the main route.
The introduction names this repository and this *lake* before it asks the reader to follow *entries*.
The Methodology tells contributors which perspective to use before editing.
Project Self-Application explains why the recursive form exists
and how the terms keep it readable.
