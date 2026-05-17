---
desc: A user-shaped route through Sirno concepts that can be saved as a narrative entry.
lifecycle: Active
name: Interactive Narrative Session
structural:
  category:
  - concept
  refines:
  - narrative
---

An interactive narrative session adapts Sirno concepts to a reader's task before it saves a route.

It begins by naming the reader, the task, and the design pressure that makes a route useful.
The session asks questions when an answer would change the sequence,
then turns feedback into earlier concepts, deferred detail, or a sharper endpoint.

The session treats *entries* as canonical knowledge.
It uses a narrative to choose order, prerequisites, and motivation,
then points back to *entries* for durable detail instead of copying the *lake*.
A route orders the reader's path to canonical knowledge; it does not sell or replace it.

The route should make knowledge worth moving toward before it makes knowledge complete.
That pull may be practical, aesthetic, playful, urgent, elegant, sexy, desire-shaped, or clarifying.
Desire is safe to name when it is real, but no route should reduce every reader to one desire.
The route shows tension before explanation, gives a clean first bite,
adds texture through examples and consequences, keeps sequence tight, honors the reader's agency,
and leaves an aftertaste the reader can reuse.

The materialized artifact is a narrative *entry*.
It records who the route serves, why it matters, which *entries* appear in order,
and which details remain deferred.
The local `sirno-narrative-session` skill implements this workflow for agent conversations.
