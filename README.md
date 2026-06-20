# projectManger

맥/윈도우 데스크톱용 **프로젝트 관리 툴** — 내부용·단일 사용자·로컬 우선.
일을 "사업" 단위로 묶고 할 일·문서·산출물·개발 서버(SSH) 접속을 한 곳에서 다룬다.

## 스택
- **Tauri** (Rust 코어) + **React** + **TypeScript**
- **SQLite** (로컬 저장)
- 시크릿은 OS 키체인에 위임 (DB 평문 저장 없음)

## 개발
```bash
npm install        # 의존성 설치
npm run dev        # 프론트엔드 개발 서버
npm run tauri dev  # 데스크톱 앱 실행
npm test           # 테스트 (Vitest)
```

## 상태
초기 개발 단계. 기능 단위 TDD로 진행.
