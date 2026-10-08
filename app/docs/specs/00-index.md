# Spec Index

One line per spec. Full detail lives in each file. Update the status as
work moves — don't delete rows for finished or shelved specs.

Status flow: `Draft` → `Approved` (human only) → `Implemented`
(implementer) → `Reviewed` (reviewer). `Shelved` at any point.

| File | Status | Implementer | Source PRD | Summary |
|---|---|---|---|---|
| _(example) 260219-1956-01-login-token-refresh.md_ | Approved | codex | planning/docs/prd/login.md | Refresh expired token once before logging out |
| 261006-1535-01-m1-app-skeleton-gmail.md | Reviewed | codex | planning/docs/prd/ai-os-dashboard.md | M1: 화면 틀(디자인), Connector, Gmail, 캐시·동기화, 카드 계산 |
| 261006-1730-01-m2a-google-calendar.md | Implemented | codex | planning/docs/prd/ai-os-dashboard.md | M2a: 기본 캘린더 일정, 초대 미응답 판정 |
| 261006-1730-02-m2b-notion.md | Approved | codex | planning/docs/prd/ai-os-dashboard.md | M2b: 프로젝트 플래너 작업, 마감 지남 판정 |
| 261006-1730-03-m2c-github.md | Approved | codex | planning/docs/prd/ai-os-dashboard.md | M2c: 리뷰 요청·내 PR·할당 이슈 |
| 261006-1730-04-m3-ai-judgment.md | Approved | codex | planning/docs/prd/ai-os-dashboard.md | M3: Claude 메일 판정, 👍/👎, 비용 상한 |

## How to add a spec

1. Copy `_template.md` to `YYMMDD-HHMM-NN-short-title.md` in this folder.
2. Fill it in with status `Draft`.
3. Add a row above, and put the spec's path in the source PRD's
   `## Handoff` section.
