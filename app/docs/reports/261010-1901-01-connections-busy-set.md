# 연결 카드별 처리 상태 유지

- Date: 2026-10-10
- Agent: codex (Jim)
- Spec: none — trivial fix requested after `app/docs/reports/261008-1521-01-review-suggestions-cleanup.md` review
- Commits: pending

## Summary

Google 브라우저 로그인이 진행되는 동안 Notion 또는 GitHub 작업을 시작해도 Google 카드의 처리 상태와 안내가 유지되게 했다.

## Changes by file

### `app/src/pages/Connections.tsx`
- Where: `Connections` busy state, `run`, 각 카드의 입력·버튼·안내 조건
- Purpose: 다른 카드 작업이 단일 `busy` 값을 덮어 Google 로그인 표시와 버튼 잠금을 풀던 문제를 해결한다.
- Method: `Set<Card>`에 작업 중인 카드를 추가하고 해당 작업의 `finally`에서 그 카드만 제거한다. 각 카드의 렌더링 조건은 `busy.has(card)`를 사용한다.

## Verified

- `npm run build` → exit 0; TypeScript 검사 및 Vite 빌드 성공 (1,585 modules transformed).
- `git diff --check` → exit 0.
- `awk 'length>200 {print FNR ":" length}' src/pages/Connections.tsx` → 출력 없음.
- 실제 Google·Notion·GitHub 계정이 없어 동시 연결 동작은 수동 검증하지 못했다.

## Review

- Reviewer: claude (Pam), 2026-10-10, round 1
- Result: must-fix 0, suggestion 1. 스펙 없음(cleanup 리뷰 suggestion 후속, trivial).
- Re-run: `npm run build` 성공.
- 보고서 ↔ 코드 일치: ce8bc01은 `Connections.tsx`의 `busy`를 `Set<Card>`로 바꾸고, `run`이 시작 때 카드를 넣고 `finally`에서 그 카드만 뺀다(함수형 업데이트라 동시 작업끼리 덮어쓰지 않음). `run(...)` 호출 7곳과 `busy.has(...)` 조건 모두 카드 이름이 맞고 남은 `busy ===` 없음. Google 로그인 중 Notion·GitHub을 눌러도 Google 카드의 "처리 중…"·안내가 유지된다. 실제 계정 수동 확인은 미실행(보고서 기재대로).

1. `suggestion` — `run`의 `setError('')`와 공용 `error` 배너: 다른 카드 작업을 시작하면 진행 중인 카드의 이전 오류 문구도 지워지고, 두 작업이 실패하면 나중 것만 보인다. 기존부터 있던 동작이고 각 소스 오류는 카드 안 `status.error`로도 보이므로 낮은 우선순위.

## Follow-ups

- 실제 계정 연결 수동 확인: `app/docs/todo/261006-1730-01-notion-github-setup.md`.
