---
name: Clustee
description: A relation that groups an entry by a named clique closure.
category:
  - relation
clustee:
  - relation
refiner:
  - relation
---

`clustee` groups an entry into a named clique.

The named entry is the clique closure.
It gives the shared subject, local vocabulary, or design neighborhood a place to be named and explained.

This relation covers the useful part of tags, scopes, namespaces, and domains.
The member entries keep their own ids while the clique entry provides a route into the group.

A two-member clique closure can describe an undirected relation by recording why the two entries belong together.

Use `clustee` when several entries should be visited together.
The relation says that the member belongs to a named neighborhood,
not that it is an instance of a kind and not that it refines the clique entry.
The clique closure can explain the shared theme,
the local vocabulary,
or the reason the entries should be considered as a set.

This is useful for domains that cut across categories.
For example, a store neighborhood may include concepts, metadata rules,
generated footer behavior, and checks.
Those entries are different kinds of objects,
but they belong together because they explain the same part of the project.

The closure entry should carry real explanatory value.
If the group name is only a loose tag,
it may not deserve a clustee relation yet.
If the group helps a reader enter a complicated region of the store,
then the closure gives that region a stable front door.

> **Sirno generated links begin. Do not edit this section.**
## Sirno Links

- clustee: [relation](relation.md)
> **Sirno generated links end.**
