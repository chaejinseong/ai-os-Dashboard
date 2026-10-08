# M3 월 AI 비용 상한 엄수

- Date: 2026-10-08
- Agent: codex (Jim)
- Spec: app/docs/specs/261008-1420-01-m3-strict-cost-cap.md
- Commits: 1ce5b77

## Summary

AI 요청마다 최대 비용을 메모리에 예약한다. 완료 후 실제 비용으로 교체해 동시 2건을 유지하면서 한 번의 동기화에서 설정된 월 상한을 넘는 새 요청을 보내지 않는다.

## Changes by file

### `app/src-tauri/src/ai.rs`
- Where: `JudgeError`, `worst_case_cost`, `fits_cap`, `judge`, `tests::worst_case_uses_utf8_bytes_and_cap_boundary`
- Purpose: 요청 전 최대 비용과 HTTP 상태 오류·네트워크 오류를 구별한다.
- Method: system·user 내용의 UTF-8 바이트 수에 64토큰 여유와 최대 출력 토큰을 합산한다. `spent + reserved + worst <= cap` 경계로 예약 가능 여부를 판정한다. `under_cap`은 제거한다.

### `app/src-tauri/src/sync.rs`
- Where: `AppState::judge_mail_candidates`, `run_capped`, 예약 경계 테스트 3개
- Purpose: 이미 보낸 동시 요청의 잠재 비용까지 월 상한에 반영한다.
- Method: 후보별 최대 비용을 예약하고 최대 2건을 실행한다. 성공 시 실제 비용으로 바꾸고 HTTP 상태 실패 시 예약을 해제하며 네트워크·작업 오류는 예약액을 보수적으로 사용한 것으로 센다. 가짜 판정 함수로 두 후보·예약 반환·네트워크 실패를 검증한다.

### `app/src-tauri/src/commands.rs`
- Where: `StatsResponse`, `get_stats`, `cap_reached`, 경계 테스트
- Purpose: 실제 비용이 상한에 닿기 전에 더 이상 최소 요청을 보낼 수 없는 상태를 표시한다.
- Method: 빈 메일 요청의 최대 비용이 잔여 한도를 넘는지 계산해 `cap_reached`를 반환한다.

### `app/src/api.ts`
- Where: `Stats.cap_reached`
- Purpose: 상한 중지 상태를 프런트엔드 타입에 전달한다.
- Method: 통계 응답의 불리언 필드를 추가한다.

### `app/src/components/TopBar.tsx`
- Where: `TopBar` 상한 배너 조건
- Purpose: 예약 가능한 최소 요청이 없으면 중지 문구를 보인다.
- Method: 비용의 단순 `>=` 비교 대신 서버의 `cap_reached`를 사용한다.

### `app/docs/specs/261008-1420-01-m3-strict-cost-cap.md`, `app/docs/specs/00-index.md`
- Where: `Status`, `Reports`, `Done when`, 후속 스펙 행
- Purpose: 구현 상태와 검증 결과를 리뷰어에게 넘긴다.
- Method: 자동 검증 결과를 체크하고 보고서 경로를 연결한다.

### `app/docs/todo/261008-1415-01-m3-inflight-cost-cap.md`, `app/docs/todo/00-todo-list.md`
- Where: `Status`, `What's left`, TODO 인덱스 행
- Purpose: 동시 요청으로 상한을 넘을 수 있던 과제를 닫는다.
- Method: 예약 방식 구현과 검증을 기록하고 열린 목록에서 제거한다.

### `app/docs/reports/261008-1429-01-m3-strict-cost-cap.md`
- Where: `Changes by file`, `Verified`, `Follow-ups`
- Purpose: 변경 이유와 검증 근거를 남긴다.
- Method: 실행한 명령의 실제 결과와 실제 키를 사용하는 수동 확인의 한계를 기록한다.

## Verified

- `cargo test --manifest-path src-tauri/Cargo.toml` → god 호스트 실행: exit 0, 40 passed, 0 failed. 로컬에서는 38 passed, 기존 OAuth loopback 테스트 2개가 `PermissionDenied`(localhost bind 금지)로 실패했다.
- `cargo test --manifest-path src-tauri/Cargo.toml --lib -- --skip oauth` → 38 passed, 0 failed, 2 filtered out. 새 테스트는 가짜 판정 함수만 사용했다.
- `cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings` → exit 0, 경고 0.
- `npm run build` → exit 0, TypeScript 검사·Vite 빌드 성공, 1,585 modules transformed.
- `grep -rn "under_cap" src-tauri/src` → 결과 없음 (exit 1).
- `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check`, `git diff --check` → exit 0.
- 실제 Anthropic API 호출·키 사용 없음. 수동 확인은 키가 필요하므로 미실행했다.

## Review

- Reviewer: claude (pending)
- Result: pending

## Follow-ups

- 수동 확인: app/docs/todo/261006-1730-01-notion-github-setup.md에 필요한 Anthropic 키 준비 후 스펙 시나리오를 실행한다.
- 네트워크 오류·타임아웃 요청의 과금 여부는 이 동기화에서만 보수적으로 예약액을 센다. 이후 재시도 비용 추적은 스펙의 범위 밖이다.
