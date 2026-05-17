---
desc: A development style that keeps implementation aligned with named purpose.
lifecycle: Active
name: Intent-Driven Development
structural:
  category:
  - concept
  belongs:
  - development-style
  refines:
  - concept-driven-development
---

Intent-driven development starts from the reason for the change.

Intent names the purpose that should survive implementation choices.
It is broader than a test and often softer than a specification,
but it is still actionable.
It says which user need, design pressure, invariant, simplification,
or future review question the work is meant to serve.

In concept-driven development,
intent becomes durable when it is attached to a concept.
The concept keeps the intent from dissolving into a private hunch.
The intent keeps the concept from becoming a detached label.

Intent-driven work should be reflected when code teaches a clearer purpose.
A rename, abstraction, or test may be small,
but the reason behind it can be the design fact future work needs.
The *entry* should keep that reason available without forcing readers to reconstruct it from commits.
