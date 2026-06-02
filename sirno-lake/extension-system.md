---
name: Charm And Spell System
desc: A proposal for resolving entry-owned charms into invokable spells.
category:
  - concept
  - proposal
belongs:
  - future-work
  - entry-artifact
  - interfaces
prerequisite:
  - entry-artifact
  - project-config
  - interfaces
refines:
  - future-work
---

The charm and spell system lets a Sirno-managed project run user-owned code
from Sirno hook points.

A *charm* is an *entry* whose artifact tree declares one runnable bundle.
The artifact tree may contain a script, a native executable,
or source files for building an executable.
The entry states the design intent for the charm.
The artifact tree stores the runnable material that actualizes that intent.

A *spell* is the ready-to-run script or executable resolved from a *charm*.
A direct charm resolves an artifact script or executable into a spell.
A source charm builds a spell into Sirno cache state.
Hooks invoke spells.
Charm commands prepare charms.

A charm is identified by a charm manifest inside the owner artifact tree.
The manifest records the spell command shape, optional charm preparation commands,
and the hooks for which the charm is eligible.
Hook names and hook payloads are defined by later hook entries.
The charm and spell system treats a hook as a named event
with a declared payload and result contract.

Entry artifacts remain opaque lake state.
Sirno interprets an artifact tree as a charm only when a charm manifest is present
and the owning entry is enabled by local project configuration.
Artifact commands continue to manage bytes.
Charm commands interpret those bytes for preparation.
Spell commands invoke resolved runtime artifacts.

## Manifest

The manifest should use TOML and live at `Sirno.charm.toml` under the owner artifact root.
It should contain these concepts:

| Field | Meaning |
|---|---|
| `spell.command` | The argv vector used to invoke the resolved spell. |
| `charm.setup.command` | Optional argv vector used to prepare local dependencies or generated inputs. |
| `charm.check.command` | Optional argv vector used to type check or validate the charm. |
| `charm.build.command` | Optional argv vector used to build source into a spell. |
| `charm.build.output` | Optional artifact-relative or cache-relative spell path produced by the build. |
| `hooks` | Hook ids for which the charm is eligible. |
| `inputs` | Optional declared needs, such as repo paths, lake paths, or stdin JSON. |

Sirno should execute commands as argv vectors.
It should not pass manifest commands through a shell.
Shell scripts remain supported by making the shell or interpreter the first argv element.

## Enablement And Trust

Charm enablement is a local opt-in decision.
A project enables charm entries in `Sirno.toml`, not in the artifact tree alone.
A present manifest is discoverable design data.
An enabled charm is executable project policy.

The operator grants each spell resolved from an enabled charm
the same filesystem authority as the Sirno process.
Sirno should make that authority explicit in CLI output and config comments.
It should not imply sandboxing unless a later implementation provides one.

Frozen entries protect the charm artifact tree from accidental mutation.
They do not make the code trusted.
Trust comes from local enablement and ordinary repository review.
Anchor and Tide already track owner artifact-tree fingerprints,
so charm code changes participate in lake review like other entry artifact changes.

## Charm Resolution

A charm with `charm.build.command` is a source charm.
Sirno builds it before the first invocation that needs the spell.
The build writes spell output under `.sirno/spells/<entry-address>/<fingerprint>/`.
The fingerprint includes the entry metadata, manifest, artifact tree, and relevant Sirno version.

Build output is cache state, not lake state.
Changing charm source, manifest fields, or build-relevant metadata invalidates the cache.
A failed build fails the hook invocation that required it.

Before a build,
Sirno may run charm setup or check commands when the manifest declares them.
Those commands prepare or validate the charm.
They do not invoke the spell unless the hook or operator requested invocation.

A charm without `charm.build.command` is a direct charm.
Sirno resolves its `spell.command` against the owner artifact root and the project root.
The manifest should state which paths are artifact-relative and which are project-relative.

## Hook Invocation

A hook invocation supplies an event envelope to the spell.
The envelope contains the hook id, project root, lake root, Sirno version,
the charm entry address, and hook-specific payload.
The hook entry defines the payload and the meaning of spell stdout.

Sirno captures stdout, stderr, exit status, and elapsed time for each spell run.
The hook entry defines whether stdout is ignored, parsed as JSON,
or treated as user-facing text.
Stderr is diagnostic output.
A non-zero exit status fails the spell run.
The hook entry defines whether that failure blocks the Sirno operation or is only reported.

Spells run in deterministic order for a hook.
The default order is the order of enabled charms in `Sirno.toml`.
A later ordering policy may refine this without changing artifact storage.

## Interfaces

The CLI should expose charm discovery, enablement, setup, check, build,
and cache-clean commands under `sirno charm`.
It should expose invocation and run inspection commands under `sirno spell`.
Compilation, type check, setup, and cache maintenance are charm operations.
Hook invocation and direct runtime execution are spell operations.
The MCP surface may expose discovery and status.
Execution should remain a human-enabled project policy,
because MCP clients may run in contexts where silent code execution is surprising.

Hook entries should refine this proposal.
Each hook entry should name the hook id, trigger point, event payload,
stdout contract, failure policy, and ordering constraints.
The charm and spell system supplies runnable artifacts and process execution.
The hook design supplies when and why spells run.

---

> **Sirno generated links begin. Do not edit this section.**

- belongs (to):
  - [entry-artifact](entry-artifact.md)
  - [future-work](future-work.md)
  - [interfaces](interfaces.md)
- belongs (from): (none)

> **Sirno generated links end.**
