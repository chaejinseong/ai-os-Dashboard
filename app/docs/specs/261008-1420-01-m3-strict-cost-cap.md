# m3-strict-cost-cap

- Status: Reviewed (claude/Pam, 2026-10-08; Implemented: codex, 2026-10-08)
- Created: 2026-10-08
- Author: claude (Pam)
- Implementer: codex
- Source PRD: planning/docs/prd/ai-os-dashboard.md (M3, G5) — todo app/docs/todo/261008-1415-01-m3-inflight-cost-cap.md
- Depends on: app/docs/specs/261006-1730-04-m3-ai-judgment.md (Reviewed)
- Reports: app/docs/reports/261008-1429-01-m3-strict-cost-cap.md

> Only the human sets `Approved`. Once approved, the implementer builds
> this without asking for plan approval again — so write it so nobody
> has to guess.

## Goal

AI 판정의 이번 달 누적 비용(설정의 가격 기준)이 월 상한을 절대 넘지 않는다.
이미 보낸 요청이 끝나도 넘지 않는다. 동시 2건 판정은 유지한다.

## Out of scope

- DB 스키마·설정 항목 추가 (예약은 메모리에서만, 한 번의 동기화 안에서 처리)
- Anthropic token counting API 호출 (네트워크 호출을 늘리지 않는다)
- 네트워크 오류·타임아웃 요청의 과금 여부 추적 (아래 Approach의 처리로 충분)
- M3의 다른 동작(프롬프트, 파싱, 👍/👎, 통계 계산) 변경

## Files

| File | Change | What |
|---|---|---|
| `app/src-tauri/src/ai.rs` | modify | `worst_case_cost(body, price_in, price_out)`, `fits_cap(spent, reserved, worst, cap)` 추가 + 테스트. `under_cap` 삭제 |
| `app/src-tauri/src/sync.rs` | modify | `judge_mail_candidates`의 동시 실행 루프를 `run_capped`로 분리하고 예약 방식으로 변경 + 테스트 |
| `app/src-tauri/src/commands.rs` | modify | `StatsResponse`에 `cap_reached: bool` 추가 |
| `app/src/api.ts` | modify | `Stats`에 `cap_reached: boolean` 추가 |
| `app/src/components/TopBar.tsx` | modify | 배너 조건을 `stats.cap_reached`로 변경 |

`lib.rs`, `store.rs`, `model.rs`, `missed.rs`, `summary.rs`, `Settings.tsx`는 바꾸지 않는다
(새 명령·테이블·설정 없음). Anything not listed here is off-limits without stopping to ask.

## Approach

**선택: 요청마다 최대 비용을 예약하고, 끝나면 실제 비용으로 바꾼다.**
동시 1건 + 사전 확인은 "확인 시점 누적 < 상한"만 보장하므로, $4.9999에서 보낸
마지막 1건이 상한을 넘긴다. 즉 동시 1건으로 줄여도 "최대 비용만큼 남았는지" 검사는
어차피 필요하다. 그 검사를 넣으면 동시 2건을 유지하는 비용은 메모리 변수 하나
(`reserved`)뿐이므로 이 방식이 더 작고 느려지지도 않는다. `operation` 잠금으로
동기화는 한 번에 하나라서 예약을 DB에 둘 필요가 없다.

**최대 비용 (`ai.rs`)**
- `pub fn worst_case_cost(body: &Value, price_in: f64, price_out: f64) -> f64`
  - 입력 상한 = `body["system"]` 문자열의 UTF-8 바이트 수 + `body["messages"][0]["content"]`
    문자열의 UTF-8 바이트 수 + `INPUT_OVERHEAD_TOKENS` (상수 64, 역할·형식 토큰 여유).
    Claude 토크나이저는 바이트 단위 BPE라 토큰 수 ≤ 바이트 수다. 한국어(3바이트/자)는
    크게 잡히는데, 건당 $0.001 미만이라 괜찮다.
  - 출력 상한 = `body["max_tokens"]` (150, API가 보장).
  - 반환 = `cost_usd(입력 상한, 출력 상한, price_in, price_out)`.
- `pub fn fits_cap(spent: f64, reserved: f64, worst: f64, cap: f64) -> bool` =
  `spent + reserved + worst <= cap`.
- `under_cap`은 지운다 (호출처는 `sync.rs`뿐).

**루프 (`sync.rs`)** — `judge_mail_candidates`는 후보·캐시 확인·키 확인을 지금처럼
하고, 시작 시 `spent = store.month_cost(now)` 를 한 번 읽어 `run_capped`에 넘긴다.
첫 `under_cap` 사전 검사는 지운다 (`run_capped`가 대신한다).

```rust
async fn run_capped<J, F>(
    pending: Vec<Item>, spent: f64, settings: &Settings,
    judge: J,                                  // 실제: ai::judge 호출, 테스트: 가짜
    mut on_done: impl FnMut(&Item, &ai::ResultWithUsage, f64) -> Result<(), String>,
) -> Result<f64, String>                       // 반환: 이번 실행 후 spent
where J: Fn(Item) -> F, F: Future<Output = (Item, Result<ai::ResultWithUsage, String>)> + Send + 'static
```

- 상태: `spent`, `reserved = 0.0`, `JoinSet`. 각 작업은 `(item, worst, result)`를 돌려준다.
- 채우기: `tasks.len() < 2`인 동안 다음 후보의 `worst = worst_case_cost(&ai::request_body(&settings.ai_model, &item), …)`를
  계산해 `fits_cap(spent, reserved, worst, cap)`이면 `reserved += worst` 후 spawn.
  안 맞으면 그 후보를 다시 앞에 두고(`Peekable`) 채우기를 멈춘다.
- 대기: `join_next()`가 `None`이면 끝. 완료되면 `reserved -= worst`.
  - 성공: `cost = cost_usd(실제 토큰…)`, `spent += cost` (실제 비용 그대로 — 상한
    보장은 `cost ≤ worst`에서 나온다), `on_done(&item, &result, cost)?`
    (실제: `record_judgment` + `notify`).
  - HTTP 오류 응답(상태 코드 있음): 과금 없음 → 예약만 푼다.
  - 네트워크 오류·타임아웃·`JoinError`: 과금됐을 수 있으므로 `spent += worst` (이번
    실행 안에서만, DB 기록 없음). 이 판정은 기록되지 않으니 다음 동기화에 다시 시도된다.
    이를 위해 `ai::judge`의 오류를 구분할 필요가 있으면 `Err` 문자열 대신
    `enum JudgeError { Status(u16), Network }`로 바꿔도 된다 (ai.rs 안에서만).
- 안 맞는 후보가 있고 진행 중 작업이 0이면 종료 (상한 도달). 진행 중이 있으면 완료를
  기다린 뒤 실제 비용이 예약보다 작아 남은 여유로 다시 채운다.
- 불변식: 어느 시점이든 `spent + reserved <= cap`. `cost <= worst`이므로 최종
  `spent <= cap`.

**상한 표시 (`commands.rs`, `api.ts`, `TopBar.tsx`)** — 엄격한 상한에서는 누적이
상한에 "닿지" 않으므로 기존 `month_cost_usd >= cap_usd` 배너가 뜨지 않는다.
`get_stats`에서 `cap_reached = !ai::fits_cap(stats.month_cost_usd, 0.0,
ai::worst_case_cost(&ai::request_body(&model, &빈 메일 Item), price_in, price_out), cap)`
로 계산한다 (가장 짧은 요청도 못 보내는 상태). 빈 메일 Item은 `title`·`subtitle`·`snippet`이
빈 Email Item. TopBar는 `stats.cap_reached`일 때 "AI 판정 중지 (월 상한)"을 보인다.
Settings 화면의 비용 막대는 그대로 둔다.

## Don't touch

- `ai::request_body`·`build_request`·`parse_response`·`cost_usd`의 동작, 프롬프트
- DB 스키마, `Settings` 필드, Tauri 명령 목록
- 동시 실행 수 2

## Done when

명령은 `app/`에서 실행한다. 테스트는 실제 Anthropic API를 부르지 않는다 (`judge`는 가짜 클로저).

- [x] `cargo test --manifest-path src-tauri/Cargo.toml` 통과. 추가 테스트:
  - `ai::tests`: `worst_case_cost`가 (system 바이트 + content 바이트 + 64)·가격in +
    150·가격out을 1e6으로 나눈 값과 같다; 한국어 snippet은 글자당 3바이트로 센다.
    `fits_cap`의 경계 (`spent + reserved + worst == cap` → true, 1e-12 초과 → false).
  - `sync::tests::cap_holds_with_two_concurrent_candidates` (`#[tokio::test]`):
    후보 2건, 각 `worst = W`, `cap = 1.5 × W`, `spent = 0`. 가짜 judge는 최대 토큰을
    다 쓴 결과를 돌려준다(실제 비용 = W). → 1건만 판정, `on_done` 비용 합 ≤ cap,
    가짜 judge 호출 횟수 1.
  - `sync::tests::released_reservation_admits_next_candidate`: 같은 조건에서 가짜 judge가
    `W`의 20%만 쓰면 → 2건 모두 판정, 비용 합 ≤ cap. 가짜 judge 안에서 공유 카운터로
    잰 동시 실행 수의 최대값 ≤ 2, 그리고 두 번째 spawn 시점에 `spent + reserved ≤ cap`.
  - `sync::tests::network_error_keeps_reservation`: 가짜 judge가 첫 건에 네트워크 오류 →
    두 번째 후보는 `cap = 1.5 × W`에서 판정되지 않는다.
  - `commands::tests`: `cap_reached`가 남은 여유 < 빈 요청 최대 비용일 때 true.
- [x] `cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings` 경고 0
- [x] `npm run build` 성공
- [x] `grep -rn "under_cap" src-tauri/src` 결과 없음
- [ ] 수동 확인 (키 필요, 사람): 설정에서 상한을 $0.0001로 낮추고 동기화 → AI 요청이
  나가지 않고(설정의 이번 달 비용 변화 없음) 상단 바에 "AI 판정 중지 (월 상한)",
  MISSED 메일에 "(AI 판정 없음)"이 뜬다.

## Open questions

- 없음. 상한은 설정의 가격 기준이다 — 실제 청구 가격과 설정이 다르면 보장도 그 차이만큼
  어긋난다 (기존 M3와 같음, 설정 화면 안내 문구 유지).

---
File naming: `YYMMDD-HHMM-NN-short-title.md`, same as todo/ and reports/.
