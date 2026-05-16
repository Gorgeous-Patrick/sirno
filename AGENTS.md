# Repository Guidelines

> `CLAUDE.md` is a symlink to this file. Do not edit `CLAUDE.md` directly -- edit `AGENTS.md` instead.


## Core Principles

Prioritize a clean codebase and elegant design over compatibility or migration work.
Do not keep transitional layers, compatibility shims, or legacy interfaces unless the task explicitly requires them.
If compatibility or migration concerns conflict with a clearer design, prefer the clearer design.

## Documentation and Language

All written documentation must be concise, clear, accurate, and written in English unless explicitly stated.
Keep sentences short. The line budget is 120 characters.
Break Markdown prose at natural punctuation boundaries or conjunctions.
A line may slightly exceed the line-length budget when that makes the text read better.
No emojis unless strictly necessary.
Add bold text only if it emphasizes truly valuable information.
Prefer direct definitions over defensive framing.
- Define what the system does before explaining limits or exclusions.
- Keep definition-by-negation to a minimum;
  use it only when a nearby confusion is likely and the contrast is genuinely clarifying.
- Avoid prose that reads like a rebuttal, disclaimer, or argument with an imaginary reviewer.
- When documenting a constraint, state the positive rule first, then the consequence if needed.

### Sirno Lake

The authoritative project design lives in `sirno-docs/`.
The split Chinese translation snapshot lives in `sirno-docs-zh/`.
Do not update `sirno-docs-zh/` during normal agent workflow or lake maintenance.
Whenever anything changes about the project design, keep the Sirno Lake in sync.
Use `sirno-editor` when editing, moving, or reorganizing lake knowledge.
Use `sirno-explorer` before design-sensitive repository exploration.
Use `sirno-witness` when adding or refining repository witness blocks.
Use `sirno-skill-synthesizer` after changing meta discipline entries or skill packages.
If more details are needed, the Sirno skill roster is documented in `sirno-docs/agent-skills.md`.
After lake metadata changes, run generated-link maintenance and `sirno check`.
Use `sirno-docs/introduction.md` as the first narrative route.
Use `sirno-docs/methodology.md` as the working guide.

### Rust

When editing Rust code or inline Rust documentation, use the `rust-programmer` skill.
The skill carries the detailed Rust standards.

## Version Control

This project uses git. Use git to operate.

### Commit Message Convention

Format: `prefix: lowercase description`

No capitalization after the colon. No trailing period. One line.
The description should say *what changed*, not *why* (the diff shows what; the description names it).

#### Prefix Vocabulary

| Prefix | When to use |
|--------|-------------|
| `feat`  | A user-visible capability that did not exist before. |
| `incr`  | Incremental progress on an existing feature: bug fixes, polish, tuning, small additions. |
| `sisy`  | Mechanical changes: formatting, linting, renaming passes, internal restructuring with no behavior change. |
| `vibe`  | Exploratory, prototype-quality work. Expect rough edges; may be revised or replaced. |
| `repo`  | Repository housekeeping: migrations, dependency changes, formatter config, file reorganization, one-off maintenance. |
| `docs`  | Documentation-only changes (AGENTS.md, README, inline Rust docs/comments). |
| `test`  | Adding or updating tests without changing production code. |

#### Guidelines

- One logical change per commit.
  If two things can be reverted independently, they are two commits.
- Pair implementation files with their tests in the same commit.
- Order commits by dependency level: types and utilities first, then logic, then UI, then config.
- Prefer many small commits over one large commit.
  Rule of thumb: a reviewer should understand a commit in under 30 seconds.
