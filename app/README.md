# docs-template

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
  03-product-plan.md             purpose, scope, requirements
  todo/
    00-todo-list.md              index of open/deferred work
    _template.md                 format for a new todo item
  reports/
    _template.md                 format for a completed-work report
```

- `01`–`03` are the fixed reference docs for a project: fill them in once,
  update only when they actually go stale.
- `todo/` and `reports/` are the working log: `todo/` is what's left to do,
  `reports/` is what's already been done, why, and how it was verified.
- `AGENTS.md` is the only file loaded on every single task — everything in
  it earns its place. If a rule only matters sometimes, it belongs in
  `docs/02-specs.md` or a project-specific note, not `AGENTS.md`.

## How to use it in a new project

Option A — copy in manually:

```bash
git clone https://github.com/<you>/docs-template.git /tmp/docs-template
cp -r /tmp/docs-template/AGENTS.md /tmp/docs-template/docs ./
```

Option B — mark this repo as a GitHub "template repository" (Settings →
Template repository), then click "Use this template" when starting a new
project repo on GitHub, and delete anything you don't need.

Then:

1. Fill in `docs/03-product-plan.md`, `docs/02-specs.md` (especially the
   `## Commands` section — an agent can't guess these), and
   `docs/01-folder-architecture.md` for the new project.
2. Point your AI tool at `AGENTS.md` as its entry point, or just tell it
   "read AGENTS.md first." Most current tools (Claude Code, Cursor,
   Copilot, Codex, Windsurf, ...) pick this filename up automatically.
3. As work happens, add todo items and reports using the `_template.md`
   files — don't freehand a different format, it breaks the index.

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
- **Plan before code**: for anything beyond a one-line trivial fix, the
  agent posts a short plan and waits for approval before implementing —
  same rule whether you're driving it from a CLI or a desktop app. For
  large or ambiguous work, that plan becomes a throwaway `SPEC.md` instead
  of just a chat message. See `AGENTS.md`'s "Before you implement" section.

## Monorepos

For a repo with multiple independent projects/packages, give each one its
own `AGENTS.md` in its subfolder. The closest `AGENTS.md` to the file being
edited wins over the root one — see `AGENTS.md`'s "Monorepos" section.

## Updating this template

If you learn something that should apply to every future project (a rule
that keeps needing to be re-added to `AGENTS.md`, a section every
`02-specs.md` ends up needing), fix it here and re-pull it into active
projects — don't just patch it in one project and forget the source.
