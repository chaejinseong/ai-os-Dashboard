# 남은 리뷰 제안 정리

- Date: 2026-10-08
- Agent: codex (Jim)
- Spec: app/docs/specs/261008-1520-01-review-suggestions-cleanup.md
- Commits: `f1b34f1`, `e07d6ed`, `b939879`, `b6f2e69`, `ebaa611`, `7b6549b`

## Summary

M1~M3에서 승인된 리뷰 제안을 여섯 개의 작은 커밋으로 정리한다. W1은 스펙 결정에 따라 변경하지 않는다.

W1의 PR events 100건 제한은 유지했다. 재요청된 리뷰가 많은 PR에서 MISSED가 일찍 표시될 수 있지만, 이를 고치려면 PR별 페이지 요청이 추가된다. 스펙은 비용 대비 이득이 작다고 결정했다.

## Changes by file

### `app/src-tauri/src/connectors/gmail.rs`
- Where: `Gmail::fetch`, `collect_threads`
- Purpose: 메일 스레드 하나의 데이터 형식 오류로 전체 Gmail 동기화가 실패하지 않게 한다.
- Method: `thread_to_item`의 `None`은 고정 로그를 남기고 건너뛰며, 네트워크·HTTP 오류는 전파한다. 혼합 결과 테스트를 추가했다.

### `app/src-tauri/src/connectors/github.rs`
- Where: `GitHub::fetch`, `requested_at_or`
- Purpose: PR events 조회 한 건의 실패로 GitHub 동기화 전체가 멈추지 않게 한다.
- Method: events 요청 실패 시 고정 로그 후 PR 생성 시각으로 대체한다. 요청 이벤트가 있으면 해당 시각을 우선한다.

### `app/src-tauri/src/oauth.rs`
- Where: `GoogleAuth::access_token`, `refresh_error_message`
- Purpose: Google 토큰 갱신 중 일시적 연결 실패를 재로그인 요구로 잘못 표시하지 않는다.
- Method: 서버가 갱신을 거절한 경우에만 재로그인을 요구하고 나머지 오류는 다음 동기화 재시도 문구로 분류한다. 순수 함수 테스트로 둘을 확인한다.

### `app/src-tauri/src/summary.rs`
- Where: `summarize`, `schedule_includes_overlapping_events_only`
- Purpose: 어제 시작해 오늘·내일까지 이어지는 일정을 SCHEDULE에 표시한다.
- Method: 로컬 오늘 0시~모레 0시와 일정 구간이 겹치면 포함한다. 시작·종료 경계와 정렬을 테스트했다.

### `app/src-tauri/src/missed.rs`
- Where: `unanswered_invitation`, `invitation_without_valid_created_uses_now`
- Purpose: 초대 생성 시각이 없는 미응답 일정도 MISSED에 표시한다.
- Method: 생성 시각이 없거나 잘못됐으면 현재 시각을 `since`로 사용하고 두 경우를 테스트한다.

### `app/src-tauri/src/connectors/notion.rs`
- Where: `mapping_from_source`, `page_to_item`, `option_id_survives_rename_and_old_name_mapping_still_works`
- Purpose: 상태 옵션 이름이 변경돼도 작업 단계를 유지한다.
- Method: 새 매핑은 옵션 ID를 저장하고 페이지 판정은 ID를 먼저 본다. 기존 이름 기준 매핑은 이름 조회로 계속 지원한다.

### `app/src-tauri/src/sync.rs`
- Where: `AppState::judge_mail_candidates`, `run_capped`, `status_error_message`
- Purpose: 실패한 AI 호출의 이유를 설정 화면에 전달하고 잘못된 키로 새 요청을 반복하지 않는다.
- Method: 고정 오류 문구를 반환·저장하고 401·403 후에는 진행 중인 요청만 기다린다. 가짜 judge로 401 이후 후보 중지를 검증한다.

### `app/src-tauri/src/store.rs`
- Where: `ai_error`, `set_ai_error`, `record_judgment`
- Purpose: AI 오류를 기존 설정 테이블에 보관하고 실제 과금 사용량을 모두 기록한다.
- Method: `ai.last_error`를 저장·삭제하고 판정 `INSERT OR IGNORE` 결과와 관계없이 사용량을 트랜잭션에 넣는다. 기존 중복 판정 테스트와 왕복 테스트로 확인한다.

### `app/src-tauri/src/commands.rs`
- Where: `save_anthropic_key`, `StatsResponse`, `get_stats`
- Purpose: 새 키 저장 시 오래된 오류를 지우고 최신 오류를 화면에 보낸다.
- Method: Keychain 저장 후 오류 키를 삭제하고 통계 응답에 `ai_error`를 추가한다.

### `app/src/api.ts`
- Where: `Stats.ai_error`
- Purpose: 통계 응답의 AI 오류를 프런트엔드 타입에 반영한다.
- Method: nullable 문자열 필드를 추가한다.

### `app/src/pages/Settings.tsx`
- Where: `Settings` AI 사용 현황
- Purpose: AI 키·권한·한도·네트워크 오류를 설정 화면에 표시한다.
- Method: 통계 갱신 이벤트를 구독하고 오류가 있으면 빨간 문구를 렌더링한다.

### `app/src/pages/Connections.tsx`
- Where: `Connections` busy state, `run`
- Purpose: 한 카드의 처리 상태가 다른 연결 카드까지 잠그지 않게 한다.
- Method: 카드 식별자로 busy를 관리하고 각 카드 입력·버튼 및 Google 안내 문구에만 해당 상태를 적용한다.

### `app/src-tauri/src/connectors/mod.rs`
- Where: `GOOGLE_SOURCES`, `google_sources_match_connector_ids`
- Purpose: Google 연결 해제 대상 소스를 커넥터 정의와 함께 관리한다.
- Method: Gmail·Calendar ID 상수를 만들고 실제 두 커넥터의 `id()`와 일치하는지 테스트한다.

### `app/src-tauri/src/commands.rs` (D1 추가)
- Where: `clear_google_cache`
- Purpose: Google 연결 해제 대상의 중복 문자열을 없앤다.
- Method: `GOOGLE_SOURCES`를 순회한다. 다른 소스는 그대로 보존한다.

Formatting only: `app/src/components/Sidebar.tsx`, `TitleBar.tsx`, `TopBar.tsx`, `app/src/pages/Overview.tsx`, `WorkList.tsx`, `app/src/components/cards/BoardCard.tsx`, `CompletionCard.tsx`, `PipelineCard.tsx` — 200자 초과 줄을 나눴다. 8개 파일 모두 공백 제거 후 이전 버전과 토큰이 같아 로직·문구·클래스 변경이 없다.

### `app/src-tauri/src/missed.rs` (C1 추가)
- Where: `feedback_key`, `MissedItem`, `missed`
- Purpose: 이전 메일의 👎가 같은 스레드의 새 메시지까지 숨기지 않게 한다.
- Method: 메일은 스레드 ID와 마지막 메시지 ID를 합친 키를 사용하고 다른 항목은 기존 ID를 유지한다. 새 메시지와 Task의 경계를 테스트했다.

### `app/src-tauri/src/commands.rs` (C1 추가)
- Where: `apply_ai_stages`
- Purpose: AI 단계 표시에도 메시지별 👎 키를 일관되게 적용한다.
- Method: `missed::feedback_key`로 피드백을 조회하고 새 메시지가 다시 표시되는 테스트를 확장했다.

### `app/src-tauri/src/summary.rs` (테스트만)
- Where: `boundaries_board_pipeline_and_empty`
- Purpose: `MissedItem.feedback_key` 추가 후 기존 요약 테스트를 유지한다.
- Method: 테스트 구조체에 새 키를 채웠다. 요약 계산은 그대로다.

### `app/src/api.ts` (C1 추가)
- Where: `MissedItem.feedback_key`
- Purpose: 직렬화된 메시지별 피드백 키를 화면 타입에 전달한다.
- Method: 문자열 필드를 추가한다.

### `app/src/components/cards/MissedCard.tsx`, `app/src/pages/Alerts.tsx`
- Where: `vote`, `hidden`, list keys
- Purpose: 두 알림 화면의 투표와 즉시 숨김이 메시지별 키를 사용하게 한다.
- Method: `setFeedback` 인자와 로컬 hidden/React 키를 `feedback_key`로 바꾼다.

## Verified

- Commit 1 (A1, A3): `cargo test --manifest-path src-tauri/Cargo.toml --lib -- --skip oauth` → 46 passed, 0 failed, 2 filtered out (샌드박스의 localhost 바인드 제한); `cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings` → exit 0, 경고 0; `git diff --check` → exit 0.
- Commit 2 (A2): `cargo test --manifest-path src-tauri/Cargo.toml --lib -- --skip oauth::tests::idle` → 47 passed, 0 failed, 2 filtered out; `cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings` → exit 0, 경고 0; `git diff --check` → exit 0.
- Commit 3 (B1, B2, B3): `cargo test --manifest-path src-tauri/Cargo.toml --lib -- --skip oauth::tests::idle` → 50 passed, 0 failed, 2 filtered out; `cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings` → exit 0, 경고 0; `git diff --check` → exit 0.
- Commit 4 (C1, C2, C3): `cargo test --manifest-path src-tauri/Cargo.toml --lib -- --skip oauth::tests::idle` → 53 passed, 0 failed, 2 filtered out; `npm run build` → exit 0, 1,585 modules transformed. `cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings` → exit 0, 경고 0; `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check`, `git diff --check` → exit 0.
- Commit 5 (D1, D2): `cargo test --manifest-path src-tauri/Cargo.toml --lib -- --skip oauth::tests::idle` → 54 passed, 0 failed, 2 filtered out. `cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings`, `npm run build`, `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check`, `git diff --check` → exit 0.
- Commit 6 (E1): `find src -name '*.tsx' -exec awk 'length>200' {} +` → 출력 없음. 8개 파일의 공백을 제외한 내용은 변경 전과 동일; `npm run build`와 `git diff --check` → exit 0.
- 최종 `cargo test --manifest-path src-tauri/Cargo.toml`: god 호스트 실행에서 56 passed, 0 failed. 로컬 샌드박스에서는 54 passed, 2 failed (`oauth::tests::idle_*` 두 테스트가 localhost bind 시 `PermissionDenied`), 따라서 위의 두 테스트를 제외한 실행도 별도로 통과시켰다.
- D2 수동 연결 확인은 실제 Notion/GitHub 토큰이 없어 실행하지 않았다. 코드에서 카드별 disabled 조건과 Google 안내 조건을 확인했고 프런트 빌드는 통과했다. 실제 연결 동작은 미검증이다.

## Review

- Pending Claude review.

## Follow-ups

- D2 실제 계정 연결 화면 확인은 계정 준비 후 수행해야 한다 (`app/docs/todo/261006-1730-01-notion-github-setup.md`).
