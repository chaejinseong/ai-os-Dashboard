# M2a Google Calendar

- Date: 2026-10-08
- Agent: codex (Jim)
- Spec: app/docs/specs/261006-1730-01-m2a-google-calendar.md
- Commits: cf81bb9

## Summary

기본 Google Calendar 일정을 공통 `Item`으로 동기화하고, 응답하지 않은 다가오는 초대를 놓친 업무와 PIPELINE 요청에 포함했다. 연결 화면에는 Gmail과 Calendar 상태를 각각 표시한다.

## Changes by file

### `app/src-tauri/src/connectors/google_calendar.rs`
- Where: `GoogleCalendar::fetch`, `event_time`, `event_to_item`, `tests`
- Purpose: 기본 캘린더 일정을 가져와 공통 모델로 변환한다.
- Method: 1일 전부터 14일 뒤까지 시간순으로 페이지를 따라 읽고, 취소된 일정은 제외한다. 종일 일정은 로컬 자정으로 바꾸고 본인의 `needsAction` 응답만 요청 단계로 표시한다. 변환 경계 사례를 순수 함수 테스트로 확인한다.

### `app/src-tauri/src/connectors/mod.rs`
- Where: `registry`
- Purpose: 기존 동기화 흐름에 Calendar를 등록한다.
- Method: Gmail과 Calendar가 동일한 `GoogleAuth`를 공유하도록 `Arc`를 전달한다.

### `app/src-tauri/src/missed.rs`
- Where: `unanswered_invitation`, `missed`, `tests`
- Purpose: 다가오는 미응답 초대를 놓친 업무로 보여준다.
- Method: Event의 본인 응답이 `needsAction`이고 시작 시간이 현재보다 뒤인 경우만 포함한다. 판정 시각은 일정의 `created`를 사용하며 지난 일정, 시작 시각 경계, 수락한 일정은 제외하는 테스트를 추가한다.

### `app/src/components/cards/ScheduleCard.tsx`
- Where: `ScheduleCard`
- Purpose: 일정 카드에 종일 일정과 응답 필요 상태를 표시한다.
- Method: `all_day`는 날짜와 종일 문구로, `my_response`가 `needsAction`이면 배지로 렌더링한다. 일정이 없을 때의 문구도 실제 상태에 맞췄다.

### `app/src/pages/Connections.tsx`
- Where: `Connections`
- Purpose: Google 연결에서 Calendar의 동기화 상태를 구분해 보여준다.
- Method: Gmail과 Calendar의 `SourceStatus`를 각각 읽어 연결·동기화·마지막 시각·오류를 표시하고 공통 Google 연결 작업은 유지한다.

### `app/docs/specs/261006-1730-01-m2a-google-calendar.md`, `app/docs/specs/00-index.md`
- Where: `Status`, `Reports`, `Done when`, M2a 행
- Purpose: 구현 상태와 검증 결과를 리뷰어에게 넘긴다.
- Method: 구현 완료 상태와 본 보고서 경로를 기록하고 확인된 자동 검증만 체크한다.

### `app/docs/reports/261008-1036-01-m2a-google-calendar.md`
- Where: `Changes by file`, `Verified`, `Follow-ups`
- Purpose: 변경 의도와 검증 한계를 남긴다.
- Method: 파일별 근거, 실제 명령 결과, 수동 확인의 선행 조건을 기록한다.

## Verified

- `cargo test --manifest-path src-tauri/Cargo.toml` (`CARGO_HOME`의 준비된 오프라인 캐시 사용) → 호스트 재실행 exit 0, 13 passed, 0 failed. 신규 Calendar 테스트 4개와 초대 판정 테스트 1개, 기존 OAuth loopback 테스트 2개가 포함된다. 이 샌드박스에서는 loopback 바인드 제한으로 OAuth 테스트 2개만 `PermissionDenied`가 났다.
- 동일 명령에 `-- --skip oauth::tests::idle_and_closed_connections_do_not_abort_callback --skip oauth::tests::idle_connections_preserve_overall_deadline`를 추가 → exit 0. 11 passed, 0 failed, 2 filtered out. main 및 doc 테스트도 실패 0.
- `cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings` (`CARGO_HOME=/tmp/claude-501/cargo-home CARGO_NET_OFFLINE=true`) → exit 0, 경고 0.
- `npm run build` → exit 0. TypeScript 검사와 Vite 빌드 통과, 1,585 modules transformed.
- `cargo fmt --manifest-path src-tauri/Cargo.toml --check`, `git diff --check` → exit 0.
- 수동 확인 미실행: 실제 Google 계정의 초대·일정 확인에는 Google Cloud 설정과 다른 계정이 필요하다. M1의 `app/docs/todo/261006-1535-01-google-cloud-setup.md`에서 연결 준비를 추적한다.

## Review

- Reviewer: claude (대기)
- Result: 대기

## Follow-ups

- `app/docs/todo/261007-0931-01-google-disconnect-scope.md`: M2b로 넘어가기 전에 Google 연결 해제 대상을 Google 소스로 제한하고 다른 소스 캐시·상태 보존을 검증한다. M2a 스펙의 허용 파일에 `commands.rs`가 없어 이번 커밋에는 포함하지 않았다.
