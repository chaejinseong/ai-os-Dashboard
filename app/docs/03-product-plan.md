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

- In scope for the current phase (M1): 앱 골격, 커넥터 인터페이스, Gmail
  연결(읽기 전용), 로컬 캐시, 주기 동기화, 메일 규칙 판정, 메인 화면.
- Explicitly out of scope (for now): 쓰기 동작 전부, 여러 사용자, macOS 외
  플랫폼, M2~M4 (Calendar·Notion·GitHub, Claude API, 피드백, 비용 기록).

## Requirements

- Must-have: PRD의 Functional requirements 1~17, Non-functional 전부.
- Nice-to-have: 메뉴 막대 아이콘의 놓친 업무 개수 배지.

## Open questions

- Claude API 월 비용 상한 (M3 전에 필요)
- Google OAuth 동의 화면 게시 여부 (M1은 Testing 상태로 진행)
- "프로젝트"로 볼 Notion 데이터베이스 (M2 전에 필요)
