# Working Agreement — Product/Service Planning

Entry point for any AI agent (or human) doing planning work in this repo:
PRDs, one-pagers, requirements, roadmap decisions. Read this first. Keep
everything in `docs/` short — it's re-read on every task, so bloat here
costs tokens on every run.

## Before you start

1. Read `docs/01-context.md` — product, users, constraints. Doesn't change
   often; if it looks stale, flag that instead of guessing around it.
2. Read `docs/02-guidelines.md` — how we source data, write goals, and who
   needs to review what.
3. Check `docs/prd/00-index.md` for related or overlapping initiatives
   before starting a new one — don't plan in a vacuum.
4. Check `docs/todo/00-todo-list.md` for open questions relevant to your
   task.
5. Skim only the last few relevant `docs/reports/` entries for context on
   recent decisions — don't read the whole folder.

## Boundaries

**Always allowed, no confirmation needed**
- Researching, drafting, and revising a one-pager or PRD in `docs/prd/`
- Asking clarifying questions
- Adding/updating an item in `docs/todo/`
- Marking your own draft as "Draft" status

**Ask first**
- Sharing or sending a doc outside the immediate team
- Marking a PRD's status "Approved" or "Final" (someone with authority to
  greenlight it should confirm)
- Committing to a specific public date, number, or promise in any doc that
  might leave the team
- Changing the status or conclusion of an existing decision record instead
  of writing a new one that supersedes it

**Never, under any circumstances**
- Presenting a made-up number, quote, or data point as if it were sourced
- Presenting an assumption or guess as a validated fact
- Deleting a past decision record instead of marking it superseded
- Publishing anything externally (customers, press, public roadmap) without
  explicit sign-off

## Before you write a full PRD

This applies whether you're working from a CLI, a desktop app, or a chat —
the person should see the shape of the idea before a full PRD gets
written.

- **Trivial** (a small edit to an existing, already-approved PRD): just
  make it.
- **A new idea or initiative**: write a one-pager first
  (`docs/prd/_one-pager-template.md`) — problem, customer, why now, and the
  biggest open question — and get it confirmed before expanding it into a
  full PRD. This is the planning equivalent of "don't start coding before
  the plan is approved": a one-pager is cheap to redirect, a full PRD is
  not.
- **Large or ambiguous** (spans multiple teams, or requirements are
  genuinely unclear): treat this like any other ambiguous work — interview
  the person, write out your understanding, and get it confirmed before
  drafting the full document.

## Verification — don't call it done without this

A PRD doesn't get "tested," but it still needs to hold up. Before treating
a draft as ready for review:

- Every number or claim has a cited source (user interview, data query,
  competitor page, etc.) — or is explicitly flagged `[ASSUMPTION]` if it
  isn't.
- Every goal in "Goals & Success Metrics" is measurable — if you can't say
  how you'd know it succeeded, it's not done yet.
- Read it once as the target user would and once as a skeptical
  reviewer — does the problem read as real, and does the solution actually
  address it?

## While you work

- Keep a draft scoped to one initiative. Found a related-but-different
  idea? Add it to `docs/todo/` instead of expanding the current doc.
- Follow the terminology and format already established in
  `docs/02-guidelines.md` over inventing new structure.
- Prefer a shorter, sharper doc over a comprehensive one — "don't turn a
  PRD into a novel."

## When you finish

1. A decision got made (a direction chosen, a question resolved, a PRD
   approved or shelved)? Write a report to `docs/reports/` named
   `YYMMDD-HHMM-NN-short-title.md`, using `_template.md`. Record what was
   decided, why, and what alternatives were considered.
2. Question still open? Add or update `docs/todo/YYMMDD-HHMM-NN-short-title.md`
   and list it in `00-todo-list.md`.
3. Update `docs/prd/00-index.md` whenever a PRD's status changes.
4. Update `01-context.md` or `02-guidelines.md` ONLY if something durable
   actually changed (new constraint, new process) — not for one-off facts
   that belong in a single PRD.

## Context hygiene (keep this cheap to read)

- Don't paste full interview transcripts or raw research into a PRD —
  summarize, and link/reference the source.
- `docs/todo/00-todo-list.md` and `docs/prd/00-index.md` are indexes only:
  one line per item, full detail lives in the item's own file.
- Archive or mark superseded rather than deleting — planning history is
  often as useful as the current plan.
