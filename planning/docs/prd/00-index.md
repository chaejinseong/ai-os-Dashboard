# PRD / Initiative Index

One line per initiative. Full detail lives in each file. Update the status
column as things move — don't delete a row when something ships or gets
shelved, just update its status so the history stays visible.

| File | Status | Spec(s) | Summary |
|---|---|---|---|
| _(example) signup-onboarding-v2.md_ | Approved | app/docs/specs/260219-1956-01-onboarding-v2.md | Redesign onboarding to cut drop-off |
| ai-os-dashboard-one-pager.md | Approved | — | 메일·일정·프로젝트를 모아 놓친 업무를 알려주는 macOS 앱 (원페이저) |
| ai-os-dashboard.md | Approved | app/docs/specs/ 의 261006-* 5개 (M1, M2a~c, M3) | v1 PRD: Gmail·Calendar·Notion·GitHub, Claude API 판정, Tauri |

## How to add an initiative

1. Start with `_one-pager-template.md` for anything new (see `AGENTS.md`'s
   "Before you write a full PRD").
2. Once approved, copy `_template.md` to `<kebab-case-name>.md` in this
   folder and expand it.
3. Add a row above; keep it updated as status changes.
4. Once approved, a spec gets drafted in `app/docs/specs/` — put its path
   in the "Spec(s)" column and the PRD's `## Handoff` section.
