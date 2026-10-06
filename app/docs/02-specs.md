# Technical Specs & Constraints

> Fill in per project. This is the "how" — stack, exact commands, and
> implementation rules an agent must respect. Boundaries and verification
> defaults already live in `AGENTS.md`; only repeat them here if this
> project needs to override them.

## Stack

- Language / framework: Tauri 2 — Rust 백엔드(`src-tauri/`) + React +
  TypeScript + Vite 프런트엔드(`src/`)
- Database: SQLite 로컬 캐시 (`rusqlite`, bundled), 앱 데이터 폴더에 저장
- Key libraries: `reqwest`(rustls), `oauth2`, `keyring`, `serde`,
  `tokio`, `chrono`, `thiserror`, `tauri-plugin-opener`
- Deployment target: macOS (Apple Silicon). 배포·서명은 아직 하지 않는다.

## Commands

모두 `app/` 폴더에서 실행한다. 사전 설치: Node.js 20 이상, Rust stable,
Xcode Command Line Tools.

- Install: `npm install`
- Dev server: `npm run tauri dev`
- Test (all): `cargo test --manifest-path src-tauri/Cargo.toml`
- Test (single file): `cargo test --manifest-path src-tauri/Cargo.toml <module>::`
- Build: `npm run tauri build`
- Lint / typecheck: `npm run build` (tsc + vite) 그리고
  `cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings`

## Constraints

- 외부 서비스 API, OAuth, Claude API 호출은 전부 Rust 쪽에서 한다.
  프런트엔드는 Tauri command로 결과만 받는다.
- OAuth 토큰, OAuth 클라이언트 정보, API 키는 macOS Keychain(`keyring`)에만
  저장한다. 파일·로그·SQLite·프런트엔드에 두지 않는다.
- 외부 서비스에는 읽기 전용 권한만 요청한다 (Gmail: `gmail.readonly`).
- 새 데이터 소스는 `Connector` trait을 구현해 추가한다. 화면·판정 코드가
  특정 서비스를 직접 알면 안 된다.

## Code style

- Rust: `cargo fmt` 기본값, clippy 경고 0.
- TypeScript: `strict` 모드. 기본 템플릿의 설정을 따른다.
- 문서는 한국어, 코드·식별자·커밋 메시지는 영어.

## Project-specific boundary overrides

- Always allowed (additions):
- Ask first (additions): Google Cloud·GitHub·Notion 쪽 앱 설정 변경 (사람이 직접 한다)
- Never (additions): 외부 서비스에 쓰기 요청 보내기 (v1은 읽기 전용)
