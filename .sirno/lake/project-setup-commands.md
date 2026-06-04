---
name: Project Setup Commands
desc: The command surface for initializing a Sirno-managed repository.
category:
  - concept
  - implemented
belongs:
  - project-config
prerequisite:
  - project-config
  - cli-interface
  - lake-commands
  - utility-commands
refines:
  - command-families
---

Project setup commands create or repair the starting shape of a Sirno-managed repository.
They are top-level orchestration commands.
The domain operations they call keep their own command homes.

## Command Surface

| Surface | Operation | Behavior |
|---|---|---|
| CLI | `sirno init` | Opens an interactive setup flow. |
| CLI | `sirno init --all` | Runs full setup without prompts. |

Interactive init asks which setup parts to run,
asks for default paths when no path flag supplies them,
asks whether installed wrappers should be linked into Claude skills,
shows the init plan,
and applies it after confirmation.

Full setup creates a Sirno config,
ordinary seed entries in the reservoir,
and packaged skill wrappers.
The default reservoir path is `.sirno/lake` next to `Sirno.toml`.
The default misty workspace renders to `sirno-lake/`.

Setup flags:

| Flag | Behavior |
|---|---|
| `--lake PATH` | Chooses a non-default reservoir path. |
| `--no-lake` | Skips lake setup. |
| `--no-skills` | Skips packaged skill wrappers. |
| `--claude-skills` | Links `.claude/skills/sirno-*` to installed wrappers. |

The config is still written when another selected setup part needs it.
When a setup part is skipped,
its path option is not accepted.

`project-setup-commands` owns top-level project setup command spelling and behavior.
`lake-commands` owns the grouped lake initialization operation.
`utility-commands` owns skill wrapper maintenance commands outside top-level setup.

The *repository witnesses* for this entry should show CLI init grammar,
interactive and non-interactive init dispatch,
and the setup orchestration that calls the narrower command operations.
