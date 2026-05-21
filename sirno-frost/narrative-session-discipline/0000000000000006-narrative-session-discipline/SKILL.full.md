---
name: sirno-narrative-session
description: >-
  Conduct adaptive Sirno narrative sessions in the active repository and materialize the route as a
  narrative entry when the route should persist.
---

# Sirno Narrative Session

## Purpose

Use this skill to guide a reader through project knowledge in a Sirno-managed repository.
A session can be a temporary explanation,
or it can become a compact narrative entry in the active project's configured lake.

Treat entries as the durable source of knowledge.
A narrative chooses sequence, prerequisites, pressure, and deferral.
It points to entries instead of copying the whole lake.

This full skill text is served as `sirno://skills/sirno-narrative-session`.
It follows the project's narrative-session discipline.

## Project Binding

Bind the MCP server to the repository before calling project tools.
Call `sirno_cwd` with the repository root when the server may not already be there.
Project tools resolve `Sirno.toml` from the server current working directory on each call.
Call `sirno_cwd` again before switching projects in the same server process.

## Source Reading

Read `Sirno.toml` for the configured lake path.
Then read the active project entries that can ground the route:
entries the user named,
entries the task implies,
and likely front-door entries found by `sirno_entry_query`.
Do not assume any standard entry id exists.
Good search terms include the user's domain words,
plus local terms such as `introduction`, `methodology`, `narrative`, or `onboarding`
when the project carries those concepts.

If a source entry is missing,
state the gap and continue only with the route that existing entries can ground.
Hand off to `sirno-editor` if the session reveals a needed repository,
configuration, witness, or lake maintenance edit.

## Reader Pull

Make knowledge feel worth moving toward before making it complete.
The pull may be practical, aesthetic, playful, urgent, elegant, relieving, or clarifying.
Use the pull that genuinely helps this reader understand the project.

Useful moves:

- Pull before explanation: show the tension before giving the name.
- Clean first bite: give the smallest useful version before the full model.
- Texture: mix definition, example, contrast, consequence, and a good name.
- Sequence: reveal the next useful part, then let one idea unlock the next.
- Agency: ask what the reader is trying to do, then route knowledge toward that action.
- Aftertaste: leave a phrase, handle, or entry id the reader can reuse later.

Use the moves that make the next concept arrive at the right time.

## Session Workflow

Start by naming the session frame in one or two sentences.
State the likely route goal and the current uncertainty.

Ask targeted questions only when the answer changes the route.
Prefer one question at a time.
Good questions identify the reader's task, prior vocabulary, desired depth, confusion point,
or preferred artifact shape.

Maintain a small private session state:

- reader and task
- design pressure that makes the route useful
- pull or tension that makes the next concept worth meeting
- known terms and missing prerequisites
- ordered entry route
- user feedback and corrections
- deferred details
- aftertaste phrase, handle, or entry id
- intended narrative entry id, if materializing

Loop in short segments:

1. Explain the next concept or route choice.
2. Ask for feedback, confirmation, or a concrete input when it affects the next step.
3. Revise the route when the user shows confusion, urgency, or a sharper goal.
4. Name what moved earlier, what moved later, and why.

When the user wants momentum and the next step is clear,
continue and state the assumption.

## Materializing The Narrative

Materialize a narrative entry when the user requests a saved route,
when the route will guide future onboarding or review,
or when the session produces a reusable way through a design region.

Choose a lowercase kebab-case id.
Use structural metadata that the active project already configures.
If the project has no narrative category or equivalent convention,
use the simplest valid metadata and explain the choice.

The entry body should state:

- who the route serves
- why the route matters
- useful prerequisites
- the ordered route through entries
- what detail is intentionally deferred
- user feedback that changed the route, if durable
- the phrase, handle, or entry id the reader should carry forward

Keep the body short enough to read in place.
Point to entries that carry durable detail instead of copying their contents.
Do not include private chat transcript unless the user explicitly asks.

Use the serializer shipped beside this skill when its input contract fits.
Pass the configured lake path explicitly:

```sh
python3 .agents/skills/sirno-narrative-session/scripts/serialize_narrative_entry.py \
  --lake <configured-lake-path> \
  --input session.json
```

After changing lake metadata, run render maintenance and structural checks:

```text
sirno_lake_render
sirno_lake_check mode=edit
sirno_lake_check mode=review
```

If the serializer script is unavailable or its input contract does not fit the session,
draft the entry manually from the same recorded route state and report that fallback.
If the user wanted only an ephemeral explanation,
do not create a lake entry.

## Handoff

Finish by naming the route artifact, the entry path, and the main sequencing choice.
Mention any assumption that shaped the route.
Say whether checks were run.
