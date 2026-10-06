# notion-github-setup

- Status: open
- Created: 2026-10-06
- Owner: chaejinseong
- Related spec (if any): app/docs/specs/261006-1730-02-m2b-notion.md, app/docs/specs/261006-1730-03-m2c-github.md, app/docs/specs/261006-1730-04-m3-ai-judgment.md

## Context

M2b·M2c·M3의 수동 확인에 필요한 계정 설정이다. 사람이 직접 한다. 토큰·키는
채팅이나 파일에 붙여넣지 말고 앱 연결/설정 화면에만 입력한다.

## What's left

- [ ] Notion: 통합(내부) 만들기 → 읽기 권한 → "프로젝트 플래너" 데이터베이스에서
  연결(Connections)에 통합 추가
- [ ] GitHub: fine-grained personal access token — Metadata·Pull requests·Issues
  읽기, 대상 저장소 선택
- [ ] Anthropic: API 키 발급, 결제 수단 등록 (월 상한 $5는 앱이 지킨다)

## Notes

GitHub fine-grained 토큰은 고른 저장소만 검색된다.
