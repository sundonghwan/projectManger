# projectManger

맥/윈도우 데스크톱용 **Work Vault**입니다. 내부용·단일 사용자·로컬 우선 앱이며, 일을 "사업" 단위로 묶고 프로젝트·할 일·문서·산출물·메모·개발 서버(SSH)를 하나의 vault에서 다룹니다.

## 대상

- 이 저장소를 이어받는 개발자
- 로컬 앱을 실행해 기능을 확인하는 사용자/운영자
- 파일 기반 vault 저장 구조와 동기화 제약을 이해해야 하는 유지보수자

## 현재 스택

- **Tauri v2** + **Rust** 백엔드
- **React 19** + **TypeScript** + **Vite** 프론트엔드
- **파일 기반 vault 저장소**
  - 메인 데이터는 엔티티별 JSON 파일로 저장합니다.
  - 기본 위치는 `<appData>/.projectManger/`입니다.
  - 사용자가 vault 폴더를 선택하면 `<vault>/.projectManger/`를 사용합니다.
  - 업로드한 산출물 파일은 현재 store 안의 `<storeRoot>/files/deliverables/`에 복사합니다.
- **SSH 로컬 저장소**
  - 서버 연결 정보와 스니펫은 `<appData>/local/`에 저장합니다.
  - 시크릿은 OS 키체인에 위임하며 vault에 동기화하지 않습니다.

> 이전 SQLite/rusqlite 저장 구조는 제거되었습니다. README나 오래된 인계 문서에서 SQLite를 언급한다면 현재 코드가 아니라 과거 상태를 설명하는 것입니다.

## 주요 기능

- 사업/프로젝트 트리 탐색
- 칸반·리스트·타임라인 기반 태스크 관리
- 문서 작성 및 문서 폴더 분류
- 산출물 업로드, 상태 변경, 폴더 분류
- Keep식 메모: 색상, 고정, 보관
- SSH 터미널, SFTP 목록, 명령 스니펫
- 휴지통 복구/영구 삭제
- vault 폴더 선택으로 iCloud Drive, Dropbox, Syncthing 같은 외부 폴더 동기화 도구와 함께 사용 가능

## 프로젝트 구조

```text
src/
  api/client.ts          # Tauri invoke 래퍼
  App.tsx                # 앱 상태, 사이드바 트리, 뷰 전환
  components/            # 공용 UI 컴포넌트
  domain/                # 프론트 도메인 타입/정렬/트리/대시보드 로직
  hooks/                 # 기능별 데이터 로딩 훅
  views/                 # 화면 단위 컴포넌트

src-tauri/src/
  lib.rs                 # Tauri 앱 초기화, command 등록
  commands.rs            # 프론트에서 호출하는 command 경계
  config.rs              # vaultPath 로컬 설정
  store/                 # 파일 기반 Store, 모델, ops
  terminal.rs            # SSH 터미널 세션
  sftp.rs                # SFTP 목록
  hostkey.rs             # SSH host key 확인/신뢰
  secrets.rs             # OS 키체인 연동

docs/superpowers/
  specs/                 # 설계 스펙
  plans/                 # 구현 계획/재개 노트
```

## 개발

```bash
npm install
npm run dev
npm run tauri dev
```

- `npm run dev`: Vite 개발 서버만 실행합니다.
- `npm run tauri dev`: 데스크톱 앱을 실행합니다. Tauri 설정상 Vite 서버도 함께 실행됩니다.

## 검증

```bash
npm test
npx tsc --noEmit
npm run build
cd src-tauri && cargo test
```

현재 주 검증 기준은 다음과 같습니다.

- 프론트 단위 테스트: Vitest
- 타입 체크: TypeScript `--noEmit`
- 프로덕션 프론트 빌드: `npm run build`
- 백엔드 단위 테스트: `cargo test`

## 저장소 동작

메인 데이터는 `Store`가 앱 시작 시 JSON 파일을 읽어 인메모리 컬렉션으로 로드하고, 쓰기 시 해당 엔티티 파일만 원자적으로 갱신합니다. 쓰기 경로는 임시 파일을 만든 뒤 rename 하는 방식입니다.

vault 폴더는 앱 상단의 폴더 버튼에서 변경할 수 있습니다. 변경 시 앱은 새 `<vault>/.projectManger/` store를 열고, 실패하면 이전 설정으로 롤백합니다.

산출물 업로드 파일은 active store 기준 `<storeRoot>/files/deliverables/<deliverableId>/<filename>`에 저장합니다. 따라서 vault가 iCloud Drive라면 JSON 메타데이터와 실제 업로드 파일이 모두 iCloud 폴더 아래에 위치합니다. 이전 버전에서 `<appData>/deliverables/`에 저장된 산출물은 앱 시작 시 active store 아래로 복사하고, 기존 로컬 파일은 백업처럼 남겨 둡니다.

동기화는 앱 내 전용 클라우드 API가 아니라 사용자가 선택한 폴더 동기화 도구에 맡깁니다. 같은 ID를 가진 충돌 복사본이 로드되면 `updatedAt` 기준 최신 항목이 메모리에서 승리합니다(LWW).

## SSH/터미널 동작

SSH 연결은 앱 내부 PTY에서 시스템 `ssh` 명령을 실행합니다. 비밀번호 인증을 사용하는 서버는 터미널 프롬프트에서 사용자가 직접 비밀번호를 입력합니다. 서버에 저장된 시크릿 참조는 OS 키체인에 보관하지만, 현재 구현은 비밀번호를 `ssh` 프로세스에 자동 주입하지 않습니다.

Host key는 앱 전용 known_hosts 파일에 저장합니다.

```text
<appData>/known_hosts
```

macOS의 appData 경로에는 보통 `Application Support`처럼 공백이 포함됩니다. `terminal.rs`와 `sftp.rs`는 `UserKnownHostsFile=...` 값을 OpenSSH 설정 파서 규칙에 맞게 escape해서 전달합니다. 이 처리가 없으면 `StrictHostKeyChecking=yes`에서 이미 등록된 host key를 찾지 못하고 다음 오류가 날 수 있습니다.

```text
No ED25519 host key is known for <host> and you have requested strict checking.
Host key verification failed.
```

터미널 세션은 `TERM=xterm-256color`로 실행합니다. 원격에서 Codex 같은 interactive TUI를 실행할 때 `TERM is set to "dumb"` 경고가 보이면, 현재 코드가 반영된 Tauri 앱을 다시 실행했는지 확인합니다.

## 현재 제한과 후속 작업

- 실시간 폴더 감시와 프론트 자동 refetch는 아직 구현되지 않았습니다. 앱 재시작 또는 vault 재오픈 시 외부 변경을 다시 로드합니다.
- 충돌 복사본은 로드 시 메모리에서 LWW로 해소하지만, 진 쪽 파일을 즉시 디스크에서 정리하는 정규화 단계는 별도 후속 작업입니다.
- 서버 연결/스니펫/시크릿은 의도적으로 동기화하지 않습니다. 다른 기기에서 SSH 정보를 공유하려면 별도 설계가 필요합니다.

## 참고 문서

- 파일 vault 설계: `docs/superpowers/specs/2026-06-24-file-vault-storage-design.md`
- Plan 2 폴더 감시 설계: `docs/superpowers/specs/2026-06-29-folder-watch-sync-design.md`
- 파일 vault 재개 노트: `docs/superpowers/plans/RESUME-file-vault.md`
