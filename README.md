# product-workspace

One folder for one product: planning docs and the app live side by side,
so a PRD can point at the spec that implements it and a spec can point
back at the PRD it came from. Built from two templates —
`planning-docs-template` (now `planning/`) and `docs-template` (now
`app/`) — plus the rules for splitting work between Claude Code and Codex.

## Layout

```
README.md                  this file — how the workspace fits together
AGENTS.md                  shared rules every agent reads first (roles, paths, handoff)
CLAUDE.md                  Claude Code entry point (imports AGENTS.md + Claude-only notes)
planning/                  product planning: one-pagers, PRDs, decisions
  AGENTS.md / CLAUDE.md
  docs/
    01-context.md          product, users, constraints
    02-guidelines.md       sourcing, writing rules, review flow
    prd/                   one-pagers + PRDs (+ Handoff to app/docs/specs/)
    todo/                  open questions
    reports/               decision records
app/                       the code + its working docs
  AGENTS.md / CLAUDE.md
  docs/
    01-folder-architecture.md
    02-specs.md            stack, exact commands, conventions
    03-product-plan.md     purpose, scope (links to source PRDs)
    specs/                 implementation specs — the Claude → Codex handoff
    todo/                  open/deferred work
    reports/               work log: per-file "where / purpose / method"
  (your source code goes here: src/, tests/, package.json, ...)
```

## Who does what

| Stage | Where | Who writes | Who approves |
|---|---|---|---|
| One-pager → PRD | `planning/docs/prd/` | Claude | you |
| Implementation spec | `app/docs/specs/` | Claude | you (sets `Approved`) |
| Code + tests + report | `app/` + `app/docs/reports/` | Codex | — |
| Review | the report's `## Review` section | Claude | you (merge) |

Why this split: implementation loops (read files, edit, run tests, retry)
burn the most usage, so they go to Codex. Planning, spec-writing and
review need judgment but few tokens, so they stay with Claude. Swap roles
per task if a piece of code is delicate enough that you'd rather Claude
write it — just set `Implementer:` in the spec.

## The flow

```
planning/docs/prd/        one-pager → PRD            (Claude, you approve)
        │  PRD "Handoff" section
        ▼
app/docs/specs/           spec: files, approach, done criteria
                          Draft → Approved (you)
        ▼
app/                      Codex implements + runs tests
app/docs/reports/         per-file log; spec → Implemented
        ▼
review                    Claude compares report vs. git diff vs. spec,
                          writes findings; Codex fixes; spec → Reviewed
```

## How to start a new product

1. Use this repo as a GitHub template (or clone it) — one copy per product.
2. Fill in `planning/docs/01-context.md` and `02-guidelines.md`.
3. Fill in `app/docs/02-specs.md` — especially `## Commands`; no agent can
   guess these.
4. Where to open each tool:
   - Planning work → open Claude Code in `planning/` (or the root).
   - Spec writing / review → open Claude Code at the root, so it can see
     both `planning/` and `app/`.
   - Implementation → open Codex in `app/`.

Both tools pick up the nearest `AGENTS.md` / `CLAUDE.md` plus the ones
above it, so the root rules always apply and folder rules add to them.

## Optional: Codex review from inside Claude Code

OpenAI's Codex plugin for Claude Code (`openai/codex-plugin-cc`) adds
`/codex:review` and `/codex:adversarial-review`. Useful for a second
opinion on a PRD or before shipping. Leave its automatic "review gate"
off on a Pro plan — it can loop and drain usage quickly.

## Updating the templates

If a rule keeps getting re-added in real products, fix it here so every
future workspace gets it.
