# <short-title>

- Status: Draft | Approved | Implemented | Reviewed | Shelved
- Created: YYYY-MM-DD
- Author: claude | <name>
- Implementer: codex | claude | <name>
- Source PRD: planning/docs/prd/<file>.md (or "none — <why>")
- Reports: app/docs/reports/<file>.md (filled in by the implementer)

> Only the human sets `Approved`. Once approved, the implementer builds
> this without asking for plan approval again — so write it so nobody
> has to guess.

## Goal

One or two sentences: what will be true when this is done, in user or
system terms.

## Out of scope

What this spec deliberately does NOT do, so the implementer doesn't
drift into it.

## Files

| File | Change | What |
|---|---|---|
| `src/...` | modify | ... |
| `src/...` | new | ... |
| `tests/...` | new | ... |

Anything not listed here is off-limits without stopping to ask.

## Approach

How to do it, in enough detail that two people would build roughly the
same thing. Name the functions/components involved. If there's an
obvious alternative you're ruling out, say which and why.

## Don't touch

- (files, APIs, behaviors that must not change)

## Done when

- [ ] `<exact test command from app/docs/02-specs.md>` passes
- [ ] `<build / lint command>` passes
- [ ] (any behavior to check by hand, stated concretely)

## Open questions

Anything still unresolved. If one blocks implementation, the spec
shouldn't be `Approved` yet.

---
File naming: `YYMMDD-HHMM-NN-short-title.md`, same as todo/ and reports/.
