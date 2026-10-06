# m1-app-skeleton-gmail

- Status: Draft
- Created: 2026-10-06 (rev. 2026-10-06 17:30 — 화면 설계 반영)
- Author: claude
- Implementer: codex
- Source PRD: planning/docs/prd/ai-os-dashboard.md (M1)
- Reports: app/docs/reports/<file>.md (filled in by the implementer)

> Only the human sets `Approved`. Once approved, the implementer builds
> this without asking for plan approval again — so write it so nobody
> has to guess.

## Goal

Tauri 앱을 실행하면 planning/docs/design/ui-design.md 의 화면 틀(사이드바,
상단 바, 개요 카드 5개, 연결·업무·알림·설정 화면)이 뜬다. Google 계정을
연결하면 Gmail 받은편지함의 최근 스레드가 로컬에 캐시되고 15분마다 갱신되며,
"2일 넘게 답장하지 않은 메일"이 MISSED 카드와 알림 화면에 뜬다. 카드 숫자는
모든 서비스에 공통인 `Item`에서 계산하므로, M2 커넥터가 들어오면 화면 코드
수정 없이 채워진다.

## Out of scope

- Google Calendar·Notion·GitHub 커넥터 (M2a~c). 단, Google 로그인은 이번에
  Calendar 읽기 권한까지 함께 요청한다 (다시 로그인하지 않도록).
- Claude API, 요약, 👍/👎 피드백, 비용 기록 (M3)
- 모든 쓰기 동작, 앱 서명·배포, 라이트 테마, 메뉴 막대 배지

## Files

| File | Change | What |
|---|---|---|
| create-tauri-app가 만드는 파일 (`package.json`, `index.html`, `vite.config.ts`, `tsconfig*.json`, `src-tauri/build.rs`, `src-tauri/icons/`, `public/` 등) | new | 템플릿 그대로. 쓰지 않는 예제 코드는 지워도 된다 |
| `app/.gitignore` | new | `node_modules/`, `dist/`, `src-tauri/target/` |
| `app/src/main.tsx`, `app/src/App.tsx` | new | 진입점, 레이아웃(사이드바 + 본문), 화면 전환 |
| `app/src/styles/tokens.css`, `app/src/styles/app.css` | new | ui-design.md의 색 토큰과 공통 스타일 |
| `app/src/api.ts` | new | Tauri command·이벤트 래퍼 + TS 타입 (Rust 타입과 같은 모양) |
| `app/src/components/Sidebar.tsx`, `TitleBar.tsx`, `TopBar.tsx` | new | 메뉴·SOURCES 점, live 상태, 검색 ⌘K·알림 숫자 |
| `app/src/components/cards/CompletionCard.tsx`, `BoardCard.tsx`, `PipelineCard.tsx`, `MissedCard.tsx`, `ScheduleCard.tsx` | new | 개요 카드 |
| `app/src/pages/Overview.tsx`, `Connections.tsx`, `WorkList.tsx`, `Alerts.tsx`, `Settings.tsx` | new | 사이드바 메뉴별 화면 |
| `app/src-tauri/Cargo.toml` | new | 아래 Approach의 crate |
| `app/src-tauri/tauri.conf.json`, `app/src-tauri/capabilities/default.json` | new | 템플릿 기본 + opener 권한, 창 최소 크기 1000×680 |
| `app/src-tauri/src/main.rs`, `lib.rs` | new | 앱 시작, 상태·command 등록, 동기화 루프 시작 |
| `app/src-tauri/src/model.rs` | new | `Item`, `ItemKind`, `Stage`, `SourceStatus` |
| `app/src-tauri/src/connectors/mod.rs` | new | `Connector` trait, `Registry` |
| `app/src-tauri/src/connectors/gmail.rs` | new | Gmail 커넥터 |
| `app/src-tauri/src/oauth.rs` | new | Google OAuth (loopback + PKCE), 토큰 갱신 |
| `app/src-tauri/src/secrets.rs` | new | Keychain 래퍼 |
| `app/src-tauri/src/store.rs` | new | SQLite 캐시 |
| `app/src-tauri/src/sync.rs` | new | 주기·수동 동기화 |
| `app/src-tauri/src/missed.rs` | new | 놓친 업무 규칙 |
| `app/src-tauri/src/summary.rs` | new | 카드 숫자 계산 |
| `app/src-tauri/src/commands.rs` | new | 프런트엔드용 command |
| `app/docs/01-folder-architecture.md` | modify | 실제 구조와 다르면 갱신 |

Anything not listed here is off-limits without stopping to ask.

## Approach

**0. 골격** — `app/`에는 이미 `docs/`, `README.md`, `AGENTS.md`,
`CLAUDE.md`가 있다. 임시 폴더에서 `npm create tauri-app@latest`
(React, TypeScript, npm)로 만든 뒤 파일을 `app/`으로 옮긴다. 기존 파일은
덮어쓰지 않는다. 프런트엔드 추가 의존성: `lucide-react` 하나 (라우터·상태관리·
CSS 프레임워크는 쓰지 않는다. 화면 전환은 `App.tsx`의 state 하나로 한다).

**1. 데이터 모델 (`model.rs`)**
- `Item { id: String, source: String, kind: ItemKind, title: String,
  subtitle: Option<String>, url: String, timestamp: DateTime<Utc>,
  due: Option<DateTime<Utc>>, done: bool, stage: Option<Stage>,
  meta: serde_json::Value }`
- `ItemKind`: `Email`, `Event`, `Task`, `PullRequest`, `Issue`
- `Stage`: `Requested`, `InProgress`, `Review`, `Done`
- `SourceStatus { source, connected: bool, syncing: bool,
  last_synced: Option<DateTime<Utc>>, error: Option<String> }`
- 메일 `meta`: `{ thread_id, last_message_id, last_from_me: bool, snippet }`.
  `snippet`은 저장만 한다 (M3의 AI 입력용). M1에서 메일의 `stage`는 `None`,
  `done`은 `false`.

**2. 커넥터 (`connectors/mod.rs`)**
- `#[async_trait] trait Connector: Send + Sync { fn id(&self) -> &'static str;
  fn label(&self) -> &'static str; async fn is_connected(&self) -> bool;
  async fn fetch(&self) -> Result<Vec<Item>, ConnectorError>; }`
- 연결/해제는 인증 방식마다 달라서 trait에 넣지 않는다. Google 계정 연결은
  `oauth.rs`가 담당하고, Gmail(M2a에서 Calendar도)은 Google 계정이 연결돼
  있으면 `is_connected() == true`다.
- `Registry`는 `Vec<Arc<dyn Connector>>`. `sync.rs`, `commands.rs`,
  `missed.rs`, `summary.rs`는 trait과 `Item`만 안다.

**3. OAuth (`oauth.rs`)** — Google "Desktop app" 클라이언트 + loopback
redirect + PKCE (`oauth2` crate).
- `127.0.0.1`의 빈 포트에 임시 HTTP 리스너를 열고, 인증 URL을
  `tauri-plugin-opener`로 기본 브라우저에서 연다. 콜백으로 code를 받아 토큰으로
  교환한 뒤 리스너를 닫는다. 콜백 페이지에는 "앱으로 돌아가세요"를 보여준다.
  5분 안에 콜백이 없으면 실패로 처리한다.
- scope 상수 `GOOGLE_SCOPES`: `https://www.googleapis.com/auth/gmail.readonly`,
  `https://www.googleapis.com/auth/calendar.readonly`.
  `access_type=offline`, `prompt=consent`로 refresh token을 받는다.
- 클라이언트 ID/secret은 연결 화면에서 사용자가 입력하고 Keychain에 저장한다.
  코드·설정 파일·`.env`에 넣지 않는다.
- access token이 만료되면 refresh token으로 갱신한다. 갱신이 실패하면
  (`invalid_grant` 등) Google 소스들을 `error = "다시 로그인 필요"`로 표시한다.
  Testing 상태에서는 7일마다 이렇게 된다.
- `disconnect_google()`: refresh token을 Keychain에서 지우고, Google 소스의
  캐시 항목을 삭제한다. 클라이언트 ID/secret은 남긴다.

**4. Keychain (`secrets.rs`)** — `keyring` crate. service
`com.chaejinseong.ai-os-dashboard`. account: `google.client_id`,
`google.client_secret`, `google.refresh_token` (M2·M3가 `notion.token`,
`github.token`, `anthropic.api_key`를 추가한다). access token은 메모리에만 둔다.
`get/set/delete(account)` 세 함수만 둔다.

**5. Gmail 커넥터 (`connectors/gmail.rs`)** — REST를 `reqwest`로 직접 호출.
- 내 주소: `GET gmail/v1/users/me/profile` → `emailAddress` (메모리 캐시).
- 목록: `GET gmail/v1/users/me/threads?q=in:inbox newer_than:14d&maxResults=100`.
- 각 스레드: `GET gmail/v1/users/me/threads/{id}?format=metadata&metadataHeaders=From&metadataHeaders=Subject&metadataHeaders=Date`.
  동시 요청은 최대 5개.
- 변환은 HTTP와 분리된 순수 함수 `fn thread_to_item(json: &Value, my_email: &str) -> Option<Item>`:
  title = 마지막 메시지의 Subject (없으면 "(제목 없음)"), subtitle = 마지막
  메시지 From의 이름(없으면 주소), timestamp = 마지막 메시지 `internalDate`,
  url = `https://mail.google.com/mail/u/0/#inbox/{threadId}`,
  `last_from_me` = 마지막 메시지 From 주소가 내 주소와 같은지 (`"이름 <주소>"`
  형식 파싱, 대소문자 무시). `snippet` = 마지막 메시지 `snippet`.

**6. 캐시 (`store.rs`)** — 앱 데이터 폴더의 `cache.sqlite`.
- 테이블 `items(source, id, json, PRIMARY KEY(source,id))`,
  `source_status(source PRIMARY KEY, json)`, `settings(key PRIMARY KEY, value)`.
- `replace_items(source, items)`: 해당 source의 행을 한 트랜잭션에서 통째로 교체.
- settings 기본값: `sync_minutes=15`, `mail_unanswered_days=2`.

**7. 동기화 (`sync.rs`)** — 앱 시작 직후 1회, 이후 `sync_minutes`마다, 그리고
`sync_now` 호출 시. 연결된 커넥터를 순서대로 `fetch` → `replace_items` →
`source_status` 갱신 → `dashboard-updated` 이벤트. 이미 실행 중이면 새 요청은
건너뛴다. 오류가 나면 캐시는 그대로 두고 `error`만 기록한다.

**8. 놓친 업무 (`missed.rs`)** — 순수 함수
`fn missed(items: &[Item], now: DateTime<Utc>, rules: &Rules) -> Vec<MissedItem>`.
- `MissedItem { item: Item, reason: String, since: DateTime<Utc> }`.
  `since` = 놓친 상태가 된 기준 시각 (경과일 표시에 쓴다).
- M1 규칙 (메일): `kind == Email` && `!last_from_me` &&
  `now - timestamp >= mail_unanswered_days`. since = timestamp,
  reason = "{N}일째 답장 안 함".
- 정렬: since 오래된 순. 규칙은 `fn` 하나씩 나눠 두고, M2·M3가 규칙을 추가한다.

**9. 카드 계산 (`summary.rs`)** — 순수 함수
`fn summarize(items: &[Item], missed: &[MissedItem], now: DateTime<Local>, period_days: u32) -> Summary`.
ui-design.md "카드별 데이터"를 그대로 구현한다:
- `completion { done, left }`: `kind == Task`이고 `due`가 기간 안인 항목
- `board { today, this_week, later }`: `kind ∈ {Task, Issue}` && `!done`, 마감 기준
  (주는 월~일, 로컬 시간)
- `pipeline { requested, in_progress, review, done, missed_by_stage }`: `stage` 기준.
  `Done`은 `due`가 기간 안인 것만 센다
- `schedule`: `kind == Event`이고 시작이 오늘·내일(로컬)인 항목, 시간순
- `missed_count`, `open_work_count` (`kind ∈ {Task, Issue, PullRequest}` && `!done`)
M1에서는 Task·Event·PR 데이터가 없으므로 대부분 0이다. 카드는 0일 때 빈 상태
문구를 보여준다 (예: COMPLETION "Notion을 연결하면 표시됩니다").

**10. Command (`commands.rs`)**
- `get_dashboard(period_days) -> { summary, missed, statuses }` (캐시에서만 읽음)
- `get_items(filter) -> Vec<Item>` (업무 화면용: 완료 안 된 Task·Issue·PR)
- `sync_now()`, `connect_google()`, `disconnect_google()`,
  `save_google_client(client_id, client_secret)`, `has_google_client() -> bool`
- `get_settings()`, `save_settings(...)`

**11. 화면** — planning/docs/design/ui-design.md 를 따른다.
- 시작하면 `get_dashboard`로 캐시를 즉시 그리고, `dashboard-updated` 이벤트가
  오면 다시 부른다. 기간 토글 기본값은 "7일".
- COMPLETION 도넛은 SVG 원 두 개(배경, 진행)로 그린다. 차트 라이브러리는 쓰지 않는다.
- 타이틀바는 macOS 기본 타이틀바를 쓰고, 그 아래 바에 "✳ AI 운영체제"와 live
  상태를 둔다. (커스텀 창 테두리는 하지 않는다.)
- 검색 ⌘K: 입력창에 포커스, 업무 화면으로 이동해 제목을 필터링한다.
- 연결 화면: Google 카드(클라이언트 ID/secret 입력 → 연결/해제, Gmail·Calendar
  상태). Notion·GitHub 카드는 "준비 중" 자리만 둔다.
- 설정 화면: 동기화 주기(분), 메일 미응답 기준(일).
- 항목 클릭 = opener로 `url` 열기.

**선택하지 않은 대안**: Gmail 공식 Rust SDK(`google-gmail1`)는 의존성이 커서
쓰지 않는다. OAuth 클라이언트 정보를 `.env`에 두는 방식은 비밀값 파일 규칙
때문에 쓰지 않는다. 카드 숫자를 프런트엔드에서 계산하면 TS 테스트 환경이 따로
필요해서, Rust(`summary.rs`)에서 계산하고 cargo 테스트로 검증한다.

## Don't touch

- `app/docs/` 중 이 spec, report, todo, `01-folder-architecture.md` 외의 파일
- `app/AGENTS.md`, `app/CLAUDE.md`, `app/README.md`, `planning/` 전체
- 실제 Gmail 계정에 쓰기 요청 (읽기 전용 API만)

## Done when

- [ ] `npm install` 성공
- [ ] `cargo test --manifest-path src-tauri/Cargo.toml` 통과. 최소 테스트:
  - `missed.rs`: 2일 경계(1일 23시간 → 제외, 2일 → 포함), `last_from_me=true` 제외, 정렬
  - `gmail.rs`: `thread_to_item` — Subject 없음, From이 `"이름 <주소>"` 형식,
    내 주소의 대소문자가 다른 경우
  - `summary.rs`: 기간 경계(7일 토글에서 6일 전 0시 포함, 7일 전 23:59 제외),
    BOARD의 오늘(지난 마감 포함)/이번 주(일요일 23:59까지)/나중에(마감 없음 포함),
    PIPELINE stage별 개수와 `missed_by_stage`, 항목 0개일 때
  - `store.rs`: 인메모리 SQLite로 `replace_items` 교체·조회
- [ ] `cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings` 경고 0
- [ ] `npm run build` 성공
- [ ] `grep -rln "gmail" src-tauri/src --include=*.rs` 결과가
  `connectors/gmail.rs`, `connectors/mod.rs`(레지스트리 생성), `oauth.rs`(scope 상수)뿐이다
- [ ] 수동 확인 (사람, `npm run tauri dev`, Google Cloud 설정 후):
  - 참고 이미지와 같은 배치의 다크 화면이 뜬다
  - 연결 화면에서 클라이언트 정보 입력 → 연결 → 브라우저 로그인(Gmail·Calendar
    읽기 권한 요청 확인) → MISSED 카드와 알림 화면에 미응답 메일이 뜬다
  - 앱을 껐다 켜면 캐시로 바로 뜬다
  - 연결 해제 후 `security find-generic-password -s com.chaejinseong.ai-os-dashboard -a google.refresh_token`
    가 실패한다

## Open questions

- 없음. Google Cloud 설정은 사람이 한다 —
  app/docs/todo/261006-1535-01-google-cloud-setup.md. 설정 없이도 코드와
  자동 테스트는 완료할 수 있고, 수동 확인만 설정 후에 한다.

---
File naming: `YYMMDD-HHMM-NN-short-title.md`, same as todo/ and reports/.
