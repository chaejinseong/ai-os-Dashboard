# Folder Architecture

> Fill this in once per project, near the start. Update it only when the
> structure actually changes — not on every task.

## Top-level layout

```
app/
  docs/                 작업 문서 (AGENTS.md 참고)
  src/                  React 프런트엔드
    components/         사이드바·타이틀·상단 바
      cards/            개요 카드 5개
    pages/              개요·연결·업무·알림·설정
    styles/             다크 테마 토큰·공통 스타일
    api.ts              Tauri command 호출 래퍼
  src-tauri/            Rust 백엔드
    src/
      lib.rs            Tauri 설정, command 등록
      commands.rs       프런트엔드에 노출하는 command
      model.rs          공통 데이터 타입 (Item 등)
      connectors/       외부 서비스별 커넥터 (mod.rs = Connector trait)
      oauth.rs          OAuth (loopback + PKCE)
      secrets.rs        Keychain 읽기/쓰기
      store.rs          SQLite 캐시
      sync.rs           주기 동기화
      ai.rs             Claude 메일 판정·비용 계산
      missed.rs         놓친 업무 규칙 판정
      summary.rs        공통 Item 기반 카드 숫자 계산
  package.json
```

M1 구현 구조. 각 Rust 모듈의 순수 함수 테스트는 같은 파일에 둔다.

## Responsibilities

- `docs/` — standards and working history (see `AGENTS.md`)
- `src/` — 화면만. 외부 API를 직접 부르지 않는다.
- `src-tauri/` — 데이터 가져오기, 저장, 판정, 비밀값 관리.

## Conventions

- File/folder naming: Rust는 snake_case, React 컴포넌트는 PascalCase.tsx.
- Where a new feature or module goes: 새 서비스 → `src-tauri/src/connectors/<service>.rs`.
- What must NOT be touched without discussion: `src-tauri/tauri.conf.json`의
  보안 설정(capabilities), `Cargo.lock`/`package-lock.json` 수동 편집.
