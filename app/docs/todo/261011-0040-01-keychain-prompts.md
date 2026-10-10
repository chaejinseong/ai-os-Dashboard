# keychain-prompts

- Status: blocked
- Created: 2026-10-11
- Owner: chaejinseong
- Related spec (if any): 없음 (A를 고르면 새 spec 필요)

## Context

2026-10-10 사람 확인 중 macOS가 Keychain 접근 확인 창을 계속 띄웠다. dev 빌드는
ad-hoc 서명이라 다시 빌드할 때마다 서명 식별자가 바뀌고, 앱은 동기화마다 Keychain
항목 6개를 읽는다. 그래서 macOS가 항목마다 다시 묻는다.

## What's left

- [ ] 사람이 방법을 고른다:
  - A) Keychain 항목을 하나로 합치고 메모리에 캐시 — 코드 변경, spec 필요 (Claude 작성)
  - B) 고정 서명 인증서로 서명하고 /Applications에 설치 — 사람의 인증서 필요
- [ ] 고른 방법 진행 (A: spec 작성 → 승인 → 구현·리뷰, B: 서명 설정)

## Notes

임시 해결: 확인 창마다 "항상 허용"을 누른다 (항목별, 다시 빌드하면 또 물을 수 있음).

관련: 2026-10-10에 Google 재로그인이 한 번 필요했다 — refresh가 거부됐고 앱 재시작으로는
안 풀렸다. OAuth 동의 화면이 Testing 상태라 refresh token이 7일 뒤 만료되는 건 이미
app/docs/todo/261006-1535-01-google-cloud-setup.md에 적혀 있다.
