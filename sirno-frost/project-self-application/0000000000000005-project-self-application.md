---
desc: The use of Sirno's own model to describe Sirno itself.
lifecycle: Active
name: Project Self-Application
structural:
  category:
  - concept
  belongs:
  - sirno
---

Sirno describes its own design through its own model.

This repository is a Sirno-managed project whose design subject is Sirno itself.
That creates a recursive reading:
Sirno is the tool being described,
and Sirno is also the tool used to organize the description.

The *lake* makes Sirno's design addressable in small *entries*.
A materialized introduction *entry* gives the *lake* a first narrative route.
A materialized methodology *entry* gives the *lake* a working guide.
An optional Sirno Frost path can preserve frozen *lake* snapshots.
Repository artifacts can witness *entries* through `mosaika`.

The recursive form is useful,
but it can blur perspective when the prose shifts between Sirno as tool
and this repository as a project that uses Sirno.
This *lake* now keeps those readings separate through explicit perspective labels.
`Sirno` names the tool and project model.
`a Sirno-managed project` names any project that applies Sirno.
`this repository` names the implementation workspace for Sirno.
`this lake` names `sirno-docs/`,
the self-hosted Sirno Lake that describes Sirno.

The introduction should stay readable as one route.
Local details that become dense should stay in *entries*
and be linked through categories, `belongs`, `refines`, and *witnesses*.

This self-application exercises the design under its own constraints.
When implementation work changes the model,
that change can be internalized into the *lake* before any narrative route is revised.

The `meta` category is the bootstrap surface.
It contains *entries* that answer how a Sirno-managed project wants its documentation to develop.
Those *entries* are available to people, agents, and tools before they revise the rest of the *lake*.

The *lake* should name the objects the project expects future work to cite:
*forms*, *entries*, *structural fields*, *transforms*, metadata,
checks, *generated footers*, *witnesses*, and storage boundaries.
Those names become the handles used by code work, documentation work, and review.

Sirno terms become proper names when they appear with Sirno:
Sirno Lake and Sirno Frost.
Otherwise, lowercase italics mark local model terms:
*lake*, *entry*, *witness*, *ripple*, *transform*, and *repository*.
Ordinary words stay plain when they describe normal project work.
That vocabulary boundary lets Sirno explain any project,
including this repository,
without making every sentence sound like it belongs to the tool's internal model.

Sirno does not just document the project;
it lets the project document its own documentation method.

Repository *witnesses* make self-application stronger.
When code actualizes *entry* parsing, *generated footer* handling, or *structural checks*,
that code can be placed inside a *witness* block for the relevant *entry* id.
Then Sirno can answer both sides of a design question:
what does this *entry* mean,
and where is it witnessed?
