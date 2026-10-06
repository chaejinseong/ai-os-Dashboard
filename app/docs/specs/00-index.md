# Spec Index

One line per spec. Full detail lives in each file. Update the status as
work moves — don't delete rows for finished or shelved specs.

Status flow: `Draft` → `Approved` (human only) → `Implemented`
(implementer) → `Reviewed` (reviewer). `Shelved` at any point.

| File | Status | Implementer | Source PRD | Summary |
|---|---|---|---|---|
| _(example) 260219-1956-01-login-token-refresh.md_ | Approved | codex | planning/docs/prd/login.md | Refresh expired token once before logging out |

## How to add a spec

1. Copy `_template.md` to `YYMMDD-HHMM-NN-short-title.md` in this folder.
2. Fill it in with status `Draft`.
3. Add a row above, and put the spec's path in the source PRD's
   `## Handoff` section.
