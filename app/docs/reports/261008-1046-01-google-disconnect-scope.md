# Google 연결 해제 범위 제한

- Date: 2026-10-08
- Agent: codex (Jim)
- Spec: none — app/docs/todo/261007-0931-01-google-disconnect-scope.md
- Commits: 2c574a7

## Summary

Google 연결 해제 시 Gmail과 Calendar 캐시·상태만 지우도록 범위를 제한했다. 향후 Notion·GitHub가 등록되어도 해당 데이터가 보존되는 회귀 테스트를 추가했다.

## Changes by file

### `app/src-tauri/src/commands.rs`
- Where: `disconnect_google`, `clear_google_cache`, `tests::google_disconnect_preserves_other_sources`
- Purpose: Google 연결 해제가 레지스트리 전체 데이터를 지우던 문제를 해결한다.
- Method: Keychain 연결 해제 후 Google 소스 두 개만 초기화한다. 메모리 SQLite에 네 소스의 항목과 상태를 넣고 Google은 삭제, Notion·GitHub는 항목과 상태가 보존됨을 검사한다.

### `app/docs/todo/261007-0931-01-google-disconnect-scope.md`, `app/docs/todo/00-todo-list.md`
- Where: `Status`, `What's left`, TODO Index
- Purpose: M2b 선행 작업을 완료로 기록한다.
- Method: 완료 항목을 체크하고 열린 작업 목록에서 제거한다. 과거 TODO 파일은 유지한다.

### `app/docs/reports/261008-1046-01-google-disconnect-scope.md`
- Where: `Changes by file`, `Verified`
- Purpose: 수정 의도와 검증 결과를 남긴다.
- Method: 코드 심볼별 근거와 실제 테스트 결과를 기록한다.

## Verified

- `cargo test --manifest-path src-tauri/Cargo.toml commands::tests::` → exit 0, `google_disconnect_preserves_other_sources` 1 passed.
- `cargo test --manifest-path src-tauri/Cargo.toml -- --skip oauth::tests::idle_and_closed_connections_do_not_abort_callback --skip oauth::tests::idle_connections_preserve_overall_deadline` → exit 0, 12 passed, 0 failed, OAuth loopback 2 filtered out. 샌드박스는 loopback 바인드를 허용하지 않는다.
- `cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings` → exit 0, 경고 0.
- `npm run build` → exit 0, TypeScript 검사와 Vite 빌드 성공, 1,585 modules transformed.
- `cargo fmt --manifest-path src-tauri/Cargo.toml`, `git diff --check` → exit 0.

## Review

- Reviewer: claude (대기)
- Result: 대기

## Follow-ups

- M2b 스펙 파일 범위 보완: app/docs/todo/261008-1045-01-m2b-spec-file-scope.md.

- god 검증(샌드박스 밖, 2026-10-08): `cargo test --manifest-path src-tauri/Cargo.toml` → `ok. 14 passed; 0 failed` (`commands::tests::google_disconnect_preserves_other_sources` 포함).
