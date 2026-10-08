# M2b 스펙 파일 범위 보완

- Status: done
- Created: 2026-10-08
- Owner: claude
- Related spec: app/docs/specs/261006-1730-02-m2b-notion.md

## Context

M2b 스펙은 `save_notion`·`disconnect_notion` 명령과 `settings`의 `notion.mapping` 저장을 요구한다. 그러나 허용 파일 목록에 Tauri 명령 등록이 있는 `app/src-tauri/src/lib.rs`와 `Store`의 SQLite 접근 메서드가 있는 `app/src-tauri/src/store.rs`가 없다. 현재 구조에서는 두 파일을 바꾸지 않고 해당 동작을 연결할 수 없다.

## What's left

- [x] 승인된 M2b 스펙의 파일 목록과 접근 방식에 `lib.rs`, `store.rs`를 포함할지 결정한다. 저장된 매핑을 커넥터에 전달하는 방식에 따라 `sync.rs`도 필요할 수 있다.
- [x] 결정된 범위로 스펙을 수정한 뒤 M2b 구현을 재개한다.

## Notes

`app/AGENTS.md`는 승인된 스펙에서 벗어나야 하면 중단하고 TODO에 이유를 남기도록 한다. Google 연결 해제 범위 수정은 별도 TODO로 진행 중이다.
