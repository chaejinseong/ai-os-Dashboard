# planning-docs-template

The planning-side sibling of `docs-template` (the dev version): a reusable
`docs/` architecture + `AGENTS.md` working agreement for AI-assisted
product/service planning — PRDs, one-pagers, requirement docs. Keep this
as its own GitHub repo and pull it into a new planning workspace before
you start.

## What's inside

```
AGENTS.md                        entry point — boundaries, verification, workflow rules
docs/
  01-context.md                  product, users, constraints (static reference)
  02-guidelines.md                data sourcing, writing conventions, review flow
  prd/
    00-index.md                   index of all initiatives + status
    _one-pager-template.md        cheap early-validation doc (PR/FAQ-style)
    _template.md                  full PRD (10-section format)
  todo/
    00-todo-list.md               index of open questions
    _template.md                  format for a new open question
  reports/
    _template.md                  format for a decision record
```

## Where this structure comes from

Rather than invent a planning format from scratch, this borrows from
patterns already proven in the industry:

- **The one-pager** is a lightweight take on Amazon's "Working Backwards"
  PR/FAQ process — write the customer-facing outcome and the biggest risk
  *before* investing in a full PRD, so a bad idea is cheap to redirect.
  See [workingbackwards.com](https://workingbackwards.com/resources/working-backwards-pr-faq/).
- **The full PRD template** follows the widely-used modern structure
  (overview, goals & metrics, user stories, functional/non-functional
  requirements, design & technical considerations, timeline, open
  questions, appendix) — see
  [ChatPRD's PRD guide](https://www.chatprd.ai/learn/prd-template).
- **Status tracking** (`Draft → In Review → Approved → Shipped/Shelved`)
  and the emphasis on **goals & non-goals** + **alternatives considered**
  in decision records come from how engineering RFCs/design docs are run
  at companies like Google, Uber, and Stripe — see
  [Pragmatic Engineer's survey of RFC/design doc templates](https://blog.pragmaticengineer.com/rfcs-and-design-docs/).
- **Keeping `AGENTS.md` thin** and pushing anything occasional into its
  own doc (rather than growing the always-loaded file) mirrors how AI
  coding agents are told to handle skills vs. always-on context — see
  [Factory's product-management agent skill docs](https://docs.factory.ai/guides/skills/product-management).

## How this differs from the dev `docs-template`

Same skeleton (`AGENTS.md` + `docs/` + `todo/` + `reports/`), different
content, because "done" means something different:

| | Dev version | Planning version |
|---|---|---|
| "Plan before you build" | Plan → approval → code | One-pager → approval → full PRD |
| "Verification" | Tests pass, build succeeds | Every claim is sourced, every goal is measurable |
| "Ask first" examples | New dependency, DB migration, remote push | Sharing outside the team, marking a PRD "Approved" |
| "Never" examples | Force-push to main, touch `.env` | State a guess as fact, delete a past decision |

## How to use it in a new planning workspace

```bash
git clone https://github.com/<you>/planning-docs-template.git /tmp/pt
cp -r /tmp/pt/AGENTS.md /tmp/pt/docs ./
```

Or mark this repo as a GitHub "template repository" and click "Use this
template" for each new planning workspace.

Then:

1. Fill in `docs/01-context.md` and `docs/02-guidelines.md` once for the
   team/product.
2. For each new idea, start with `docs/prd/_one-pager-template.md`, not
   the full PRD — see `AGENTS.md`'s "Before you write a full PRD."
3. Point your AI tool at `AGENTS.md` as its entry point.

## Updating this template

If something should apply to every future planning workspace (a sourcing
rule you keep re-adding, a PRD section every initiative ends up needing),
fix it here and re-pull it — don't just patch one workspace and forget the
source.
