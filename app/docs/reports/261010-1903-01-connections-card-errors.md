# 연결 카드별 오류 표시

- Date: 2026-10-10
- Agent: codex (Jim)
- Spec: none — trivial fix requested after `app/docs/reports/261010-1901-01-connections-busy-set.md` review
- Commits: pending

## Summary

한 연결 카드의 작업이 다른 카드의 오류를 지우지 않도록 오류 상태와 표시를 카드별로 나눴다.

## Changes by file

### `app/src/pages/Connections.tsx`
- Where: `Connections` errors state, `setCardError`, `run`, 카드별 오류 배너
- Purpose: 공용 오류를 쓰면 다른 카드의 작업 시작 시 오류가 사라지고 여러 오류 중 마지막 것만 보인다.
- Method: 기존 `Card`를 키로 하는 `Record<Card, string>`을 두고 작업 시작·실패 때 해당 카드만 갱신한다. Google 클라이언트 확인과 GitHub 로그인 조회 오류도 각 카드에 저장하고, 오류 배너를 각 카드 안에 렌더링한다.

## Verified

- `npm run build` → exit 0; TypeScript 검사 및 Vite 빌드 성공 (1,585 modules transformed).
- `git diff --check` → exit 0.
- `awk 'length>200 {print FNR ":" length}' src/pages/Connections.tsx` → 출력 없음.
- 실제 계정으로 여러 오류를 동시에 내는 수동 검증은 하지 못했다.

## Review

- Pending Claude review.

## Follow-ups

- 실제 계정 연결 수동 확인: `app/docs/todo/261006-1730-01-notion-github-setup.md`.
