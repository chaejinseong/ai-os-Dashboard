# google-cloud-setup

- Status: open
- Created: 2026-10-06
- Owner: chaejinseong
- Related spec (if any): app/docs/specs/261006-1535-01-m1-app-skeleton-gmail.md

## Context

M1의 Gmail 연결 수동 확인에는 Google Cloud의 OAuth 클라이언트가 필요하다.
계정 설정이라 사람이 직접 한다.

## What's left

- [ ] Google Cloud Console에서 프로젝트 생성
- [ ] Gmail API 사용 설정 (M2 대비 Google Calendar API도)
- [ ] OAuth 동의 화면: User type External, 상태 Testing, 테스트 사용자에 본인 추가
- [ ] 사용자 인증 정보 → OAuth 클라이언트 ID → 애플리케이션 유형 "데스크톱 앱"
- [ ] 받은 클라이언트 ID/secret을 앱 연결 화면에 입력 (파일·채팅에 붙여넣지 않기)

- [ ] `npm run tauri dev`에서 다크 화면 배치를 참고 이미지와 비교
- [ ] 브라우저에서 Gmail·Calendar 읽기 권한 승인 후 미응답 메일 카드·알림 확인
- [ ] 앱 종료·재실행 후 캐시가 즉시 표시되는지 확인
- [ ] 연결 해제 후 캐시 삭제 및 `security find-generic-password -s com.chaejinseong.ai-os-dashboard -a google.refresh_token` 실패 확인

## Notes

Testing 상태에서는 refresh token이 7일 뒤 만료되어 다시 로그인이 필요하다.
