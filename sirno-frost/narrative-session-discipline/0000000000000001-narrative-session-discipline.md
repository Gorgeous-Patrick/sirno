---
desc: The agent procedure for conducting and materializing a narrative session.
lifecycle: Active
name: Narrative Session Discipline
structural:
  category:
  - meta
  belongs:
  - agent-skills
---

A narrative session builds an adaptive route through lake knowledge for one reader or task.

Read the route sources first.
Read `Sirno.toml` for the lake path,
then the lake's narrative, introduction, and methodology entries,
and any entries the user named or the task implies.

Maintain a small private session state.
It holds the reader and task,
the design pressure that makes the route useful,
the pull or tension that makes the next concept worth meeting,
known terms and missing prerequisites,
the ordered entry route,
user feedback and corrections,
deferred detail,
the aftertaste phrase or handle,
and the intended narrative entry id when materializing.

Loop in short segments.
Explain the next concept or route choice,
ask for feedback only when the answer changes the next step,
revise the route when the user shows confusion, boredom, urgency, or a sharper goal,
and name what moved earlier, what moved later, and why.
Prefer questions that unlock better sequencing over questions that only feel interactive;
when the next step is clear and the user wants momentum, continue and state the assumption.

Design the route so accurate concepts arrive at the right time.
Show tension before explanation,
give a clean first bite before the full model,
add texture through example, contrast, and consequence,
keep the sequence tight,
honor the reader's agency,
and leave an aftertaste the reader can reuse.

Materialize when the route should guide future onboarding or review.
The artifact is a narrative entry built by the serializer contract; see `narrative-serializer`.
After materializing, run render maintenance and the editing-mode structural check.
Finish by naming the artifact, the entry path, and the main sequencing choice,
and confirm the route preserves pull, a clean first bite, and an aftertaste.
