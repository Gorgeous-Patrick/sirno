---
name: Charm Manifest
desc: The Sirno.charm.toml file that declares how a charm resolves into a spell.
category:
  - concept
  - proposal
belongs:
  - extension-system
  - entry-artifact
prerequisite:
  - charm
  - spell
refines:
  - extension-system
---

A *charm manifest* is the `Sirno.charm.toml` file inside a charm artifact root.

The manifest declares how Sirno prepares a charm
and how it invokes the resolved spell.
It records command shapes as argv vectors.
Sirno should not pass manifest commands through a shell.
Shell scripts remain supported by making the shell or interpreter the first argv element.

The manifest should contain these concepts:

| Field | Meaning |
|---|---|
| `spell.command` | The argv vector used to invoke the resolved spell. |
| `charm.setup.command` | Optional argv vector used to prepare local dependencies or generated inputs. |
| `charm.check.command` | Optional argv vector used to type check or validate the charm. |
| `charm.build.command` | Optional argv vector used to build source into a spell. |
| `charm.build.output` | Optional artifact-relative or cache-relative spell path produced by the build. |
| `hooks` | Hook ids for which the charm is eligible. |
| `inputs` | Optional declared needs, such as repo paths, lake paths, or stdin JSON. |

The manifest makes a charm discoverable.
It does not make the charm executable project policy.
Project configuration owns enablement.
