# Product Plan

> Fill in per project. This is the "why" and "what" — purpose and scope,
> not implementation detail (that belongs in `02-specs.md`).

## Source PRDs

- planning/docs/prd/ai-os-dashboard.md (Approved 2026-10-06) — 요구사항
  번호(FR1~17)는 이 PRD 기준이다.

## Purpose

본인 한 명이 쓰는 macOS 데스크톱 앱. Gmail·Google Calendar·Notion·GitHub
데이터를 한 화면에 모으고, 놓친 업무를 규칙과 Claude API로 골라 맨 위에
보여준다.

## Scope

- In scope for v1 (M1~M3): 화면(ui-design.md), 커넥터 4개(Gmail·Calendar·Notion·GitHub), Claude 메일 판정, 피드백·비용 기록. M1은 앱 골격, 커넥터 인터페이스, Gmail
  연결(읽기 전용), 로컬 캐시, 주기 동기화, 메일 규칙 판정, 메인 화면.
- Explicitly out of scope (for now): 쓰기 동작 전부, 여러 사용자, macOS 외
  플랫폼.

## Requirements

- Must-have: PRD의 Functional requirements 1~17, Non-functional 전부.
- Nice-to-have: 메뉴 막대 아이콘의 놓친 업무 개수 배지.

## Open questions

- 없음 (2026-10-06 모두 결정 — planning/docs/reports/261006-1720-01-open-questions-resolved.md)

## Milestones → specs

- M1 app/docs/specs/261006-1535-01-m1-app-skeleton-gmail.md
- M2a app/docs/specs/261006-1730-01-m2a-google-calendar.md
- M2b app/docs/specs/261006-1730-02-m2b-notion.md
- M2c app/docs/specs/261006-1730-03-m2c-github.md
- M3 app/docs/specs/261006-1730-04-m3-ai-judgment.md
- M4 (평가, 코드 없음) planning/docs/todo/261006-1720-01-m4-evaluation.md

화면 설계: planning/docs/design/ui-design.md
