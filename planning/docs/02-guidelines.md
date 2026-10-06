# Planning Guidelines

> 1인 개인 프로젝트 기준. 필요해지면 늘린다.

## Data & sourcing

- Where to look first for user data: 본인 사용 기록 — 앱의 👍/👎 피드백,
  직접 기록한 업무 파악 시간 (PRD G1~G3).
- How to cite a claim: 링크, 또는 "본인 기록 YYYY-MM-DD".
- What counts as validated vs. an assumption: 측정값이나 공식 문서가 있으면
  validated. 그 밖의 추정·목표치는 `[ASSUMPTION]`.

## Writing conventions

- Goals must be stated as measurable outcomes, not activities.
- Keep a PRD to what's necessary for someone to build and evaluate the
  feature — implementation detail belongs in an engineering spec, not here.
- 문서는 한국어, 파일명·코드·식별자는 영어.

## Glossary

- 놓친 업무: 기한이 지났거나 응답·후속 조치가 빠진 항목 (판정 규칙은 PRD FR7~10)
- 커넥터: 외부 서비스 데이터를 가져오는 모듈 (`Connector` trait)
- 항목(Item): 커넥터가 가져온 메일·일정·작업·PR·이슈 한 건
- 단계(stage): 요청 / 진행 / 검토 / 완료 (PIPELINE 카드)

## Review & approval flow

Status values used across `docs/prd/`: `Draft` → `In Review` → `Approved`
→ `Shipped` / `Shelved`.

- Who needs to review before `Approved`: chaejinseong
- Who needs to review before something is shared externally: chaejinseong

## Stakeholder map

- 모든 결정: chaejinseong (1인 프로젝트)
