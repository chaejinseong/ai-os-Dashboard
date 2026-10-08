# m3-inflight-cost-cap

- Status: done
- Created: 2026-10-08
- Owner: unassigned
- Related spec (if any): app/docs/specs/261006-1730-04-m3-ai-judgment.md

## Context

M3는 메일 후보를 최대 2건 동시에 판정하고 각 완료 후 월 누적 비용을 다시 확인한다. 이미 전송된 두 번째 요청은 첫 번째 응답으로 상한에 도달해도 비용이 발생할 수 있어 최종 합계가 상한을 소폭 넘을 수 있다.

## What's left

- [x] 사람 승인으로 요청별 최대 비용 예약 방식을 선택하고 구현했다.

## Notes

구현 스펙: app/docs/specs/261008-1420-01-m3-strict-cost-cap.md (Implemented). 보고서: app/docs/reports/261008-1429-01-m3-strict-cost-cap.md.

현재 구현은 동시 2건을 유지하며 요청 전에 최대 비용을 예약한다. 테스트·개발 중 실제 Anthropic API 요청은 하지 않았다.
