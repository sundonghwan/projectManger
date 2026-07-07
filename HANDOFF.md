# 작업 인계 (HANDOFF) — projectManger

> 갱신: 2026-06-29 · 다음 담당 에이전트/개발자용. 현재 코드 기준으로 작성했습니다.

## 0. 프로젝트 개요

- **무엇**: **Work Vault** — 사업 단위로 프로젝트/태스크/문서/산출물/메모/SSH를 하나의 vault에서 관리하는 **Tauri v2 데스크톱 앱**.
- **스택**: Rust/Tauri 백엔드(`src-tauri/src/`), React+TypeScript+Vite 프론트(`src/`).
- **저장소**: https://github.com/sundonghwan/projectManger
- **로컬 경로**: `/Users/polaris/Documents/sundonghwan/projectManger`
- **현재 브랜치**: `feat/folders`

## 1. 현재 저장 구조

SQLite는 제거되었습니다. 현재 앱은 파일 기반 vault 저장소를 사용합니다.

- 메인 데이터: `<storeRoot>/<collection>/<id>.json`
  - 기본 storeRoot: `<appData>/.projectManger/`
  - 사용자가 vault를 고르면: `<vault>/.projectManger/`
- 업로드 산출물 파일: `<storeRoot>/files/deliverables/<deliverableId>/<filename>`
  - 이전 버전의 `<appData>/deliverables/` 파일은 앱 시작 시 active store 아래로 복사하고 메타데이터를 새 경로로 갱신합니다.
  - 기존 로컬 파일은 자동 삭제하지 않습니다.
- 로컬 전용 데이터: `<appData>/local/`
  - 서버 연결 정보
  - 명령 스니펫
  - SSH 시크릿 참조
- 시크릿 값: OS 키체인

핵심 파일:

- Store 초기화: `src-tauri/src/lib.rs`
- Store 모델/컬렉션: `src-tauri/src/store/`
- Store ops: `src-tauri/src/store/ops/`
- vault 설정: `src-tauri/src/config.rs`, `commands.rs`의 `vault_path`/`vault_set`
- 프론트 API 래퍼: `src/api/client.ts`

## 2. 최근 완료 상태

- SQLite/rusqlite 기반 저장소 제거
- 백엔드 Store를 파일 기반 JSON 컬렉션으로 전환
- SSH 서버/스니펫을 vault 밖 LocalStore로 분리
- 프론트 전체 ID 타입을 string/uuid 기준으로 정리
- vault 폴더 선택 UI 추가
- 같은 ID 충돌 복사본 로드 시 `updatedAt` 기준 LWW 선택 구현
- 산출물 열기를 `deliverable_open(id)` 백엔드 command로 이동하고, active store의 `files/deliverables/`와 legacy `<appData>/deliverables/`만 열도록 canonicalize 검증 추가
- 산출물 업로드 파일 저장 위치를 appData에서 active store root의 `files/deliverables/`로 이동
- 기존 appData 산출물 파일을 active store로 복사하는 startup migration 추가
- SSH/SFTP의 `UserKnownHostsFile` 값을 OpenSSH 설정 파서 규칙에 맞게 escape하도록 수정
- SSH 터미널 세션에 `TERM=xterm-256color` 설정 추가

검증된 체크:

```bash
npm test
npx tsc --noEmit
npm run build
cd src-tauri && cargo test
```

최근 SSH/산출물 보안 및 vault 파일 저장 수정 후 `cd src-tauri && cargo test` 기준 134개 테스트가 통과했습니다.

## 3. 현재 Git/워킹트리 주의

- `HANDOFF.md`는 현재 untracked 파일일 수 있습니다. 필요한 경우 의도적으로 stage/commit 하세요.
- 브랜치 tip은 `feat/folders`입니다.
- 오래된 문서에 SQLite, export/import, `repo/*`, `migrations/*`가 언급될 수 있습니다. 현재 코드 기준으로는 stale 정보입니다.

## 4. 남은 작업 우선순위

### A. 문서 최신화

- README와 HANDOFF가 파일 vault 구조를 설명하도록 유지합니다.
- 과거 SQLite 설계 문서는 "폐기/과거 상태"임을 명확히 표시합니다.

### B. SSH/터미널 후속 확인

현재 구현:

- 앱 전용 known_hosts: `<appData>/known_hosts`
- `terminal.rs`/`sftp.rs`: `UserKnownHostsFile=...` 값 escape 처리
- `terminal.rs`: SSH PTY 세션에 `TERM=xterm-256color` 설정

운영 확인 포인트:

- host key를 신뢰한 서버는 `StrictHostKeyChecking=yes`에서도 다시 연결되어야 합니다.
- macOS appData 경로의 `Application Support` 공백 때문에 known_hosts 경로가 잘리는 오류가 없어야 합니다.
- 원격에서 Codex 같은 interactive TUI를 실행할 때 `TERM=dumb` 경고가 없어야 합니다.
- 비밀번호 인증은 현재 터미널 프롬프트에서 직접 입력하는 방식입니다. 저장된 비밀번호를 자동 입력하는 기능은 별도 설계가 필요합니다.

### C. Plan 2 — 동기화 견고화

이미 구현됨:

- 앱 재시작/Store reload 시 같은 ID 충돌 복사본 LWW 선택

미구현:

- store root 재귀 감시
- 외부 변경 감지 후 Store reload
- Tauri event emit
- 프론트 refetch
- self-write 이벤트 debounce
- 디스크의 패배 충돌 복사본 정규화/삭제

신중히 설계해야 하는 이유:

- command write와 watcher reload가 같은 `Mutex<Store>`를 사용해야 합니다.
- 앱이 직접 쓴 파일 이벤트를 다시 읽는 루프를 피해야 합니다.
- 여러 파일이 한 번에 바뀌는 sync burst를 debounce해야 합니다.

## 5. 개발/검증 명령

```bash
cd /Users/polaris/Documents/sundonghwan/projectManger
npm test
npx tsc --noEmit
npm run build
cd src-tauri && cargo test
npm run tauri dev
```

`npm run tauri dev`는 실제 네이티브 창 검증이 필요할 때 실행합니다.

## 6. 참고 문서

- 파일 vault 설계: `docs/superpowers/specs/2026-06-24-file-vault-storage-design.md`
- Plan 2 폴더 감시 설계: `docs/superpowers/specs/2026-06-29-folder-watch-sync-design.md`
- 파일 vault 재개 노트: `docs/superpowers/plans/RESUME-file-vault.md`
- 폴더 설계: `docs/superpowers/specs/2026-06-23-folders-design.md`
- 메모 설계: `docs/superpowers/specs/2026-06-24-memo-design.md`

## 7. 알려진 리스크

- 파일 store의 현재 디스크 레이아웃은 flat collection 구조입니다. 설계 문서 일부는 business별 nested layout을 설명하므로, watch/sync 설계 시 실제 구현을 우선 확인해야 합니다.
- 실시간 외부 변경 반영은 아직 없습니다.
- 삭제 전파는 물리 삭제 기반이며 tombstone 모델은 없습니다. 여러 기기 삭제/편집 경합은 데이터 보존 쪽으로 치우칠 수 있습니다.
- SSH 정보는 기기 로컬 전용입니다. 이 동작은 현재 의도된 설계입니다.
