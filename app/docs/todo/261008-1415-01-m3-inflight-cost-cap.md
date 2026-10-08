# m3-inflight-cost-cap

- Status: open
- Created: 2026-10-08
- Owner: unassigned
- Related spec (if any): app/docs/specs/261006-1730-04-m3-ai-judgment.md

## Context

M3는 메일 후보를 최대 2건 동시에 판정하고 각 완료 후 월 누적 비용을 다시 확인한다. 이미 전송된 두 번째 요청은 첫 번째 응답으로 상한에 도달해도 비용이 발생할 수 있어 최종 합계가 상한을 소폭 넘을 수 있다.

## What's left

- [ ] 실제 사용량을 보고 상한을 엄격히 지켜야 하는지 결정한다. 필요하면 동시성·예상 비용 예약·응답 후 취소 중 한 방식을 스펙으로 정한다.

## Notes

현재 구현은 승인된 M3 스펙의 동시 2건과 매 완료 후 재확인 절차를 따른다. 테스트·개발 중 실제 Anthropic API 요청은 하지 않았다.
