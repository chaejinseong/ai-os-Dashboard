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

- Pending Claude review.

## Follow-ups

- 실제 계정 연결 수동 확인: `app/docs/todo/261006-1730-01-notion-github-setup.md`.
