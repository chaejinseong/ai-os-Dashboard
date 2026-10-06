# <feature / initiative name>

- Status: Draft | In Review | Approved | Shipped | Shelved
- Created: YYYY-MM-DD
- Owner:
- One-pager (if any): planning/docs/prd/<file>.md

## Overview / problem statement

Why are we building this? What's the context a reader needs before
anything else makes sense.

## Goals & success metrics

Measurable outcomes, not activities. Include how you'd actually check
each one (a dashboard, a survey, a specific number).

## Non-goals

What this explicitly does NOT try to solve — as important as the goals,
since it's what keeps the doc (and the eventual build) from scope-creeping.

## User stories / use cases

Concrete scenarios, from the user's perspective.

## Functional requirements

What the system/feature must do. Numbered, so they're easy to reference
and check off later.

## Non-functional requirements

Performance, security, accessibility, usability standards that apply.

## Design considerations

UI/UX expectations, links to mockups/prototypes if they exist.

## Technical considerations

Known backend/integration constraints — not a full spec, just what a
planner needs to know before committing to scope. (The implementation
detail belongs in an engineering spec, not here.)

## Timeline & milestones

Key dates and dependencies, if known. Don't invent false precision.

## Open questions & risks

What's still unresolved. Move anything that needs someone else's input
into `docs/todo/` and link it here.

## Appendix

Supporting research, links, data behind the claims above.

## Handoff

Filled in once this PRD is `Approved` and work moves to the app.

- Spec(s): app/docs/specs/<file>.md
- Status of each: (mirror `app/docs/specs/00-index.md`)
- Anything the spec intentionally left out of this PRD, and why:
