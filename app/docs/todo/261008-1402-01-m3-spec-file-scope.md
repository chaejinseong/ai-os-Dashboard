# m3-spec-file-scope

- Status: done
- Created: 2026-10-08
- Owner: codex
- Related spec (if any): app/docs/specs/261006-1730-04-m3-ai-judgment.md

## Context

승인된 M3 스펙의 Files 표에는 설정 데이터 모델과 상단 바 표시를 구현하는 파일이 빠져 있다. `app/AGENTS.md`에 따라 범위 확정 전에는 M3 코드를 수정하지 않는다.

## What's left

- [x] 사람이 M3 Files 표에 다음 파일을 추가할지 결정한다.
  - `app/src-tauri/src/model.rs`: 기존 `Settings`에 AI 모델·월 상한·입출력 가격 필드와 검증 추가.
  - `app/src/App.tsx`: 확장된 `Settings` 초기값을 맞추고 설정 상태를 화면에 전달.
  - `app/src/components/TopBar.tsx`: 월 상한 도달 시 "AI 판정 중지 (월 상한)" 표시.
  - `app/src/styles/app.css`: 피드백 버튼·요약·비용 막대·상한 표시 스타일.
- [x] 승인되면 스펙 Files 표를 보완한 뒤 구현한다.

## Notes

Gmail 항목의 `thread_id`, `last_message_id`, `snippet`은 이미 `connectors/gmail.rs`의 `meta`에 있어 커넥터 수정은 필요 없다. `set_feedback`에서 앱 갱신 이벤트를 보내면 `Overview.tsx` 변경 없이 카드가 갱신된다.
