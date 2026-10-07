# Google 연결 해제 범위 제한

- Status: open
- Created: 2026-10-07
- Owner: codex
- Related spec: app/docs/specs/261006-1535-01-m1-app-skeleton-gmail.md

## Context

M1 리뷰 suggestion 4. `app/src-tauri/src/commands.rs`의 `disconnect_google`은
현재 Registry 전체의 캐시와 상태를 지운다. M1에는 Google 소스만 있지만,
M2에서 다른 서비스를 등록하면 해당 캐시까지 지워진다.

## What's left

- [ ] M2 커넥터 등록 전에 삭제 대상을 Google 소스로 제한한다.
- [ ] Google 연결 해제 후 Notion·GitHub 캐시와 상태가 보존되는지 검증한다.

## Notes

M1 must-fix 수정과 분리한다. 리뷰 근거:
app/docs/reports/261007-0919-01-m1-app-skeleton-gmail.md (Round 1, suggestion 4).
