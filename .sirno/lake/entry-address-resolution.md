---
name: Entry Address Resolution
desc: The lookup rule that resolves entry addresses into entries.
category:
  - concept
belongs:
  - entry
prerequisite:
  - entry
  - sirno-lake
---

*Entry address resolution* is the lookup rule that turns an *entry address* into an *entry*.

It groups the address vocabulary.
An *entry atom* is one filesystem-safe segment.
An *entry domain* is a non-final atom prefix that maps to a lake folder.
An *entry address* is the dot-joined form a reader or tool follows.

Inside one lake,
resolution maps address segments to a Markdown file under the configured lake path.
In a composed lake,
several addresses may resolve to the same entry through sheaf rules.
The address is therefore the route used for lookup,
not necessarily the entry's durable identity.

The entry metadata field `name` is separate.
It is display text for readers,
not the address syntax that Sirno resolves.
