# m1-app-skeleton-gmail

- Status: Draft
- Created: 2026-10-06
- Author: claude
- Implementer: codex
- Source PRD: planning/docs/prd/ai-os-dashboard.md (M1)
- Reports: app/docs/reports/<file>.md (filled in by the implementer)

> Only the human sets `Approved`. Once approved, the implementer builds
> this without asking for plan approval again — so write it so nobody
> has to guess.

## Goal

Tauri 앱을 실행하면 Gmail을 연결할 수 있다. 연결 후 받은편지함의 최근
스레드가 로컬에 캐시되고 15분마다 갱신되며, "2일 넘게 답장하지 않은 메일"이
메인 화면 맨 위 "놓친 업무"에 뜬다. 커넥터 구조는 M2에서 Calendar·Notion·
GitHub를 화면·판정 코드 수정 없이 붙일 수 있는 형태다.

## Out of scope

- Google Calendar·Notion·GitHub 커넥터 (M2)
- Claude API 호출, 요약, 👍/👎 피드백, 비용 기록 (M3)
- 메일 쓰기·답장·라벨 변경 등 모든 쓰기 동작
- 앱 서명·배포, 메뉴 막대 배지, 오프라인 이외의 에러 화면 디자인

## Files

| File | Change | What |
|---|---|---|
| `app/package.json`, `app/index.html`, `app/vite.config.ts`, `app/tsconfig*.json` | new | create-tauri-app (React + TypeScript) 기본 파일 |
| create-tauri-app가 만드는 그 밖의 파일 (`src-tauri/build.rs`, `src-tauri/icons/`, `src/*.css`, `public/` 등) | new | 템플릿 그대로. 쓰지 않는 예제 코드는 지워도 된다 |
| `app/.gitignore` | new | `node_modules/`, `dist/`, `src-tauri/target/` |
| `app/src/main.tsx` | new | 앱 진입점 |
| `app/src/App.tsx` | new | 메인 화면 레이아웃 |
| `app/src/api.ts` | new | Tauri command 호출 래퍼 + TS 타입 |
| `app/src/components/MissedList.tsx` | new | 놓친 업무 목록 |
| `app/src/components/SourcePanel.tsx` | new | 서비스별 요약 + 마지막 동기화 시각 + 오류 |
| `app/src/components/Settings.tsx` | new | Gmail 연결/해제, OAuth 클라이언트 정보 입력, 판정 기준값 |
| `app/src-tauri/Cargo.toml` | new | 아래 Approach의 crate |
| `app/src-tauri/tauri.conf.json`, `app/src-tauri/capabilities/default.json` | new | 템플릿 기본 + opener 권한 |
| `app/src-tauri/src/main.rs`, `lib.rs` | new | 앱 시작, 상태·command 등록, 동기화 루프 시작 |
| `app/src-tauri/src/model.rs` | new | `Item`, `ItemKind`, `SourceStatus` |
| `app/src-tauri/src/connectors/mod.rs` | new | `Connector` trait, 커넥터 레지스트리 |
| `app/src-tauri/src/connectors/gmail.rs` | new | Gmail 커넥터 |
| `app/src-tauri/src/oauth.rs` | new | Google OAuth (loopback + PKCE), 토큰 갱신 |
| `app/src-tauri/src/secrets.rs` | new | Keychain 래퍼 |
| `app/src-tauri/src/store.rs` | new | SQLite 캐시 |
| `app/src-tauri/src/sync.rs` | new | 15분 주기 + 수동 동기화 |
| `app/src-tauri/src/missed.rs` | new | 놓친 업무 규칙 |
| `app/src-tauri/src/commands.rs` | new | 프런트엔드용 command |
| `app/docs/01-folder-architecture.md` | modify | 실제 구조와 다르면 갱신 |

Anything not listed here is off-limits without stopping to ask.

## Approach

**0. 골격** — `app/`에는 이미 `docs/`, `README.md`, `AGENTS.md`,
`CLAUDE.md`가 있다. 임시 폴더에서 `npm create tauri-app@latest`
(React, TypeScript, npm)로 만든 뒤 파일을 `app/`으로 옮긴다. 기존 파일은
덮어쓰지 않는다.

**1. 데이터 모델 (`model.rs`)**
- `Item { id: String, source: String, kind: ItemKind, title: String,
  subtitle: Option<String>, url: String, timestamp: DateTime<Utc>,
  due: Option<DateTime<Utc>>, meta: serde_json::Value }`
- `ItemKind`: `Email`, `Event`, `Task`, `PullRequest`, `Issue` (M1은 Email만 사용)
- `SourceStatus { source, connected: bool, last_synced: Option<DateTime>,
  error: Option<String> }`
- 메일의 `meta`: `{ thread_id, last_from_me: bool, snippet }`.
  `snippet`은 저장만 하고 M1 화면에는 쓰지 않는다 (M3의 AI 입력용).

**2. 커넥터 (`connectors/mod.rs`)**
- `#[async_trait] trait Connector: Send + Sync { fn id(&self) -> &'static str;
  async fn is_connected(&self) -> bool; async fn connect(&self, app) -> Result<()>;
  async fn disconnect(&self) -> Result<()>; async fn fetch(&self) -> Result<Vec<Item>>; }`
- `Registry`는 `Vec<Arc<dyn Connector>>`. `sync.rs`, `commands.rs`,
  `missed.rs`는 trait과 `Item`만 안다. `gmail` 이름을 쓰는 곳은
  `connectors/gmail.rs`와 레지스트리 생성 한 줄뿐이어야 한다.

**3. OAuth (`oauth.rs`)** — Google "Desktop app" 클라이언트 + loopback
redirect + PKCE (`oauth2` crate).
- `127.0.0.1`의 빈 포트에 임시 HTTP 리스너를 열고, 인증 URL을
  `tauri-plugin-opener`로 기본 브라우저에서 연다. 콜백에서 code를 받아 토큰으로
  교환한 뒤 리스너를 닫는다. 5분 안에 콜백이 없으면 실패로 처리한다.
- scope: `https://www.googleapis.com/auth/gmail.readonly` 하나.
  `access_type=offline`, `prompt=consent`로 refresh token을 받는다.
- 클라이언트 ID/secret은 Settings 화면에서 사용자가 입력하고 Keychain에
  저장한다. 코드·설정 파일·`.env`에 넣지 않는다.
- access token이 만료되면 refresh token으로 갱신한다. 갱신이 실패하면
  (`invalid_grant` 등) 해당 소스를 `connected=false`, `error="다시 로그인 필요"`로
  표시한다. Testing 상태에서는 7일마다 이렇게 된다 (PRD Technical considerations).

**4. Keychain (`secrets.rs`)** — `keyring` crate. service
`com.chaejinseong.ai-os-dashboard`, account는
`google.client_id`, `google.client_secret`, `google.refresh_token`.
access token은 메모리에만 둔다.

**5. Gmail 커넥터 (`connectors/gmail.rs`)** — REST를 `reqwest`로 직접 호출.
- 내 주소: `GET users/me/profile` → `emailAddress` (연결 시 1회, 메모리 캐시).
- 목록: `GET users/me/threads?q=in:inbox newer_than:14d&maxResults=100`.
- 각 스레드: `GET users/me/threads/{id}?format=metadata&metadataHeaders=From&metadataHeaders=Subject&metadataHeaders=Date`.
  동시 요청은 최대 5개.
- `Item` 변환: title = 마지막 메시지의 Subject, subtitle = 마지막 메시지의
  From, timestamp = 마지막 메시지의 `internalDate`,
  url = `https://mail.google.com/mail/u/0/#inbox/{threadId}`,
  `last_from_me` = 마지막 메시지 From 주소가 내 주소와 같은지 (대소문자 무시).

**6. 캐시 (`store.rs`)** — 앱 데이터 폴더의 `cache.sqlite`.
- 테이블 `items(source, id, json, updated_at, PRIMARY KEY(source,id))`,
  `source_status(source PRIMARY KEY, json)`, `settings(key PRIMARY KEY, value)`.
- 동기화 결과로 해당 source의 items를 통째로 교체한다 (한 트랜잭션).

**7. 동기화 (`sync.rs`)** — 앱 시작 직후 1회, 이후 `settings.sync_minutes`
(기본 15)마다. 연결된 커넥터를 순서대로 `fetch` → 저장 → `source_status`
갱신 → 프런트엔드에 `items-updated` 이벤트. 이미 실행 중이면 새 요청은 건너뛴다.
네트워크 오류면 캐시는 그대로 두고 `error`만 기록한다.

**8. 놓친 업무 (`missed.rs`)** — 순수 함수
`fn missed(items: &[Item], now: DateTime<Utc>, rules: &Rules) -> Vec<MissedItem>`.
- 메일 규칙: `kind == Email` && `!last_from_me` &&
  `now - timestamp >= rules.mail_unanswered_days` (기본 2일).
- `MissedItem { item, reason: String }`. reason 예: "3일째 답장 안 함".
- 오래된 순으로 정렬.

**9. Command (`commands.rs`)**: `get_dashboard() -> { missed, items_by_source,
statuses }` (캐시에서만 읽음), `sync_now()`, `connect_source(id)`,
`disconnect_source(id)`, `get_settings()`, `save_settings(...)`,
`save_google_client(client_id, client_secret)`.

**10. 화면** — 앱 시작 시 `get_dashboard`로 캐시를 즉시 그리고, `items-updated`
이벤트를 받으면 다시 그린다. 순서: 상단 바(새로고침 버튼, 설정) →
"놓친 업무" (`MissedList`) → 서비스별 패널 (`SourcePanel`: 최근 메일 20개,
마지막 동기화 시각, 오류). 항목을 클릭하면 opener로 `url`을 연다.
설정: Google 클라이언트 ID/secret 입력, Gmail 연결/해제, 동기화 주기(분),
메일 미응답 기준(일). 스타일은 기본 CSS면 된다.

**선택하지 않은 대안**: Gmail 공식 Rust SDK(`google-gmail1`)는 의존성이
커서 쓰지 않는다. OAuth 클라이언트 정보를 `.env`에 두는 방식은 비밀값
파일 규칙 때문에 쓰지 않는다.

## Don't touch

- `app/docs/` 중 이 spec, report, todo, `01-folder-architecture.md` 외의 파일
- `app/AGENTS.md`, `app/CLAUDE.md`, `app/README.md`, `planning/` 전체
- 실제 Gmail 계정에 쓰기 요청 (읽기 전용 API만)

## Done when

- [ ] `npm install` 성공
- [ ] `cargo test --manifest-path src-tauri/Cargo.toml` 통과. 최소 테스트:
  - `missed.rs`: 2일 경계(1일 23시간 → 제외, 2일 → 포함), `last_from_me=true` 제외, 정렬
  - `gmail.rs`: 고정 JSON(thread metadata 응답)을 `Item`으로 바꾸는 변환 함수
    (Subject 없음, From에 이름+주소 형식, 대소문자 다른 내 주소 포함)
  - `store.rs`: 인메모리 SQLite로 교체 저장·조회
- [ ] `cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings` 경고 0
- [ ] `npm run build` 성공
- [ ] `grep -rn "gmail" src-tauri/src --include=*.rs`의 결과가
  `connectors/gmail.rs`와 레지스트리 생성 한 줄뿐이다 (oauth scope 상수 제외)
- [ ] 수동 확인 (사람, `npm run tauri dev`): 설정에서 클라이언트 정보 입력 →
  Gmail 연결 → 브라우저 로그인 → 메일 목록과 놓친 업무가 뜬다 → 앱을 껐다
  켜면 캐시로 바로 뜬다 → 연결 해제하면 Keychain의 refresh token이 지워진다
  (`security find-generic-password -s com.chaejinseong.ai-os-dashboard -a google.refresh_token`
  가 실패)

## Open questions

- 없음 (M1 기준). Google Cloud 설정은 사람이 해야 한다 —
  app/docs/todo/261006-1535-01-google-cloud-setup.md. 이 설정이 없어도 코드와
  자동 테스트는 완료할 수 있고, 수동 확인만 설정 후에 한다.

---
File naming: `YYMMDD-HHMM-NN-short-title.md`, same as todo/ and reports/.
