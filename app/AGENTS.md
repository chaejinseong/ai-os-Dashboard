# Working Agreement

Entry point for any AI agent (or human) working in this repo. Read this
first, then follow the pointers below before making changes. Keep
everything in `docs/` short — these files are re-read on every task, so
bloat here costs real tokens on every run.

## Before you start

1. Read `docs/03-product-plan.md` — what this project is and its scope.
2. Read `docs/02-specs.md` — stack, exact commands, conventions, and any
   project-specific boundary overrides.
3. Read `docs/01-folder-architecture.md` — where things live and why.
4. Check `docs/todo/00-todo-list.md` for open work related to your task.
5. Skim only the last few relevant entries in `docs/reports/` for
   context — don't read the whole folder.

## Boundaries

Default rule: reversible, local-only actions are free. Anything that
touches shared state, external systems, or is hard to undo needs a check
first. `docs/02-specs.md` can add project-specific exceptions — check
there before assuming these apply unmodified.

**Always allowed, no confirmation needed**
- Reading files and exploring the codebase
- Running local tests, builds, and linters
- Creating or editing files within the task's scope
- Installing dependencies already declared in the lockfile
- Local git commits and local branches

**Ask first**
- Adding, upgrading, or removing a dependency
- Schema changes or running a migration
- Pushing to a remote/shared branch, or opening a PR
- Changing CI/CD configuration
- Deleting or bulk-moving files outside the task's scope
- Any API call that costs money or has a real external side effect
- Writing implementation code for an addition or modification before its
  plan has been approved — see "Before you implement" below

**Never, under any circumstances**
- Force-pushing or rewriting history on `main` / `master`
- Editing `.env`, secrets, or credential/auth files
- Operating directly on production data or a production database
- Deleting tests, or marking failing tests as skipped, to make a run pass

## Before you implement anything beyond a trivial fix

This applies no matter which interface you're running in (CLI, desktop
app, or anything else) — the person should see the plan before code gets
written, not just the finished diff.

- **Trivial** (a typo, a one-line log statement, a fix you could describe
  in one sentence): just make it, no plan needed.
- **Any other addition or modification** (a new feature, a behavior
  change, a non-trivial fix): stop before writing code. Post a short plan
  — what will change, which files, the approach — and wait for explicit
  approval before implementing.
- **Large or ambiguous** (spans many files, requirements aren't clear):
  write that plan as `SPEC.md` in the repo root instead of just posting it
  inline, so it's reviewable and survives a session boundary. Get it
  confirmed, implement from it — ideally in a fresh task/session so
  implementation isn't cluttered with the back-and-forth that produced the
  spec — then delete `SPEC.md` once the work lands in a report.

## Verification — don't call it done without this

- Run the relevant tests and show the actual output (pass/fail), not just
  a claim that it works.
- Confirm the project still builds / compiles cleanly.
- If you can't verify a change (no test, no build step, no way to check),
  say so explicitly instead of asserting it works.

## While you work

- Keep changes scoped to the task. Found something unrelated? Add it to
  `docs/todo/` instead of fixing it inline.
- Follow conventions already in the codebase over inventing new ones. If
  `docs/02-specs.md` doesn't cover it, ask rather than guessing.
- Prefer small, reviewable changes over large rewrites.
- Commit messages follow Conventional Commits: `feat:`, `fix:`, `chore:`,
  `refactor:`, `docs:`, `test:`.

## When you finish

1. Verified the change (see above)? Add a report to `docs/reports/` named
   `YYMMDD-HHMM-NN-short-title.md`, using `_template.md`. Keep it to a few
   bullet points — what changed, why, how it was verified.
2. Work left over (follow-ups, known issues, deferred decisions)? Add or
   update `docs/todo/YYMMDD-HHMM-NN-short-title.md` (same naming) and list
   it in `00-todo-list.md`.
3. Update `01-folder-architecture.md` or `02-specs.md` ONLY if the change
   actually altered structure, stack, or constraints. Don't pad these
   files "just in case."

## Monorepos / multiple projects in one repo

If a subfolder needs different rules, give it its own `AGENTS.md`. The
`AGENTS.md` closest to the file being edited wins over this one.

## Context hygiene (keep this cheap to read)

- Don't paste whole files into reports or todos — reference paths and line
  ranges instead.
- `00-todo-list.md` is an index only: one line per item. Full detail lives
  in the item's own file under `docs/todo/`.
- Close out or delete stale todo/report entries periodically so the index
  doesn't grow without bound.
