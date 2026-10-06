# Workspace Agreement

Read first, by every agent, in every folder. Folder-level `AGENTS.md`
files (`planning/`, `app/`) add to these rules; when they conflict, the
one closest to the file being edited wins. Keep this file short — it is
loaded on every task.

## Map

- `planning/` — what to build and why (one-pagers, PRDs, decisions).
  Rules: `planning/AGENTS.md`.
- `app/` — the code and how it was built (specs, work log).
  Rules: `app/AGENTS.md`.

## Roles

- **Claude**: planning docs, implementation specs, code review.
- **Codex**: implementing approved specs, running tests, writing the work
  report.
- **The human**: approves one-pagers, PRDs and specs; merges.
- A spec's `Implementer:` field overrides the default for that spec.

## Paths

- Every path written inside a doc is relative to this workspace root
  (`planning/docs/prd/login.md`, `app/docs/specs/...`), never `../`-style,
  so links read the same from any folder.

## Handoff points

1. PRD `Approved` → Claude drafts a spec in `app/docs/specs/` and fills
   the PRD's `## Handoff` section with its path.
2. Spec `Approved` (set by the human only) → the implementer builds it
   without asking for plan approval again.
3. Spec `Implemented` → Claude reviews; spec becomes `Reviewed` once
   findings are resolved.

## Shared never-list

- Never mark your own PRD or spec `Approved`.
- Never edit `.env`, secrets, or credential files.
- Never delete a past report or decision — mark it superseded.

## Commits

One git repo for the whole workspace. Conventional Commits with the
folder as scope: `feat(app): ...`, `fix(app): ...`, `docs(planning): ...`,
`docs(app): ...`. Put the report path on the last line of the body.
