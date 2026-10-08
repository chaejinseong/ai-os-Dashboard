# m2a-google-calendar

- Status: Implemented (2026-10-08)
- Created: 2026-10-06
- Author: claude
- Implementer: codex
- Source PRD: planning/docs/prd/ai-os-dashboard.md (M2, FR8)
- Depends on: app/docs/specs/261006-1535-01-m1-app-skeleton-gmail.md (Reviewed)
- Reports: app/docs/reports/261008-1036-01-m2a-google-calendar.md

> Only the human sets `Approved`.

## Goal

Google 계정이 연결돼 있으면 기본 캘린더의 일정이 동기화되어 SCHEDULE 카드에
오늘·내일 일정이 뜨고, 아직 응답하지 않은 다가오는 초대가 놓친 업무와
PIPELINE "요청"에 잡힌다.

## Out of scope

- 기본(primary) 외 캘린더, 일정 생성·수정·응답
- 새 OAuth 흐름 (M1이 이미 `calendar.readonly`를 요청한다)

## Files

| File | Change | What |
|---|---|---|
| `app/src-tauri/src/connectors/google_calendar.rs` | new | Calendar 커넥터 + 변환 함수 |
| `app/src-tauri/src/connectors/mod.rs` | modify | 레지스트리에 추가 |
| `app/src-tauri/src/missed.rs` | modify | 초대 미응답 규칙 |
| `app/src/pages/Connections.tsx` | modify | Google 카드에 Calendar 상태 표시 (이미 있으면 변경 없음) |
| `app/src/components/cards/ScheduleCard.tsx` | modify | 응답 안 한 초대 표시 |

Anything not listed here is off-limits without stopping to ask.

## Approach

- `fetch`: `GET https://www.googleapis.com/calendar/v3/calendars/primary/events`
  `?timeMin={now-1d}&timeMax={now+14d}&singleEvents=true&orderBy=startTime&maxResults=250`
  (RFC3339). `nextPageToken`이 있으면 이어서 가져온다. access token은 M1의
  `oauth.rs`에서 받는다.
- 순수 함수 `fn event_to_item(json: &Value) -> Option<Item>`:
  - `status == "cancelled"`이면 `None`
  - kind `Event`, title = `summary` (없으면 "(제목 없음)"), subtitle = 시간 문자열은
    프런트에서 만들므로 `location` (없으면 None)
  - timestamp = `start.dateTime`, 종일 일정은 `start.date`의 로컬 0시를 UTC로
  - due = `end` (같은 방식), url = `htmlLink`, done = false
  - meta: `{ all_day, my_response, created }`. `created` = 이벤트 `created` 필드. `my_response` = `attendees` 중 `self == true`인
    항목의 `responseStatus` (없으면 `null` — 내가 만든 일정 등)
  - stage: `my_response == "needsAction"`이면 `Requested`, 아니면 `None`
- `missed.rs` 규칙 추가: `kind == Event` && `my_response == "needsAction"` &&
  `timestamp > now` → reason "초대에 응답 안 함", since = `meta.created`
- 화면: SCHEDULE 카드에서 `my_response == "needsAction"` 항목에 "응답 필요" 표시.

## Don't touch

- `oauth.rs` (scope는 M1에서 이미 포함), 다른 커넥터

## Done when

- [x] `cargo test --manifest-path src-tauri/Cargo.toml` 통과. 추가 테스트:
  `event_to_item` — 종일 일정, cancelled, attendees 없음, 내가 needsAction인 초대;
  `missed` — 지난 초대는 제외, 다가오는 needsAction 포함
- [x] `cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings` 경고 0
- [x] `npm run build` 성공
- [ ] 수동 확인: 다른 계정에서 나를 초대한 일정을 만들고 응답하지 않으면 MISSED와
  PIPELINE 요청에 뜬다. 오늘 일정이 SCHEDULE 카드에 시간순으로 뜬다.

## Open questions

- 없음
