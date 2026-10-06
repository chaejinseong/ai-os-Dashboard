# AI OS Dashboard

- Status: Approved (chaejinseong, 2026-10-06)
- Created: 2026-10-06
- Owner: chaejinseong
- One-pager (if any): planning/docs/prd/ai-os-dashboard-one-pager.md

## Overview / problem statement

메일(Gmail), 일정(Google Calendar), 프로젝트(Notion), 개발 작업(GitHub)이
서로 다른 앱에 흩어져 있어, 하루 업무를 파악하려면 네 곳을 돌아다녀야 한다.
그 과정에서 답장할 메일, 응답하지 않은 일정 초대, 지난 마감, 대기 중인 리뷰
요청을 놓친다. [ASSUMPTION — 본인 경험 기반, 정량 데이터 없음. G1 기준선
측정으로 확인한다.]

이 앱은 본인 한 명이 쓰는 macOS 데스크톱 앱이다. 네 서비스의 데이터를
가져와 한 화면에 보여주고, 놓친 업무를 규칙과 AI(Claude API)로 골라 맨 위에
띄운다.

## Goals & success metrics

| # | Goal | How to check |
|---|---|---|
| G1 | 아침 업무 파악 시간을 기준선 대비 50% 줄인다 [ASSUMPTION 목표치] | 앱 사용 전 1주, 사용 후 2주 동안 "업무 파악 시작~끝" 시간을 직접 기록 |
| G2 | 앱이 띄운 "놓친 업무" 중 80% 이상이 실제로 조치가 필요한 항목이다 | 항목별 👍/👎 피드백을 앱이 저장. 2주 사용 후 👍 비율 |
| G3 | 앱이 못 잡은 놓친 업무가 주당 1건 이하다 | 주 1회 네 서비스를 직접 점검해 앱에 없던 항목 수를 기록 |
| G4 | 화면의 데이터가 15분 이상 묵지 않는다 | 서비스별 마지막 동기화 시각을 화면에 표시 |
| G5 | Claude API 비용이 월 상한($5)을 넘지 않는다 | 앱이 호출별 토큰 수를 기록하고 월 누적 비용을 설정 화면에 표시 |

## Non-goals

- 여러 사용자, 회원가입, 클라우드 동기화
- 쓰기 동작: 메일 발송·답장, 일정 생성·수정, Notion·GitHub 항목 수정. v1은
  **읽기 전용**이다. 원래 앱으로 이동하는 링크만 제공한다.
- macOS 외 플랫폼 (구조만 확장 가능하게 둔다)
- 다른 AI 서비스의 사용량·토큰 표시 (2026-10-06 범위에서 제외)
- 메일 본문 전체를 AI로 보내는 것

## User stories / use cases

1. 아침에 앱을 열면 "놓친 업무"와 오늘 일정이 맨 위에 있어서, 무엇부터 할지
   바로 정한다.
2. 이틀 전에 받은 질문 메일에 답장하지 않았다면, 앱이 그 메일을 놓친 업무로
   띄우고 한 줄 요약을 보여준다. 클릭하면 Gmail의 해당 스레드가 열린다.
3. 아직 수락·거절하지 않은 회의 초대가 놓친 업무에 뜬다.
4. Notion 프로젝트에서 마감이 지났는데 완료되지 않은 작업이 뜬다.
5. GitHub에서 나에게 요청된 리뷰가 하루 넘게 대기 중이면 뜬다.
6. 잘못 뜬 항목에 👎를 누르면 숨겨지고, 그 피드백이 판정 품질 측정(G2)에
   쓰인다.

## Functional requirements

**연결**
1. 네 서비스(Gmail, Google Calendar, Notion, GitHub)를 각각 OAuth로 연결하고
   해제할 수 있다.
2. 각 서비스는 공통 커넥터 인터페이스를 구현한다. 새 서비스를 붙일 때
   화면·판정 로직을 고치지 않아도 된다.
3. Notion은 사용자가 고른 데이터베이스만 읽는다. 마감일·상태로 쓸 속성을
   사용자가 지정한다.

**동기화**
4. 기본 15분마다, 그리고 수동 새로고침 시 데이터를 가져온다.
5. 가져온 데이터는 로컬에 캐시한다. 앱을 열면 캐시로 먼저 그리고, 그 다음에
   갱신한다.
6. 서비스별 마지막 동기화 시각과 오류(예: 다시 로그인 필요)를 표시한다.

**놓친 업무 판정** — 규칙으로 후보를 고르고, AI는 메일 후보에만 쓴다.
7. 메일: 받은 지 2일 이상 지났고, 내가 답장하지 않은 스레드를 후보로 한다.
   AI가 "답장/조치가 필요한가"를 판정하고 한 줄 요약을 만든다.
8. 일정: 응답하지 않은 초대(needsAction)는 놓친 업무로 띄운다.
9. Notion: 마감일이 지났고 상태가 완료가 아닌 항목을 띄운다.
10. GitHub: 나에게 요청된 리뷰가 1일 이상 대기 중인 PR, 나에게 할당된 열린
    이슈 중 마일스톤 기한이 지난 것을 띄운다.
11. 판정 기준값(2일, 1일 등)은 설정에서 바꿀 수 있다.
12. 항목별 👍/👎 피드백을 저장한다. 👎 항목은 숨긴다.

**화면**
13. 메인 화면: 놓친 업무 → 오늘·내일 일정 → 서비스별 요약(메일, 프로젝트,
    GitHub) 순서.
14. 각 항목은 원래 서비스의 해당 위치로 여는 링크를 가진다.

**AI**
15. Claude API에는 메일의 제목·보낸 사람·본문 앞부분(최대 500자
    [ASSUMPTION])만 보낸다.
16. 같은 메일 스레드는 새 메시지가 오기 전까지 다시 판정하지 않는다
    (결과 캐시).
17. 호출별 입력·출력 토큰 수를 기록하고 월 누적 비용을 보여준다. 월 상한에
    닿으면 AI 판정을 멈추고, 규칙 판정만으로 계속 동작한다.

## Non-functional requirements

- 보안: OAuth 토큰과 Claude API 키는 macOS Keychain에 저장한다. 평문 파일이나
  웹뷰(프런트엔드) 코드에 두지 않는다.
- 개인정보: 데이터는 이 Mac의 로컬 캐시와 각 서비스 API, Claude API 사이에서만
  오간다. 다른 서버로 보내지 않는다.
- 성능: 앱 실행 후 3초 안에 캐시된 화면을 표시한다 [ASSUMPTION 목표치].
- 오프라인: 네트워크가 없으면 캐시를 보여주고 "오프라인" 상태를 표시한다.

## Design considerations

- 화면 설계: planning/docs/design/ui-design.md (참고 이미지
  planning/docs/design/reference-dashboard.png). 다크 테마, 사이드바와
  COMPLETION·BOARD·PIPELINE·MISSED·SCHEDULE 카드로 구성한다.
- 메뉴 막대 아이콘에 놓친 업무 개수 배지를 다는 것은 nice-to-have다.

## Technical considerations

- **Tauri**: Rust 백엔드 + 웹 기술 프런트엔드. 외부 API 호출과 비밀값
  처리는 Rust 쪽에서 하고, 프런트엔드는 결과만 받는다. Windows·Linux 확장은
  Tauri가 지원하지만, macOS Keychain 같은 OS 기능은 플랫폼별 구현이 필요하다.
- **Google OAuth (Gmail, Calendar)**: Google Cloud 프로젝트의 OAuth 동의
  화면이 "Testing" 상태이면 refresh token이 7일 뒤 만료된다 ([Google
  문서](https://developers.google.com/identity/protocols/oauth2)). 그대로면
  일주일마다 다시 로그인해야 하므로, 동의 화면을 게시(In production)하는
  방안을 검토해야 한다. Gmail 읽기 권한은 Google이 제한된(restricted)
  범위로 분류하므로, 게시할 때 필요한 검증 절차를 확인해야 한다
  [ASSUMPTION — 본인 1명 사용 시 요구사항은 확인 필요].
- **Notion**: 연결한 페이지·데이터베이스에만 접근할 수 있다. 연결할 때
  사용자가 데이터베이스를 공유해야 한다.
- **GitHub**: OAuth 앱 또는 fine-grained token. 어느 쪽인지는 spec에서 정한다.
- 로컬 캐시 저장소(예: SQLite)와 프런트엔드 프레임워크는 spec에서 정한다.

## Timeline & milestones

날짜는 정하지 않았다. 순서만 정한다.

1. M1: 앱 골격, 커넥터 인터페이스, Gmail 연결, 메인 화면 (규칙 판정만)
2. M2: Google Calendar, Notion, GitHub 커넥터
3. M3: Claude API 메일 판정, 피드백(👍/👎), 비용 기록
4. M4: 2주 실사용 후 G1~G5 평가

## Open questions & risks

결정됨 (2026-10-06, planning/docs/reports/261006-1720-01-open-questions-resolved.md):
- Claude API 월 상한 **$5**, 모델 **Claude Haiku 4.5** (설정에서 변경 가능)
- Google OAuth: 개발 중(M1~M3)에는 **Testing**, M4 실사용 전에 게시 절차 확인
- Notion: **"프로젝트 플래너"** 데이터베이스 (제목 `프로젝트`, 마감 `마감일`,
  상태 `진행 상태`: 시작 전 / 진행 중 / 완료)

남은 위험:
- AI 판정 품질이 낮으면(G2 미달) 앱을 쓸 이유가 약해진다 — 원페이저의
  가장 큰 위험. M3 이후 바로 측정한다.

## Appendix

- 원페이저: planning/docs/prd/ai-os-dashboard-one-pager.md
- 범위 결정 (2026-10-06, chaejinseong): 첫 연동 Gmail·Google Calendar·Notion·
  GitHub, AI는 Claude API, 메일은 제목·보낸 사람·앞부분만, 기술은 Tauri.
  planning/docs/reports/261006-1530-01-v1-scope-decisions.md

## Handoff

- Spec(s):
  - app/docs/specs/261006-1535-01-m1-app-skeleton-gmail.md (M1)
  - app/docs/specs/261006-1730-01-m2a-google-calendar.md (M2a)
  - app/docs/specs/261006-1730-02-m2b-notion.md (M2b)
  - app/docs/specs/261006-1730-03-m2c-github.md (M2c)
  - app/docs/specs/261006-1730-04-m3-ai-judgment.md (M3)
- Status of each: Approved (app/docs/specs/00-index.md)
- Anything the spec intentionally left out of this PRD, and why: M4는 코드가 아니라
  2주 실사용 평가라 spec 대신 planning/docs/todo/261006-1720-01-m4-evaluation.md 로 둔다.
