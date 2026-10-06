# v1-scope-decisions

- Date: 2026-10-06
- Author: claude
- Related initiative (if any): planning/docs/prd/ai-os-dashboard.md

## Decision

v1 범위를 정했다 (chaejinseong): 첫 연동은 Gmail, Google Calendar, Notion,
GitHub. 앱 내 AI는 Claude API. 메일은 제목·보낸 사람·본문 앞부분만 AI로
보낸다. 앱은 Tauri로 만든다.

## Why

- 네 서비스: 본인이 업무 현황을 확인하는 곳들이다.
- Claude API: 요약·판정 품질과 API 사용 편의.
- 메일 일부만 전송: 판정에는 대부분 충분하고, 외부로 나가는 개인 데이터와
  비용을 줄인다.
- Tauri: macOS 우선이지만 Windows·Linux로 확장하기 쉽고 Electron보다 가볍다.

## Alternatives considered

- AI: OpenAI API, 교체 가능한 구조, 로컬 모델(Ollama 등)
- 메일 범위: 본문 전체(정확하지만 비용·노출 증가), 메타데이터만(규칙 판정 위주)
- 기술: Electron(무거움), SwiftUI(macOS 외 확장이 어려움)

## Status

decided

## Follow-ups

- planning/docs/prd/ai-os-dashboard.md 의 Open questions
