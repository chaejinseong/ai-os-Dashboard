# M3 형식 오류 응답 재요청 방지

- Date: 2026-10-08
- Agent: codex (Jim)
- Spec: none — trivial review follow-up (app/docs/reports/261008-1436-01-m3-parse-failure-cost.md suggestion 1)
- Commits: fa4405e

## Summary

과금된 200 응답의 본문 형식이 잘못됐을 때 비용과 `needs_action=true` 판정을 함께 저장해 같은 메일의 재요청을 막는다.

## Changes by file

### `app/src-tauri/src/sync.rs`
- Where: `AppState::judge_mail_candidates`, `pending_mail_candidates`, `run_capped`, `malformed_200_is_not_retried_on_next_sync`
- Purpose: 형식 오류 응답 후 같은 메일을 다음 동기화에서 다시 판정하지 않는다.
- Method: 기존 성공 경로의 `on_done`으로 보수적 판정과 비용을 함께 저장한다. 후보 선별을 함수로 분리해 가짜 판정으로 동기화 두 번 중 API 호출 한 번을 검증한다.

### `app/src-tauri/src/store.rs`
- Where: `Store::record_judgment`, `Store::record_usage`, `usage_without_judgment_is_counted`
- Purpose: 더 이상 별도 호출하지 않는 비용 삽입 함수를 판정 저장과 공유한다.
- Method: `record_usage`를 트랜잭션 내부 함수로 사용해 판정·비용의 원자적 저장을 유지하고 기존 회귀 테스트는 보존한다. 스키마 변경은 없다.

### `app/docs/reports/261008-1443-01-m3-malformed-response-retry.md`
- Where: `Summary`, `Changes by file`, `Verified`
- Purpose: 리뷰용 변경 근거와 검증 결과를 남긴다.
- Method: 변경 함수와 실행한 검사를 기록한다.

## Verified

- `cargo test --manifest-path src-tauri/Cargo.toml --lib -- --skip oauth` → 43 passed, 0 failed, 2 filtered out (샌드박스의 localhost 바인드 제한으로 OAuth 테스트 제외).
- `cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings` → exit 0, 경고 0.
- `npm run build` → exit 0, 1,585 modules transformed.
- `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check`, `git diff --check` → exit 0.
- 실제 Anthropic API 호출 없음.

## Review

- Pending Claude review.

## Follow-ups

- 없음.

- god 호스트 검증(2026-10-08): `cargo test --manifest-path src-tauri/Cargo.toml` → exit 0, `ok. 45 passed; 0 failed`.
