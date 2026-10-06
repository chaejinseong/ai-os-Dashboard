# app/

> Part of a product workspace — see the root `README.md` for how
> `planning/` hands off to this folder and how Claude and Codex split the
> work.

A reusable `docs/` architecture + `AGENTS.md` working agreement for
AI-assisted projects (works solo or with a team, and across tools —
`AGENTS.md` is an open cross-tool standard read by Claude, Cursor, Copilot,
and others: see [agents.md](https://agents.md/)). Keep this as its own
GitHub repo and pull it into any new project before you start.

## What's inside

```
AGENTS.md                        entry point — boundaries, verification, workflow rules
docs/
  01-folder-architecture.md      where things live and why
  02-specs.md                    stack, exact commands, code style, constraints
  03-product-plan.md             purpose, scope, requirements (links source PRDs)
  specs/
    00-index.md                  index of implementation specs + status
    _template.md                 format for a spec (the Claude → Codex handoff)
  todo/
    00-todo-list.md              index of open/deferred work
    _template.md                 format for a new todo item
  reports/
    _template.md                 work log: per-file where / purpose / method + review
```

- `01`–`03` are the fixed reference docs for a project: fill them in once,
  update only when they actually go stale.
- `todo/` and `reports/` are the working log: `todo/` is what's left to do,
  `reports/` is what's already been done, why, and how it was verified.
- `AGENTS.md` is the only file loaded on every single task — everything in
  it earns its place. If a rule only matters sometimes, it belongs in
  `docs/02-specs.md` or a project-specific note, not `AGENTS.md`.

## How to use it

1. Fill in `docs/03-product-plan.md`, `docs/02-specs.md` (especially the
   `## Commands` section — an agent can't guess these), and
   `docs/01-folder-architecture.md`.
2. Put your source code in this folder (`src/`, `tests/`, ...).
3. Work flows through `docs/specs/` → code → `docs/reports/` — see
   `AGENTS.md`.

## Defaults baked into this template

These are starting defaults, not laws — adjust per project in
`docs/02-specs.md`'s "Project-specific boundary overrides" section rather
than editing `AGENTS.md` itself, so the base template stays reusable.

- **Boundaries**: a 3-tier model (always allowed / ask first / never) —
  see `AGENTS.md`. Reversible, local-only actions are free; anything
  touching shared state or hard to undo needs a check first.
- **Verification**: an agent must run tests and confirm the build/compile
  before calling anything done — not just assert it works.
- **Commit style**: [Conventional Commits](https://www.conventionalcommits.org/)
  (`feat:`, `fix:`, `chore:`, ...).
- **File naming**: `YYMMDD-HHMM-NN-short-title.md` for both `todo/` and
  `reports/` entries — always timestamped, even solo, so both folders sort
  chronologically and stay unambiguous if you ever run parallel agents.
- **Plan before code**: anything beyond a trivial fix needs a spec in
  `docs/specs/` that the human marks `Approved`. An approved spec is the
  go-ahead — the implementer doesn't ask again. Specs are kept, not
  deleted, so reviews can check the code against them.
- **Work log**: every report records, per changed file, where it changed
  (by symbol name), why, and how — the things git can't record.

## Monorepos

For a repo with multiple independent projects/packages, give each one its
own `AGENTS.md` in its subfolder. The closest `AGENTS.md` to the file being
edited wins over the root one — see `AGENTS.md`'s "Monorepos" section.

## Updating this template

If you learn something that should apply to every future project (a rule
that keeps needing to be re-added to `AGENTS.md`, a section every
`02-specs.md` ends up needing), fix it here and re-pull it into active
projects — don't just patch it in one project and forget the source.
