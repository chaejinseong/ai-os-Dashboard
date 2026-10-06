# Context

> Fill in once per product/team. Update only when something durable
> actually changes — this is background every planning doc can assume,
> not a place to log individual decisions (those go in `docs/reports/`).

## Product / service

**AI 운영체제 대시보드 (AI OS Dashboard)** — 이메일, 일정, 프로젝트 등
흩어진 업무 정보를 한 화면에 모으고, AI가 자동으로 정리해 실시간 업무
현황과 놓친 업무를 한눈에 보여주는 개인용 데스크톱 앱.

## Target users / personas

- Primary user(s): 개발자 본인 1명 (개인 도구). 다중 사용자·가입 기능 없음.
- Their main goal / job-to-be-done: 여러 앱을 돌아다니지 않고, 지금 해야
  할 일과 놓친 일을 한곳에서 파악한다.
- Known pain points: [ASSUMPTION — 본인 경험 기반, 정량 데이터 없음]
  - 메일·일정·프로젝트 도구가 분리돼 있어 매번 각각 확인해야 함
  - 답장해야 할 메일, 다가오거나 지난 마감을 놓침

## Competitive landscape

- Direct alternatives: [TODO — 조사 전. 출처 없이 채우지 말 것]
- What we do differently: 본인 워크플로에 맞춘 커스터마이즈, 데이터 소스를
  자유롭게 추가할 수 있는 구조, AI의 "놓친 업무" 판정.

## Constraints

- Business (budget, timeline, team size): 1인 개인 프로젝트. 예산·일정 미정.
  구현은 Codex, 기획·명세·리뷰는 Claude (root `AGENTS.md`).
- Technical:
  - 1차 플랫폼은 **macOS 데스크톱 앱**. 이후 다른 플랫폼으로 확장할 수
    있도록 설계 (대상 플랫폼은 미정).
  - 데이터 소스(메일·일정·프로젝트 도구)는 **교체·추가 가능한 커넥터
    구조**로 설계. 첫 연동 대상은 미정 — `docs/todo/` 참고.
  - 각 서비스는 앱이 직접 공식 API + OAuth로 연결 (Claude의 커넥터는
    이 앱에서 쓸 수 없음).
- Legal / compliance: 본인 데이터만 다룸. 메일 등 개인 데이터를 외부 AI
  API로 보낼 때 어떤 범위를 보낼지는 PRD에서 정한다.

## Brand / voice (if relevant to planning docs)

- Tone: 문서는 한국어, 코드·파일명·식별자는 영어.
- Terms to use / avoid: "놓친 업무" = 기한이 지났거나 응답·후속 조치가
  빠진 항목. "커넥터" = 외부 서비스 데이터를 가져오는 모듈.
