---
name: Spell Invocation
desc: The proposed hook execution contract for resolved spells.
category:
  - concept
  - proposal
belongs:
  - extension-system
prerequisite:
  - spell
  - charm-enablement
refines:
  - extension-system
---

*Spell invocation* is the hook execution contract for a resolved spell.

Direct operator execution already runs resolved spells through `sirno spell run`.
This entry owns the remaining hook contract around the same runtime object.

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
