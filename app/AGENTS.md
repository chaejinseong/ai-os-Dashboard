# Working Agreement — App

Entry point for any AI agent (or human) working in `app/`. The workspace
root `AGENTS.md` (roles, paths, handoff) applies too — read it first.
Keep everything in `docs/` short — these files are re-read on every task,
so bloat here costs real tokens on every run.

## Before you start

1. Read `docs/03-product-plan.md` — what this project is and its scope.
2. Read `docs/02-specs.md` — stack, exact commands, conventions, and any
   project-specific boundary overrides.
3. Read `docs/01-folder-architecture.md` — where things live and why.
4. Implementing? Read the spec you were given in `docs/specs/` in full —
   it is your task definition.
5. Check `docs/todo/00-todo-list.md` for open work related to your task.
6. Skim only the last few relevant entries in `docs/reports/` for
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
- Implementing a spec whose status is `Approved`

**Ask first**
- Adding, upgrading, or removing a dependency (unless the approved spec
  names it)
- Schema changes or running a migration (unless the approved spec
  names it)
- Pushing to a remote/shared branch, or opening a PR
- Changing CI/CD configuration
- Deleting or bulk-moving files outside the task's scope
- Any API call that costs money or has a real external side effect
- Writing implementation code for a non-trivial change that has no
  approved spec — see "Before you implement" below

**Never, under any circumstances**
- Force-pushing or rewriting history on `main` / `master`
- Editing `.env`, secrets, or credential/auth files
- Operating directly on production data or a production database
- Deleting tests, or marking failing tests as skipped, to make a run pass
- Marking a spec `Approved` yourself

## Before you implement anything beyond a trivial fix

The person should see the plan before code gets written. In this
workspace the plan is a spec in `docs/specs/`.

- **Trivial** (a typo, a one-line log statement, a fix you could describe
  in one sentence): just make it, no spec needed. Still log it in a
  report if it's committed.
- **Spec is `Approved`**: that *is* the approval. Implement it without
  re-posting a plan. If you find you must deviate from it (a file it
  didn't list, a different approach), stop, add a `docs/todo/` item
  explaining why, and wait.
- **No spec yet** (a new feature, a behavior change, a non-trivial fix):
  don't write code. Draft a spec from `docs/specs/_template.md` with
  status `Draft`, add it to `docs/specs/00-index.md`, and wait for the
  human to approve it. Implement in a fresh session so the work isn't
  cluttered by the back-and-forth that produced the spec.

## Verification — don't call it done without this

- Run every command listed under the spec's "Done when" and show the
  actual output (pass/fail), not just a claim that it works.
- Confirm the project still builds / compiles cleanly.
- If you can't verify a change (no test, no build step, no way to check),
  say so explicitly instead of asserting it works.

## While you work

- Keep changes scoped to the spec. Found something unrelated? Add it to
  `docs/todo/` instead of fixing it inline.
- Follow conventions already in the codebase over inventing new ones. If
  `docs/02-specs.md` doesn't cover it, ask rather than guessing.
- Prefer small, reviewable commits over large rewrites.
- Commit messages: Conventional Commits with `app` scope (`feat(app):`,
  `fix(app):`, `refactor(app):`, `test(app):`, `docs(app):`), report path
  on the last line of the body.

## When you finish

1. Verified the change? Write a report in `docs/reports/` named
   `YYMMDD-HHMM-NN-short-title.md` from `_template.md` — see "Work log"
   below.
2. Set the spec's status to `Implemented` and update
   `docs/specs/00-index.md`.
3. Work left over (follow-ups, known issues, deferred decisions)? Add or
   update `docs/todo/YYMMDD-HHMM-NN-short-title.md` (same naming) and list
   it in `00-todo-list.md`.
4. Update `01-folder-architecture.md` or `02-specs.md` ONLY if the change
   actually altered structure, stack, or constraints. Don't pad these
   files "just in case."

## Work log

The report is what git can't record: why each file changed and how.

- One report per commit, or per small group of commits for one spec.
- For every changed file, three lines: **Where** (function / component /
  section name), **Purpose** (why this file had to change), **Method**
  (what approach, and what you rejected if it matters).
- Locate changes by symbol name, never by line number — line numbers go
  stale on the next edit.
- Don't paste code. Reference commit hashes; `git show <hash>` has the
  rest.
- Files changed only by formatting, imports, or generated output: list
  them together on one line.

## Reviewing

Default reviewer is Claude (see the root `AGENTS.md`).

1. Read the spec, then the report — not the raw diff first.
2. Compare each file's stated Purpose/Method with `git diff` for the
   listed commits. A mismatch between what the report says and what the
   code does is the first thing to flag.
3. Check the spec's "Done when" items and re-run its commands.
4. Write findings in the report's `## Review` section, each tagged
   `must-fix` or `suggestion`.
5. The implementer fixes `must-fix` items and notes each as resolved.
   After two review rounds with `must-fix` items still open, stop and hand
   it to the human instead of looping — it saves usage and usually means
   the spec was unclear.
6. When nothing `must-fix` is open, set the spec to `Reviewed`.

## Monorepos / multiple projects in one repo

If a subfolder needs different rules, give it its own `AGENTS.md`. The
`AGENTS.md` closest to the file being edited wins over this one.

## Context hygiene (keep this cheap to read)

- Don't paste whole files into reports, specs or todos — reference paths
  and symbol names instead.
- `00-todo-list.md` and `specs/00-index.md` are indexes only: one line
  per item. Full detail lives in the item's own file.
- Close out stale todo entries periodically so the index doesn't grow
  without bound. Reports stay — they're the history.
