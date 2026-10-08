# m3-summary-test-scope

- Status: done
- Created: 2026-10-08
- Owner: codex
- Related spec (if any): app/docs/specs/261006-1730-04-m3-ai-judgment.md

## Context

M3는 `MissedItem.summary` 필드를 추가한다. `app/src-tauri/src/summary.rs`의 `tests::boundaries_board_pipeline_and_empty`는 `MissedItem`을 구조체 리터럴로 만들므로 새 필드 값을 넣지 않으면 `cargo test`가 컴파일되지 않는다. 이 파일은 승인된 Files 표에 없다.

## What's left

- [x] 사람 승인으로 `app/src-tauri/src/summary.rs`를 M3 Files 표에 추가한다. 테스트 리터럴의 `summary: None`만 수정한다. 계산 규칙은 변경하지 않는다.

## Notes

스펙의 Don't touch는 `summary.rs` 계산 규칙을 금지한다. 필요한 변경은 테스트 생성자 호환성만이다.
