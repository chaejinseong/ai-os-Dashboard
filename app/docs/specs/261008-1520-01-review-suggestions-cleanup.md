# review-suggestions-cleanup

- Status: Reviewed (claude/Pam, 2026-10-10; Implemented: codex)
- Created: 2026-10-08
- Author: claude (Pam)
- Implementer: codex
- Source PRD: none — 남은 리뷰 suggestion 정리 (사람 요청 "handle the remaining suggestions")
- Depends on: M1·M2a·M2b·M2c·M3 스펙 (모두 Reviewed)
- Reports: app/docs/reports/261008-1521-01-review-suggestions-cleanup.md

> Only the human sets `Approved`. Once approved, the implementer builds
> this without asking for plan approval again — so write it so nobody
> has to guess.

## Goal

M1~M3 리뷰에서 열린 채로 남은 suggestion을 모두 정리한다. 고칠 것은 고치고, 고치지
않을 것은 이유를 남긴다. 사용자가 보는 효과: 메일·GitHub 데이터 하나가 깨져도 동기화가
멈추지 않고, 오프라인을 "다시 로그인"으로 오해하지 않으며, 여러 날 일정과 Notion
상태 이름 변경이 올바르게 반영되고, AI 키 오류가 설정 화면에 보인다.

## Out of scope

- 새 기능, 새 커넥터, DB 스키마 변경 (아래 항목 중 스키마 변경은 **없다** —
  새 값은 기존 `settings` key/value 테이블만 쓴다)
- 의존성 추가 (포매터 포함)
- 아래 "won't fix" 항목

## Decisions

| # | 출처 | 항목 | 결정 |
|---|---|---|---|
| A1 | M1 s2 | Gmail 스레드 하나의 형식 오류가 동기화 전체 실패 | fix |
| A2 | M1 s3 | 토큰 갱신의 네트워크 실패도 "다시 로그인 필요" | fix |
| A3 | M2c s1 | PR 하나의 events 조회 실패가 GitHub 동기화 전체 실패 | fix |
| B1 | M2a s1 | 어제 시작해 오늘까지 이어지는 일정이 SCHEDULE에서 빠짐 | fix |
| B2 | M2a s2 | `meta.created` 없는 초대가 PIPELINE엔 있고 MISSED엔 없음 | fix |
| B3 | M2b s2 | Notion 상태 매핑이 option 이름 기준 → 이름 바꾸면 깨짐 | fix |
| C1 | M3 s1 | Gmail 스레드 👎가 이후 새 메일까지 숨김 | fix |
| C2 | M3 s2 | AI 호출 실패(잘못된 키·429)가 설정에 안 보이고 매 동기화 재시도 | fix |
| C3 | M3 s3 | 판정이 이미 있으면 사용량 미기록 (fa4405e 이후에도 그대로 — `record_judgment`의 `INSERT OR IGNORE` 후 `inserted == 1`일 때만 기록) | fix |
| D1 | disconnect s1 | `"gmail"`, `"calendar"` 하드코딩 | fix |
| D2 | M2b s3, M2c s3 | 연결 카드들이 `busy` 하나를 공유 | fix |
| E1 | M1 s5 | TSX 한 줄 압축 (oauth.rs는 이미 rustfmt로 해결됨) | fix — 포맷만 |
| W1 | M2c s2 | PR events를 첫 100건만 읽음 | **won't fix** — 이벤트 100건 넘는 PR에서 재요청된 리뷰만 영향, 결과는 "실제로 대기 중인 리뷰가 MISSED에 일찍 뜸"으로 무해. 고치려면 `Link` 헤더 페이지 처리와 PR당 추가 호출이 필요해 비용 대비 이득이 없다 |

## Files

| File | Change | Item |
|---|---|---|
| `app/src-tauri/src/connectors/gmail.rs` | modify | A1 |
| `app/src-tauri/src/oauth.rs` | modify | A2 |
| `app/src-tauri/src/connectors/github.rs` | modify | A3 |
| `app/src-tauri/src/summary.rs` | modify | B1 (이 스펙이 M1의 "summary.rs 규칙 금지"를 이 한 규칙에 한해 푼다) |
| `app/src-tauri/src/missed.rs` | modify | B2, C1 |
| `app/src-tauri/src/connectors/notion.rs` | modify | B3 |
| `app/src-tauri/src/sync.rs` | modify | C2 |
| `app/src-tauri/src/store.rs` | modify | C2 (settings key 읽기·쓰기), C3 |
| `app/src-tauri/src/commands.rs` | modify | C2 (`StatsResponse.ai_error`), D1 |
| `app/src-tauri/src/connectors/mod.rs` | modify | D1 |
| `app/src/api.ts` | modify | C1 (`MissedItem.feedback_key`), C2 (`Stats.ai_error`) |
| `app/src/components/cards/MissedCard.tsx`, `app/src/pages/Alerts.tsx` | modify | C1 |
| `app/src/pages/Settings.tsx` | modify | C2 |
| `app/src/pages/Connections.tsx` | modify | D2 |
| `app/src/components/Sidebar.tsx`, `TitleBar.tsx`, `TopBar.tsx`, `app/src/pages/Overview.tsx`, `WorkList.tsx`, `app/src/components/cards/BoardCard.tsx`, `CompletionCard.tsx`, `PipelineCard.tsx` | modify | E1 (포맷만) |

`lib.rs`(새 명령 없음), `model.rs`(Settings 필드 없음), `ai.rs`, DB 스키마는 바꾸지 않는다.
Anything not listed here is off-limits without stopping to ask.

## Approach

**A1 Gmail** — 스레드 작업이 `Result<Option<Item>, ConnectorError>`를 돌려주게 한다
(`thread_to_item`의 `None`은 오류가 아님). 결과 모으기를 순수 함수
`fn collect_threads(results: Vec<Result<Option<Item>, ConnectorError>>) -> Result<Vec<Item>, ConnectorError>`로
뺀다: `Ok(None)`은 건너뛰고 `eprintln!("메일 스레드 형식 오류 — 건너뜀")`(고정 문구),
`Err`(네트워크·HTTP)은 지금처럼 전체 실패.

**A2 OAuth** — `access_token`의 `exchange_refresh_token` 오류를 순수 함수
`fn refresh_error_message(err: &RequestTokenError<..>) -> &'static str`로 나눈다:
`RequestTokenError::ServerResponse(_)`(invalid_grant 등 Google이 거절) → "다시 로그인 필요",
그 밖(`Request`·`Parse`·`Other`, 네트워크) → "Google 연결 실패 — 다음 동기화에 다시 시도".
refresh token이 Keychain에 없을 때는 지금처럼 "다시 로그인 필요".

**A3 GitHub** — reviewer PR의 events 조회에서 `?`를 없앤다. 순수 함수
`fn requested_at_or(events: Option<&Value>, login: &str, fallback: Option<DateTime<Utc>>) -> Option<DateTime<Utc>>`
= `events.and_then(|e| requested_at_from_events(e, login)).or(fallback)`. 조회 실패는
`None`으로 넘기고 `eprintln!("GitHub 리뷰 요청 이벤트 조회 실패 — PR 생성 시각 사용")`.

**B1 SCHEDULE** — `summary.rs`의 일정 선택을 "시작일이 오늘·내일"에서 "오늘 0시~내일 끝과
겹침"으로 바꾼다: `start = i.timestamp`, `end = i.due.unwrap_or(i.timestamp)`;
`start < 모레 0시(로컬) && end > 오늘 0시(로컬)`이면 포함. 정렬은 그대로 시작 시각순.
(Calendar 조회는 `timeMin = now-1d`라 그보다 먼저 시작한 일정도 끝이 범위 안이면 이미 받아온다.)

**B2 초대 since** — `unanswered_invitation`에서 `meta.created`가 없거나 파싱 실패면
`since = now`로 대신한다 (항목은 계속 MISSED에 포함).

**B3 Notion option id** — `mapping_from_source`가 `stages`의 key를 option **id**로
저장한다 (`시작 전`/`진행 중`/`완료` 대체 판정도 해당 option의 id로). `page_to_item`은
`properties[status]["status"]["id"]`로 먼저 찾고, 없으면 `["name"]`으로 찾는다 → 이미 저장된
이름 기준 매핑도 다시 연결 없이 계속 동작한다. `Mapping` 구조체·`settings` 저장 형식은 그대로.

**C1 👎 키** — `missed.rs`에 `pub fn feedback_key(item: &Item) -> String`:
Email이면 `"{id}#{meta.last_message_id}"`, 그 밖은 `id`. `MissedItem`에 직렬화 필드
`feedback_key: String`을 추가하고 `missed()`가 채운다. 👎 제외 검사(`missed`)와
`commands.rs`의 `apply_ai_stages`는 `(source, feedback_key(item))`로 조회한다.
프런트는 `api.setFeedback(m.item.source, m.feedback_key, value)`를 보낸다 (명령 시그니처 그대로).
기존에 스레드 ID로 저장된 👎는 더 이상 맞지 않아 한 번 다시 보일 수 있다 — 허용.

**C2 AI 오류 표시** — `run_capped`가 `Result<(f64, Option<String>), String>`을 돌려준다
(두 번째 값 = 이번 실행의 마지막 AI 오류 문구, 고정 문구만: "AI 키 확인 필요 (401)",
"AI 권한 없음 (403)", "AI 요청 한도 초과 (429)", "AI 요청 실패 ({code})", "AI 네트워크 오류").
`JudgeError::Status(401 | 403)`이면 이후 후보를 더 보내지 않는다 (진행 중인 것만 기다림 —
키가 틀리면 나머지도 다 실패하므로). `judge_mail_candidates`는 결과로
`store.set_ai_error(Option<&str>)`를 부른다: 오류가 있으면 `settings`의 `ai.last_error`에
저장, 이번 실행에 성공이 한 건이라도 있고 오류가 없으면 지운다. `save_anthropic_key`도 지운다.
`get_stats`의 `StatsResponse`에 `ai_error: Option<String>`, Settings 화면의 AI 사용 현황에
있으면 빨간 문구로 보인다. 키 값은 어떤 문구에도 넣지 않는다.

**C3 사용량 항상 기록** — `record_judgment`에서 판정 `INSERT OR IGNORE` 결과와 무관하게
`ai_usage`에 기록한다 (비용은 이미 냈다).

**D1 Google 소스 상수** — `connectors/mod.rs`에 `pub const GOOGLE_SOURCES: [&str; 2] = ["gmail", "calendar"];`.
`commands.rs::clear_google_cache`가 이것을 쓴다.

**D2 카드별 busy** — `Connections.tsx`의 `busy: boolean`을
`busy: 'google' | 'notion' | 'github' | null`로 바꾸고 `run(card, action)`이 해당 카드만
잠근다. 브라우저 로그인 안내 문구와 "처리 중…"은 `busy === 'google'`일 때만.
다른 카드의 버튼은 동시에 누를 수 있다 (백엔드 `operation` 잠금이 이미 순서를 지킨다).

**E1 TSX 포맷** — 위 8개 파일에서 200자 넘는 줄을 JSX 요소·props 단위로 나눠 기존
`Connections.tsx` 스타일(2칸 들여쓰기)로 맞춘다. 로직·클래스명·문구 변경 없음.
이 커밋은 다른 변경과 섞지 않는다.

## Commits

1. `fix(app): keep syncing past one bad Gmail thread or GitHub events call` — A1, A3
2. `fix(app): tell offline refresh apart from expired Google login` — A2
3. `fix(app): schedule overlap, invite fallback, Notion option ids` — B1, B2, B3
4. `fix(app): per-message feedback, visible AI errors, always record usage` — C1, C2, C3
5. `refactor(app): shared Google source ids and per-card busy state` — D1, D2
6. `style(app): reflow compressed TSX` — E1

## Don't touch

- 프롬프트, 비용 상한 로직(`fits_cap`·예약), 동시 실행 수
- Tauri 명령 목록과 시그니처, DB 테이블 정의, `Settings` 필드
- 실제 외부 API 호출 (테스트는 순수 함수·메모리 DB·가짜 클로저만)

## Done when

명령은 `app/`에서 실행한다.

- [x] `cargo test --manifest-path src-tauri/Cargo.toml` 통과 (호스트 56 passed; 로컬 샌드박스는 OAuth localhost bind 권한으로 2 failed). 추가 테스트:
  - A1 `gmail::tests`: `collect_threads([Ok(Some), Ok(None), Ok(Some)])` → 2건; `[Ok(Some), Err]` → `Err`
  - A2 `oauth::tests`: `ServerResponse(invalid_grant)` → "다시 로그인 필요"; `RequestTokenError::Other(..)` → 다시 시도 문구
  - A3 `github::tests`: `requested_at_or(None, …, Some(t))` → `t`; 일치 이벤트 있으면 이벤트 시각
  - B1 `summary::tests`: 어제 시작·내일 끝 일정 포함, 어제 시작·어제 끝 제외, 모레 시작 제외, 종일 일정(오늘 0시~내일 0시) 포함
  - B2 `missed::tests`: `created` 없는 다가오는 needsAction 초대가 `since = now`로 포함
  - B3 `notion::tests`: 매핑 key가 option id; 페이지의 id로 단계 판정; option 이름만 바꾼 페이지도 같은 단계; 이름 기준 옛 매핑도 동작
  - C1 `missed::tests`: 같은 스레드에 새 `last_message_id`가 오면 이전 👎가 적용되지 않음; Task는 `id` 그대로
  - C2 `sync::tests`: 가짜 judge가 첫 건에 `Status(401)` → 두 번째 후보 호출 안 함, 오류 문구 "AI 키 확인 필요 (401)"; `store::tests`: `ai.last_error` 저장·삭제 왕복
  - C3 `store::tests`: 같은 판정을 두 번 기록하면 판정 1건, 사용량·비용은 2건 합
  - D1 `connectors::tests`: `GOOGLE_SOURCES`가 Gmail·Calendar 커넥터의 `id()`와 같음
- [x] `cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings` 경고 0
- [x] `npm run build` 성공
- [x] `find src -name '*.tsx' -exec awk 'length>200' {} +` 결과 없음 (E1)
- [ ] D2 수동 확인 (프런트 테스트 도구 없음): Notion 연결 중 Google 카드에 브라우저 로그인 안내가 안 보이고 GitHub 연결 버튼은 눌린다.

## Open questions

- 없음.

---
File naming: `YYMMDD-HHMM-NN-short-title.md`, same as todo/ and reports/.
