---
name: sirno-bootstrap
description: >-
  Install Sirno and register its stdio MCP server in the current repository, idempotently and
  revertably. Use to set up Sirno from a fresh clone before any other Sirno skill or MCP tool,
  or to repair a missing binary or missing MCP registration.
---

# Sirno Bootstrap

## Purpose

Bring a repository from "Sirno not available" to "Sirno installed and reachable over MCP".
This is the entry point the README TL;DR names as `$sirno-bootstrap`.
After it succeeds, hand off to `$sirno-narrative-session` or any other Sirno skill.

This skill is self-contained on purpose.
It runs before the Sirno MCP server exists, because it is what registers that server.
So unlike the six synthesized Sirno skills,
it does not defer to an `sirno://skills/*` MCP resource and must not call Sirno MCP tools.
It is hand-authored and deliberately outside the `sirno util skills` synthesized roster,
so the skill synthesizer neither generates nor manages it.

Use the shell directly for every step.
Do not edit lake content here.

## Idempotency

Every step is conditional.
"if not already done" means: detect the current state first, and make the step a no-op when it already holds.
Never reinitialize an existing lake.
Never add a duplicate MCP registration.
Re-running this skill on a ready repository must change nothing.

## Procedure

### 1. Confirm location

Confirm the working directory is the intended Sirno target.
A Sirno-managed project has a `Sirno.toml` at its root.
The Sirno source repository has a `Cargo.toml` whose `[package] name = "sirno"`.
If neither is present, stop and report; do not initialize a lake in an unrelated directory.

### 2. Install the `sirno` binary

Skip if `sirno --version` already runs.

Otherwise choose by context:

- Inside the Sirno source repository, install from local source:
  `cargo install --path . --locked`.
  This is revertable with `cargo uninstall sirno`.
- For a project that only consumes Sirno, install the published crate:
  `cargo install sirno --locked`.

Lighter, fully revertable alternative when nothing should be installed globally:
use `cargo run --` (or `target/debug/sirno` after one `cargo build`) wherever this skill says `sirno`.
Nothing is placed on `PATH`, so there is nothing to uninstall.

### 3. Initialize the lake

Skip if `Sirno.toml` already exists (the Sirno source repository is already initialized).

Otherwise run `sirno init` in the project root.
It creates the lake, the frost store, `Sirno.toml`, `Sirno.lock.toml`,
and the packaged skill wrappers together.

### 4. Register the MCP server

Detect the active agent and register the stdio server.
Check the existing registration list first and skip if `sirno` is already present.

- Codex: `codex mcp list`, then `codex mcp add sirno -- sirno util mcp`.
- Claude Code: `claude mcp list`, then `claude mcp add sirno -- sirno util mcp`.
- An agent that reads an MCP config file directly: add an equivalent stdio server.

  ```json
  {
    "mcpServers": {
      "sirno": {
        "command": "sirno",
        "args": ["util", "mcp"]
      }
    }
  }
  ```

A newly registered MCP server usually appears only after the agent reloads its MCP connections.
Tell the user to restart or reload the session if the Sirno tools are not yet visible.

### Project resolution

`sirno util mcp` is registered without `--config`,
so the server uses the default `Sirno.toml` path resolved from its process working directory,
and resolves it again on every project tool call.
After the agent reloads, the first Sirno step must bind the project:
call the `sirno_cwd` tool with the repository root before any other Sirno project tool,
and again before switching projects in the same server process.
For deterministic resolution instead, register with an absolute config:
`... -- sirno util mcp --config /abs/path/Sirno.toml`.

### 5. Verify

Confirm the install and the project resolve:

```sh
sirno --version
sirno status
sirno check --mode review
```

`sirno status` should report the project and frost state.
`sirno check --mode review` should end with `ok: <lake-path>`.
After the agent reloads, the Sirno MCP tools and `sirno://skills/*` resources should be reachable.

## Reverting

State the revert path when reporting completion:

- Remove the MCP registration: `codex mcp remove sirno` or `claude mcp remove sirno`,
  or delete the stdio entry from the MCP config file.
- Uninstall the binary: `cargo uninstall sirno`.
  Nothing to uninstall if the `cargo run --` path was used.
- `sirno init` artifacts (`Sirno.toml`, `Sirno.lock.toml`, lake, frost, wrappers)
  are only created when step 3 ran; they are normal version-controlled files to revert with git.

## Reporting

Report which steps ran and which were skipped as already satisfied.
Name the verification output.
End with the revert path and the suggested next skill, normally `$sirno-narrative-session`.
