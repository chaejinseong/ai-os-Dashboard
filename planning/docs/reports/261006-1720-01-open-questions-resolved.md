# open-questions-resolved

- Date: 2026-10-06
- Author: claude
- Related initiative (if any): planning/docs/prd/ai-os-dashboard.md

## Decision

PRD의 열린 질문을 모두 정했다 (chaejinseong):
- Claude API 월 상한 $5, 모델 Claude Haiku 4.5
- Google OAuth 동의 화면: 개발 중 Testing, M4 실사용 전에 게시 절차 확인
- Notion 데이터: "프로젝트 플래너" 데이터베이스
- 화면: 참고 이미지(planning/docs/design/reference-dashboard.png)의 다크
  대시보드 방향. 설계는 planning/docs/design/ui-design.md

## Why

- 비용: Haiku 4.5 가격(입력 $1 / 출력 $5, 100만 토큰당 —
  https://platform.claude.com/docs/en/about-claude/pricing)으로 메일 하루
  30건을 판정하면 월 $1 안팎으로 추정한다 [ASSUMPTION]. $5면 여유가 있다.
- OAuth: 게시 절차가 개발을 막지 않게 뒤로 미룬다. 개발 중에는 7일마다
  재로그인을 감수한다.
- Notion: 이미 쓰는 데이터베이스이고 마감일·진행 상태 속성이 판정 규칙에
  그대로 맞는다.

## Alternatives considered

- 상한 $3 / $10, 모델 Sonnet 5.5 (비용 약 2배)
- OAuth: 처음부터 게시, 계속 Testing
- 화면: 별도 디자인 시안 제작 — 사용자가 참고 이미지를 직접 제시해 생략

## Status

decided

## Follow-ups

- planning/docs/todo/261006-1720-01-m4-evaluation.md
