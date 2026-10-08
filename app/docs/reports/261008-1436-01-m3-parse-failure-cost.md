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

- Reviewer: claude (Pam), 2026-10-08, round 1
- Result: must-fix 0, suggestion 1. 스펙 없음(리뷰 제안 후속)이라 상태 변경 대상 없음.
- Re-run: `cargo clippy ... -- -D warnings` 경고 0; `cargo test ... -- --skip oauth::tests` 42 passed (oauth loopback 2개는 샌드박스 바인드 제한, god 호스트 실행 44 passed 근거).
- 보고서 ↔ 코드 일치: df71677 diff와 맞다. `parse_api_response`는 본문이 JSON이 아니거나 `content[0].text`가 없을 때만 `Response`를 돌려주고, 텍스트는 있는데 모델 답이 JSON이 아닌 경우는 기존 M3대로 `needs_action = true`로 판정·저장된다(동작 유지). `Response(usage)`는 사용량이 있으면 실제 가격, 없으면 예약 최대 비용으로 `spent`에 더하고 `record_usage`로 `ai_usage`에 남긴다 → 다음 동기화의 월 누적에도 반영. 판정은 저장하지 않는다. 스키마 변경 없음. 새 테스트 4개는 가짜 judge·메모리 DB만 쓴다(실제 API 없음).

1. `suggestion` — 판정이 저장되지 않으므로 같은 메일은 다음 동기화마다 다시 요청된다. 응답 형식이 계속 깨지는 상황(API 형식 변경 등)이면 상한까지 매번 비용이 쌓인다. 상한은 지켜지므로 차단 아님. 필요하면 같은 `(thread_id, last_message_id)`의 파싱 실패 횟수를 세어 건너뛰는 방식을 검토한다.

## Follow-ups

- 없음.

- god 호스트 검증(2026-10-08): `cargo test --manifest-path src-tauri/Cargo.toml` → exit 0, `ok. 44 passed; 0 failed`.
