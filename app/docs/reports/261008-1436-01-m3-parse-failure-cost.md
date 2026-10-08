# M3 200 응답 파싱 실패 비용 기록

- Date: 2026-10-08
- Agent: codex (Jim)
- Spec: none — trivial fix (strict cost cap review suggestion 2)
- Commits: df71677

## Summary

Anthropic 200 응답 본문을 해석할 수 없어도 읽을 수 있는 사용량 또는 예약한 최대 비용을 기록하고 판정은 저장하지 않는다.

## Changes by file

### `app/src-tauri/src/ai.rs`
- Where: `JudgeError`, `parse_api_response`, `judge`, `malformed_api_body_reports_readable_usage`
- Purpose: 200 응답의 본문 오류를 네트워크 오류와 구별한다.
- Method: 본문 형식을 검사해 사용량 토큰을 읽거나 없음으로 반환하며, 기존 정상 응답 판정 로직은 유지한다.

### `app/src-tauri/src/sync.rs`
- Where: `AppState::judge_mail_candidates`, `run_capped`, `malformed_200_*` tests
- Purpose: 과금된 파싱 실패를 월 비용에 반영하고 판정 저장을 피한다.
- Method: 사용량이 있으면 실제 가격으로, 없으면 예약 비용으로 계산해 비용 전용 콜백에 전달한다. 가짜 판정으로 두 경우와 판정 미저장을 확인한다.

### `app/src-tauri/src/store.rs`
- Where: `Store::record_usage`, `usage_without_judgment_is_counted`
- Purpose: 판정 없이 사용량과 비용만 SQLite에 저장한다.
- Method: 기존 `ai_usage` 테이블에 기록하고 월 통계에는 포함되지만 `ai_judgments`에는 기록되지 않음을 검증한다. 스키마 변경은 없다.

### `app/docs/reports/261008-1436-01-m3-parse-failure-cost.md`
- Where: `Summary`, `Changes by file`, `Verified`
- Purpose: 리뷰용 변경 이유와 검증 결과를 남긴다.
- Method: 변경한 함수와 실행한 검사를 기록한다.

## Verified

- `cargo test --manifest-path src-tauri/Cargo.toml --lib -- --skip oauth` → 42 passed, 0 failed, 2 filtered out (샌드박스의 localhost 바인드 제한으로 OAuth 테스트 제외).
- `cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings` → exit 0, 경고 0.
- `npm run build` → exit 0, 1,585 modules transformed.
- `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check`, `git diff --check` → exit 0.
- 실제 Anthropic API 호출 없음.

## Review

- Pending Claude review.

## Follow-ups

- 없음.

- god 호스트 검증(2026-10-08): `cargo test --manifest-path src-tauri/Cargo.toml` → exit 0, `ok. 44 passed; 0 failed`.
