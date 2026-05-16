---
name: Introduction
desc: A first narrative route through Sirno's project model.
category:
  - narrative
belongs:
  - sirno
refines:
  - narrative
---

*Semantic Intermediate Representation of Nominal Objects*

Sirno compiles between design forms for design-aware programming work,
moving among an optional *monograph*,
a *lake* of compact named Markdown *entries*,
and the *repository*.

This repository is a Sirno-managed project whose design subject is Sirno itself.
General claims describe Sirno as the tool and project model.
`this repository` names the implementation workspace for Sirno.
`this lake` names `sirno-docs/`,
the self-hosted Sirno Lake that describes Sirno.
The `perspective-and-terms` *entry* states this convention directly.

Design needs a form that humans can read,
tools can index,
and agents can manipulate without carrying a whole project in context.
Sirno gives design that form by naming it.
The resulting names are readable by humans,
stable for tools,
and small enough to circulate.

The central form is the Sirno Lake.
The *lake* is a directory of Markdown *entries*.
Each *entry* has a stable id, a short metadata block, and a body of prose.
The id gives humans, tools, and agents a nominal handle for the thing being discussed.
The prose keeps the handle meaningful.

Sirno keeps its metadata vocabulary small.
`category` says what kind of *entry* something is.
`belongs` places an *entry* in one or more review neighborhoods.
`refines` connects a local *entry* back to the broader *entry* it makes concrete.
These recommended fields form this repository's structural graph.
Other Sirno projects may configure a different structural field set.
Repository *witness* status is discovered through `sirno witness ENTRY_ID --full`.
The *structural fields* are explicit metadata,
so tools can query them without pretending to understand the whole design semantically.

The *lake* is not only a glossary.
An *entry* should carry enough meaning to help future work.
Some *entries* define concepts.
Some *entries* give narrative routes through those concepts.
Some *entries* name local implementation commitments,
storage boundaries, generated regions, or *witness* lookup behavior.
The point is to preserve the design object that a later edit or review should be able to cite.

Sirno also names movements between forms.
`lower` moves narrative design into *lake entries*.
`realize` uses *entries* to guide repository work.
`reflect` records durable design facts learned from the *repository* back into the *lake*.
`raise` composes *lake entries* into a readable *monograph* when a project wants one.
These *transforms* are vocabulary for work.
They do not make Sirno a judge of design quality.
They make the relevant design objects easier to name and inspect.

Repository *witnesses* close the loop with implementation.
A *witness* block lives in a configured repository member,
opens with `sirno:witness:<entry-id>:begin`,
and closes with `sirno:witness:<entry-id>:end`.
Sirno asks `mosaika` to locate those regions by *entry* id.
The *entry* states the claim.
The repository region shows where that claim can be inspected.

The *generated footers* are an interoperability layer.
Sirno can project selected metadata fields as Markdown links at the bottom of *entries*.
The footer is guard-bounded and Sirno-owned.
Metadata remains the source of structural truth.
The footer only helps editors and documentation tools follow the graph.

This repository now treats `sirno-docs/` as the design source.
The introduction you are reading is the first route through this *lake*.
The `methodology` *entry* is the compact working guide for acting inside this *lake*.
The detailed design lives in the *entries* themselves:
*forms*, *structural fields*, *transforms*, storage, checks, *witnesses*, and *generated footers*.
Read this *entry* first,
then follow `belongs`, `refines`, and *witnesses* to the local design you need.

---

> **Sirno generated links begin. Do not edit this section.**

- belongs (to):
  - [sirno](sirno.md)
- belongs (from): (none)

> **Sirno generated links end.**
