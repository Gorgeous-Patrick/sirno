---
name: Extension System
desc: A proposal for running entry-owned executable artifacts from Sirno hook points.
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

The extension system lets a Sirno-managed project run user-owned code from Sirno hook points.

An *extension* is an *entry* whose artifact tree declares one runnable bundle.
The artifact tree may contain a script, a native executable,
or source files for building an executable.
The entry states the design intent for the extension.
The artifact tree stores the runnable material that actualizes that intent.

A runnable bundle is identified by an extension manifest inside the owner artifact tree.
The manifest records the command shape, the optional build shape,
and the hooks for which the bundle is eligible.
Hook names and hook payloads are defined by later hook entries.
The extension system treats a hook as a named event with a declared payload and result contract.

Entry artifacts remain opaque lake state.
Sirno interprets an artifact tree as an extension only when an extension manifest is present
and the owning entry is enabled by local project configuration.
Artifact commands continue to manage bytes.
Extension commands interpret those bytes for execution.

## Manifest

The manifest should use TOML and live at `sirno-extension.toml` under the owner artifact root.
It should contain these concepts:

| Field | Meaning |
|---|---|
| `command` | The argv vector used to run a script or executable. |
| `build.command` | Optional argv vector used to build source into a runnable output. |
| `build.output` | Optional artifact-relative or cache-relative executable path produced by the build. |
| `hooks` | Hook ids for which the extension is eligible. |
| `inputs` | Optional declared needs, such as repo paths, lake paths, or stdin JSON. |

Sirno should execute commands as argv vectors.
It should not pass manifest commands through a shell.
Shell scripts remain supported by making the shell or interpreter the first argv element.

## Enablement And Trust

Extension execution is a local opt-in decision.
A project enables extension entries in `Sirno.toml`, not in the artifact tree alone.
A present manifest is discoverable design data.
An enabled manifest is executable project policy.

The operator grants the enabled extension the same filesystem authority as the Sirno process.
Sirno should make that authority explicit in CLI output and config comments.
It should not imply sandboxing unless a later implementation provides one.

Frozen entries protect the extension artifact tree from accidental mutation.
They do not make the code trusted.
Trust comes from local enablement and ordinary repository review.
Anchor and Tide already track owner artifact-tree fingerprints,
so extension code changes participate in lake review like other entry artifact changes.

## Build Resolution

An extension with `build.command` is a source extension.
Sirno builds it before the first run that needs the executable.
The build writes output under `.sirno/extensions/<entry-address>/<fingerprint>/`.
The fingerprint includes the entry metadata, manifest, artifact tree, and relevant Sirno version.

Build output is cache state, not lake state.
Changing extension source, manifest fields, or build-relevant metadata invalidates the cache.
A failed build fails the hook invocation that required it.

An extension without `build.command` is a direct extension.
Sirno resolves its `command` against the owner artifact root and the project root.
The manifest should state which paths are artifact-relative and which are project-relative.

## Hook Invocation

A hook invocation supplies an event envelope to the extension.
The envelope contains the hook id, project root, lake root, Sirno version,
the owning entry address, and hook-specific payload.
The hook entry defines the payload and the meaning of extension stdout.

Sirno captures stdout, stderr, exit status, and elapsed time for each extension run.
The hook entry defines whether stdout is ignored, parsed as JSON,
or treated as user-facing text.
Stderr is diagnostic output.
A non-zero exit status fails the extension run.
The hook entry defines whether that failure blocks the Sirno operation or is only reported.

Extensions run in deterministic order for a hook.
The default order is the order of enabled entries in `Sirno.toml`.
A later ordering policy may refine this without changing artifact storage.

## Interfaces

The CLI should expose extension discovery, enablement, build, run, and cache-clean commands
under a stable command group such as `sirno extension`.
The MCP surface may expose discovery and status.
Execution should remain a human-enabled project policy,
because MCP clients may run in contexts where silent code execution is surprising.

Hook entries should refine this proposal.
Each hook entry should name the hook id, trigger point, event payload,
stdout contract, failure policy, and ordering constraints.
The extension system supplies runnable artifacts and process execution.
The hook design supplies when and why those artifacts run.

---

> **Sirno generated links begin. Do not edit this section.**

- belongs (to):
  - [entry-artifact](entry-artifact.md)
  - [future-work](future-work.md)
  - [interfaces](interfaces.md)
- belongs (from): (none)

> **Sirno generated links end.**
