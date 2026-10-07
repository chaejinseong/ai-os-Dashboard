# M1 앱 골격과 Gmail

- Date: 2026-10-07
- Agent: codex (Jim)
- Spec: app/docs/specs/261006-1535-01-m1-app-skeleton-gmail.md
- Commits: 948de4b

## Summary

기존 Tauri 골격을 이어 M1의 다크 대시보드, 공통 데이터 모델, Gmail 읽기 전용 연결, 로컬 캐시·동기화·미응답 판정을 구현했다. M2/M3 커넥터와 AI 기능은 포함하지 않았다.

## Changes by file

### `app/.gitignore`
- Where: `빌드 산출물 제외`
- Purpose: 로컬 의존성과 빌드 결과가 커밋에 들어가지 않도록 한다.
- Method: 템플릿 제외 규칙에 src-tauri/target/을 포함한다.

### `app/package.json`
- Where: `dependencies, scripts`
- Purpose: 승인된 React/Tauri 도구와 아이콘을 설치한다.
- Method: 기존 골격을 유지하고 lucide-react만 추가한다.

### `app/index.html`
- Where: `문서 root`
- Purpose: 한국어 앱의 웹 진입점을 제공한다.
- Method: ko 언어와 앱 제목, main.tsx 진입점을 지정한다.

### `app/src/main.tsx`
- Where: `ReactDOM.createRoot`
- Purpose: React를 시작한다.
- Method: 템플릿 StrictMode 진입점을 유지한다.

### `app/src/App.tsx`
- Where: `App, refresh, sync`
- Purpose: 화면 전환과 캐시 갱신을 조정한다.
- Method: state 기반 메뉴, 이벤트 해제, 요청 세대 번호로 오래된 응답 방지, 네트워크 상태와 ⌘K를 처리한다.

### `app/src/api.ts`
- Where: `api, openItem, 타입 선언`
- Purpose: 백엔드 경계를 한 곳에 둔다.
- Method: Tauri command/event 래퍼와 Rust 대응 타입을 정의하고 HTTPS 항목 링크만 연다.

### `app/src/styles/tokens.css`
- Where: `root 토큰`
- Purpose: 승인된 다크 테마를 적용한다.
- Method: 디자인 색상, 시스템 폰트와 숫자 폭을 CSS 변수로 정의한다.

### `app/src/styles/app.css`
- Where: `shell, card-grid, 카드·목록·폼 스타일`
- Purpose: 대시보드 배치와 빈 상태를 제공한다.
- Method: 사이드바와 2열 카드, 전체 폭 일정, 최소 창 크기에 맞춘 간격을 적용한다.

### `app/src/components/Sidebar.tsx`
- Where: `Sidebar`
- Purpose: 메뉴와 소스 연결 상태를 표시한다.
- Method: 업무·알림 개수, 상태 점과 설정 이동 버튼을 렌더링한다.

### `app/src/components/TitleBar.tsx`
- Where: `TitleBar`
- Purpose: 동기화 상태를 표시한다.
- Method: 네트워크, 오류, 진행 여부와 마지막 갱신 시각으로 live/syncing/error/offline을 계산한다.

### `app/src/components/TopBar.tsx`
- Where: `TopBar`
- Purpose: 현재 화면과 검색을 제공한다.
- Method: 상위 ref 입력창과 알림 이동 버튼을 사용한다.

### `app/src/components/cards/CompletionCard.tsx`
- Where: `CompletionCard`
- Purpose: 기간 내 완료 비율을 표시한다.
- Method: 두 SVG 원으로 도넛을 그리고 데이터가 없으면 안내한다.

### `app/src/components/cards/BoardCard.tsx`
- Where: `BoardCard`
- Purpose: 마감별 업무 개수를 표시한다.
- Method: 백엔드 집계로 세 열과 비율 막대를 그린다.

### `app/src/components/cards/PipelineCard.tsx`
- Where: `PipelineCard`
- Purpose: 단계별 개수와 놓친 업무를 표시한다.
- Method: missed_by_stage 최댓값을 강조하고 수동 동기화 버튼을 둔다.

### `app/src/components/cards/MissedCard.tsx`
- Where: `MissedCard`
- Purpose: 오래된 미응답 메일을 요약한다.
- Method: 앞의 5개, 경과일, 소스별 테두리와 전체 알림 이동을 제공한다.

### `app/src/components/cards/ScheduleCard.tsx`
- Where: `ScheduleCard`
- Purpose: 오늘·내일 일정을 위한 공통 화면을 제공한다.
- Method: 백엔드 정렬 결과와 빈 상태를 렌더링하며 커넥터는 추가하지 않는다.

### `app/src/pages/Overview.tsx`
- Where: `Overview`
- Purpose: 개요 카드 5개를 배치한다.
- Method: 오늘/7일/30일 토글과 놓친 업무 배지를 조합한다.

### `app/src/pages/Connections.tsx`
- Where: `Connections, run`
- Purpose: Google 연결 흐름을 제공한다.
- Method: 클라이언트 입력 후 Keychain command 호출, 입력 초기화, 오류·진행 표시와 준비 중 서비스 카드를 둔다.

### `app/src/pages/WorkList.tsx`
- Where: `WorkList`
- Purpose: 미완료 업무를 검색한다.
- Method: 캐시에서 받은 목록 제목을 로컬에서 필터링한다.

### `app/src/pages/Alerts.tsx`
- Where: `Alerts`
- Purpose: 놓친 업무 전체를 표시한다.
- Method: 판정 이유와 소스 링크를 보여주며 M3 피드백 기능은 넣지 않는다.

### `app/src/pages/Settings.tsx`
- Where: `Settings`
- Purpose: 동기화·메일 기준을 설정한다.
- Method: 범위 제한 입력과 저장 상태를 제공한다.

### `app/src-tauri/Cargo.toml`
- Where: `dependencies, release profile`
- Purpose: 승인된 Rust 스택을 구성한다.
- Method: 이전 작업자가 남긴 명시 crate 목록과 bundled SQLite, rustls 설정을 유지한다.

### `app/src-tauri/tauri.conf.json`
- Where: `app.windows`
- Purpose: macOS 기본 창을 구성한다.
- Method: 기본 타이틀바, 1200×820 초기 크기와 1000×680 최소 크기를 적용한다.

### `app/src-tauri/capabilities/default.json`
- Where: `permissions`
- Purpose: 외부 항목과 로그인 URL을 연다.
- Method: 템플릿 core:default, opener:default 권한을 유지한다.

### `app/src-tauri/src/lib.rs`
- Where: `run`
- Purpose: 앱 상태와 명령을 연결한다.
- Method: 앱 데이터 경로에 DB를 만들고 Registry·공유 상태·주기 동기화를 등록한다.

### `app/src-tauri/src/model.rs`
- Where: `Item, SourceStatus, Settings`
- Purpose: 공통 데이터와 설정 검증을 제공한다.
- Method: serde 모델 및 설정 범위를 정의하고 테스트용 fixture를 제공한다.

### `app/src-tauri/src/connectors/mod.rs`
- Where: `Connector, Registry, registry`
- Purpose: 서비스별 구현을 공통 흐름과 분리한다.
- Method: async trait과 Arc 레지스트리에 M1 커넥터만 등록한다.

### `app/src-tauri/src/connectors/gmail.rs`
- Where: `Gmail.fetch, thread_to_item, from_parts`
- Purpose: 최근 받은편지함 스레드를 공통 Item으로 변환한다.
- Method: 최근 14일 최대 100개, 최대 5개 병렬 메타데이터 요청, From 파싱과 순수 변환 테스트를 사용한다.

### `app/src-tauri/src/oauth.rs`
- Where: `GoogleAuth, client, GOOGLE_SCOPES`
- Purpose: Google 읽기 전용 로그인을 제공한다.
- Method: loopback+PKCE+state 검증, 5분 제한, 메모리 access token과 Keychain refresh token, 만료 갱신을 구현한다.

### `app/src-tauri/src/secrets.rs`
- Where: `get, set, delete`
- Purpose: 비밀값을 macOS Keychain에만 보관한다.
- Method: 고정 service 이름과 세 함수로 접근하고 오류에 비밀값을 포함하지 않는다.

### `app/src-tauri/src/store.rs`
- Where: `Store`
- Purpose: 로컬 캐시와 설정을 보존한다.
- Method: 소스 단위 트랜잭션 교체, 상태·설정 저장, 롤백과 소스 격리 테스트를 구현한다.

### `app/src-tauri/src/sync.rs`
- Where: `AppState.sync, run`
- Purpose: 시작·주기·수동 동기화를 조정한다.
- Method: 중복 실행 잠금, 연결별 순차 fetch, 실패 시 기존 캐시 유지와 상태 이벤트를 사용한다.

### `app/src-tauri/src/missed.rs`
- Where: `missed, unanswered`
- Purpose: 미응답 메일을 판정한다.
- Method: 일수 경계, 본인 발신 제외와 오래된 순 정렬을 순수 함수로 테스트한다.

### `app/src-tauri/src/summary.rs`
- Where: `summarize`
- Purpose: 서비스와 무관한 카드 집계를 제공한다.
- Method: 로컬 날짜 기간·월~일 마감 구간·stage별 집계와 일정 정렬을 테스트한다.

### `app/src-tauri/src/commands.rs`
- Where: `command 함수들`
- Purpose: 프런트엔드에 캐시와 사용자 동작을 노출한다.
- Method: 캐시 조회, 검증된 설정 저장, 연결·해제와 동기화 작업을 조정한다.

### `app/docs/01-folder-architecture.md`
- Where: `Top-level layout`
- Purpose: 실제 M1 구조를 기록한다.
- Method: pages/styles/cards와 summary.rs를 추가한다.

### `app/docs/todo/261006-1535-01-google-cloud-setup.md`
- Where: What's left, Notes
- Purpose: 실제 계정이 필요한 수동 확인을 추적한다.
- Method: 연결 화면 명칭을 바로잡고 UI·로그인·재시작·연결 해제 확인 항목을 추가한다.

### `app/docs/specs/261006-1535-01-m1-app-skeleton-gmail.md`, `app/docs/specs/00-index.md`
- Where: Status, Reports, Done when, M1 행
- Purpose: 검증된 구현을 리뷰 단계로 넘긴다.
- Method: Implemented와 보고서 경로를 기록하고 자동 검증만 완료 표시한다. 수동 확인은 미완료로 남긴다.

### `app/docs/reports/261007-0919-01-m1-app-skeleton-gmail.md`
- Where: Changes by file, Verified, Follow-ups
- Purpose: 구현 선택과 검증 근거를 남긴다.
- Method: 파일별 변경 이유, 실제 명령 결과와 수동 확인 한계를 기록한다.

템플릿/생성 파일: `app/tsconfig.json`, `app/tsconfig.node.json`, `app/vite.config.ts`, `app/src/vite-env.d.ts`, `app/src-tauri/src/main.rs`, `app/src-tauri/build.rs`, `app/src-tauri/.gitignore`, `app/src-tauri/icons/*`, `app/package-lock.json`, `app/src-tauri/Cargo.lock`. 기존 템플릿 설정과 아이콘을 유지하며 lockfile은 도구가 생성했다.

## Verified

Rust 명령은 `CARGO_HOME=/tmp/claude-501/cargo-home`, `CARGO_NET_OFFLINE=true`로 실행했다. 초기 기본 캐시 실행은 DNS 차단으로 실패했고, 이전 작업자 캐시를 찾아 해결했다. god도 npm 의존성과 별도 Cargo 캐시를 준비했다. 실제 계정 인증은 실행하지 않았다.

- `npm install --offline --cache /tmp/claude-501/npm-cache --fetch-retries=0` → exit 0.
  ```text
  up to date, audited 29 packages in 1s
  found 0 vulnerabilities
  ```
- `cargo test --offline --manifest-path src-tauri/Cargo.toml` → exit 0.
  ```text
  Finished `test` profile [unoptimized + debuginfo] target(s) in 1m 38s
  running 6 tests
  test connectors::gmail::tests::address_only_and_invalid ... ok
  test connectors::gmail::tests::missing_subject_and_named_sender_case ... ok
  test missed::tests::boundary_sender_and_order ... ok
  test summary::tests::schedule_local_dates_sorted ... ok
  test summary::tests::boundaries_board_pipeline_and_empty ... ok
  test store::tests::replacement_is_atomic_and_source_scoped ... ok
  test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
  ```
  main과 doc-test도 실패 0. SQLite 중복 삽입 시 롤백·다른 소스 보존을 함께 확인했다.
- `cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings` → exit 0, 경고 0.
  ```text
  Finished `dev` profile [unoptimized + debuginfo] target(s) in 7.41s
  ```
- `npm run build` → exit 0.
  ```text
  > tsc && vite build
  vite v8.3.3 building client environment for production...
  ✓ 1585 modules transformed.
  ✓ built in 228ms
  ```
- `grep -rln "gmail" src-tauri/src --include=*.rs` → exit 0, 지정된 세 모듈만 출력.
  ```text
  src-tauri/src/connectors/mod.rs
  src-tauri/src/connectors/gmail.rs
  src-tauri/src/oauth.rs
  ```
- `cargo fmt --manifest-path src-tauri/Cargo.toml`, `git diff --check` → exit 0.
- `CARGO_HOME=/tmp/claude-501/cargo-home CARGO_NET_OFFLINE=true npm run tauri build` → god가 샌드박스 밖에서 재실행하여 exit 0. Jim이 `app/src-tauri/target/m1-build.log`를 확인했다.
  ```text
  ✓ 1585 modules transformed.
  ✓ built in 226ms
  Finished `release` profile [optimized] target(s) in 1m 49s
  Running bundle_dmg.sh
  Finished 2 bundles at:
  ```
  산출물: `app/src-tauri/target/release/bundle/macos/ai-os-dashboard.app` (7.17 MiB), `app/src-tauri/target/release/bundle/dmg/ai-os-dashboard_0.1.0_aarch64.dmg` (3.60 MiB). 이전 샌드박스 실행은 release 컴파일·앱 생성 후 DMG 단계에서 실패했으나, 재실행에서 해소됐다.
- 수동 확인: 미실행. Google Cloud 설정, 실제 브라우저 로그인, UI 이미지 비교, 재시작 캐시, Keychain 삭제 확인은 아래 todo에서 사람이 수행한다.

## Review

- Reviewer: Pam (claude)
- Round 1 (2026-10-07, commits 948de4b + 7e425ba): must-fix 1, suggestion 4
- Result: must-fix 남음 — 구현자 수정 후 Round 2

리뷰어 재실행 (`CARGO_HOME=/tmp/claude-501/cargo-home`, `--offline`):
`cargo test` 6 passed · `cargo clippy -- -D warnings` 경고 0 ·
`npm run build` 성공 · `grep -rln "gmail" src-tauri/src --include='*.rs'`
→ `oauth.rs`, `connectors/mod.rs`, `connectors/gmail.rs`. 보고서의 파일별
Purpose/Method는 코드와 일치한다. 수동 확인은 사람 몫으로 미완료.

### Round 1 findings

1. **must-fix** — `app/src-tauri/src/oauth.rs:115-117`: 로그인 콜백
   리스너에서 연결 하나가 실패하면 로그인 전체가 실패한다. `accept` 오류,
   5초 읽기 시간 초과, 읽기 오류가 모두 `?`로 함수 밖으로 나간다. Chrome
   등은 리다이렉트할 때 아무것도 보내지 않는 예비 TCP 연결을 함께 여는
   일이 흔하다. 그 연결이 먼저 accept되면 5초 뒤 "로그인 콜백 시간
   초과"로 끝나고, 뒤에 대기 중인 진짜 콜백은 읽히지 않는다. Done when의
   "연결 → 브라우저 로그인" 흐름이 깨질 수 있다. 수정: 연결 하나의
   accept/읽기 실패나 시간 초과는 그 소켓만 버리고 `continue`한다.
   전체 5분 제한은 그대로 둔다.
2. **suggestion** — `app/src-tauri/src/connectors/gmail.rs:108`: 스레드
   하나를 변환하지 못하면(`thread_to_item`이 `None`) Gmail 동기화 전체가
   실패하고 캐시가 계속 예전 상태로 남는다. `thread_to_item`이 `Option`을
   돌려주는 이유대로, 그 스레드만 건너뛰는 편이 낫다.
3. **suggestion** — `app/src-tauri/src/oauth.rs:79`: 토큰 갱신이
   네트워크 오류로 실패해도 "다시 로그인 필요"로 표시된다. spec은 이
   문구를 갱신 거부(`invalid_grant` 등)에만 쓴다. 오프라인일 때 사용자가
   불필요하게 다시 로그인하게 된다. 요청 실패와 서버 거부를 나눠 표시하는
   편이 낫다.
4. **suggestion** — `app/src-tauri/src/commands.rs:100`:
   `disconnect_google`이 Google 소스만이 아니라 레지스트리의 모든 커넥터
   캐시와 상태를 지운다. M1에는 Gmail뿐이라 지금은 맞지만, M2b/c에서
   Notion·GitHub를 등록하면 Google 연결 해제 때 그 캐시도 지워진다.
   M2 spec을 구현할 때 Google 소스만 지우도록 바꿔야 한다. 지금 고치지
   않으면 M2 리뷰에서 다시 확인한다.
5. **suggestion** — 가독성: `app/src/**/*.tsx` 대부분이 컴파일러 출력처럼
   한 줄에 몰려 있다(200자 넘는 줄 14개, `useState` 다섯 개가 한 줄).
   `oauth.rs`의 콜백 블록(114-126행)은 rustfmt가 포맷하지 못한 상태다.
   동작에는 문제가 없지만 리뷰와 M2·M3 수정이 어렵다. 여러 줄 JSX로
   풀고, 콜백 처리를 작은 함수로 빼서 rustfmt가 포맷하게 하는 편이 낫다.

## Follow-ups

- Google Cloud 설정과 실제 로그인·캐시 재시작·Keychain 삭제 수동 확인: app/docs/todo/261006-1535-01-google-cloud-setup.md. 실제 계정에는 접속하지 않았다.
