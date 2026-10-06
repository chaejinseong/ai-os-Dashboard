# AI OS Dashboard — 원페이저

- Status: Approved (chaejinseong, 2026-10-06)
- Created: 2026-10-06
- Owner: chaejinseong

> A one-pager, not a PRD. The point is to be cheap to redirect or kill
> before anyone invests in a full spec.

## The headline

macOS 앱 하나만 열면 오늘의 메일·일정·프로젝트 현황이 정리돼 있고, 놓친
업무는 AI가 먼저 짚어준다.

## The problem

업무 정보가 메일, 캘린더, 프로젝트 도구에 흩어져 있어서 상황을 파악하려면
매번 여러 앱을 돌아다녀야 한다. 그러다 보면 답장해야 할 메일이나 다가오는
마감을 놓친다.
[ASSUMPTION — 본인 경험 기반. 얼마나 자주 놓치는지에 대한 데이터는 없음.]

## The solution

- **macOS 데스크톱 앱**이 각 서비스의 공식 API에 OAuth로 직접 연결해
  데이터를 주기적으로 가져온다.
- 데이터 소스는 **커넥터 구조**로 만들어, 서비스를 새로 붙이거나 바꿀 때
  앱 본체를 고치지 않아도 되게 한다.
- AI(LLM API)가 항목을 요약하고, 놓친 업무(지난 마감, 미응답 메일, 멈춘
  작업)를 골라 맨 위에 보여준다.
- 화면 구성: 오늘 현황 / 메일 / 일정 / 프로젝트 / 놓친 업무.

검토한 대안:
- **Claude 안에서 대시보드 운영** (커넥터 + 아티팩트 + 예약 작업): 빠르게
  만들 수 있지만 화면과 동작을 자유롭게 바꾸기 어렵다. 직접 개발하기로
  결정 (2026-10-06 대화).
- **기존 생산성 앱 사용**: 조사 전. [TODO]

## Why now

개인 프로젝트로 지금 시작하기로 했다. LLM API로 요약과 우선순위 판단을
앱에 넣는 비용이 개인이 감당할 만한 수준인지는 PRD에서 확인한다.
[ASSUMPTION]

## Biggest open question / risk

**AI의 "놓친 업무" 판정을 믿을 수 있는가.** 중요한 걸 빠뜨리거나 쓸데없는
항목이 많으면, 결국 원래 앱을 다시 확인하게 되고 이 앱을 쓸 이유가 사라진다.

그 밖의 위험:
- 서비스별 API·OAuth 제약. 예: Gmail 읽기 권한은 민감한 권한으로
  분류된다. 본인만 쓰는 테스트 모드면 괜찮을 것으로 보지만 확인이 필요하다.
  [ASSUMPTION]
- 메일 내용을 외부 AI API로 보낼 때의 개인정보 범위.

## Next step if approved

`_template.md`로 정식 PRD를 쓴다. 그 PRD에서 첫 버전에 연동할 서비스,
"놓친 업무" 판정 규칙, AI에 보낼 데이터 범위를 확정한다.
