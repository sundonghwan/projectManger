// 도메인 타입 — docs/02-데이터모델.md / schema.sql 기준.
// 계층: 사업(business) > 프로젝트(project). 문서·산출물·서버는 사업 또는 프로젝트에 부착.

export type BusinessType = string;

/** 사업/프로젝트 진행 상태 */
export type EntityStatus = "active" | "onhold" | "done";

/** 태스크 상태 — 칸반 컬럼과 1:1 */
export type TaskStatus = "todo" | "doing" | "review" | "done";

/** 산출물 상태 (작성중/검토/완료) */
export type DeliverableStatus = "draft" | "review" | "done";

export type DeliverableKind = "file" | "document";

/** 폴더 종류 — 산출물/문서 어느 쪽 분류인지 */
export type FolderKind = "document" | "deliverable";

/** 메모 카드 색상 팔레트 키 (null/default = 기본색) */
export type MemoColor =
  | "default"
  | "red"
  | "orange"
  | "yellow"
  | "green"
  | "teal"
  | "blue"
  | "purple"
  | "gray";

/** 사업별 메모(Keep식) — 색상·고정·보관 */
export interface Memo {
  id: string;
  businessId: string;
  title: string;
  body: string;
  color?: MemoColor | null;
  /** 0/1 */
  pinned: number;
  sortOrder: number;
  archivedAt?: Timestamp | null;
  createdAt: Timestamp;
}

/** 산출물·문서 분류 폴더(최대 2단계). parentId 가 없으면 루트 폴더. */
export interface Folder {
  id: string;
  businessId: string;
  kind: FolderKind;
  parentId?: string | null;
  name: string;
  sortOrder: number;
  archivedAt?: Timestamp | null;
}

/** 우선순위: 0 없음 · 1 낮음 · 2 보통 · 3 높음 · 4 긴급 */
export type Priority = 0 | 1 | 2 | 3 | 4;

export type AuthType = "key" | "password" | "agent";

/** ISO8601 UTC 문자열 (예: 2026-06-20T09:14:02.000Z) */
export type Timestamp = string;
/** YYYY-MM-DD */
export type DateString = string;

export interface Business {
  id: string;
  name: string;
  type: BusinessType;
  color?: string | null;
  description?: string | null;
  status: EntityStatus;
  sortOrder: number;
  archivedAt?: Timestamp | null;
}

export interface Project {
  id: string;
  businessId: string;
  name: string;
  description?: string | null;
  status: EntityStatus;
  startDate?: DateString | null;
  dueDate?: DateString | null;
  sortOrder: number;
  archivedAt?: Timestamp | null;
}

export interface Task {
  id: string;
  businessId: string;
  projectId?: string | null;
  parentTaskId?: string | null;
  title: string;
  description?: string | null;
  status: TaskStatus;
  priority: Priority;
  dueDate?: DateString | null;
  sortOrder: number;
  completedAt?: Timestamp | null;
  archivedAt?: Timestamp | null;
}

export interface Document {
  id: string;
  businessId: string;
  projectId?: string | null;
  folderId?: string | null;
  title: string;
  icon?: string | null;
  body: string;
  editorBody?: string | null;
  editorBodyFormat?: DocumentEditorBodyFormat | null;
  collaborationState?: string | null;
  sortOrder: number;
  archivedAt?: Timestamp | null;
  createdAt: Timestamp;
}

export type DocumentEditorBodyFormat = "blocknote-json";

export interface DocumentAsset {
  id: string;
  documentId: string;
  fileName: string;
  filePath: string;
  url: string;
}

export interface Label {
  id: string;
  name: string;
  color?: string | null;
}

export interface Template {
  id: string;
  name: string;
  kind: "project" | "document";
  payload: string;
}

export interface RecurringTask {
  id: string;
  businessId: string;
  projectId?: string | null;
  title: string;
  priority: Priority;
  intervalDays: number;
  nextRun: DateString;
  /** 0/1 */
  active: number;
}

export interface CommandSnippet {
  id: string;
  serverId: string;
  name: string;
  command: string;
  sortOrder: number;
}

export interface SftpEntry {
  name: string;
  isDir: boolean;
  size: number;
}

export type SearchKind = "business" | "project" | "task" | "document" | "deliverable" | "memo";

export interface SearchHit {
  kind: SearchKind;
  id: string;
  title: string;
  businessId: string;
  projectId?: string | null;
}

export interface TrashItem {
  kind: SearchKind;
  id: string;
  title: string;
  archivedAt?: string | null;
  /** 산출물 보관 항목의 파일 크기(바이트). */
  fileSize?: number | null;
}

/** 태스크-라벨 조인 행 */
export interface TaskLabel {
  taskId: string;
  labelId: string;
  name: string;
  color?: string | null;
}

export type BlockType =
  | "paragraph"
  | "heading"
  | "checklist"
  | "code"
  | "quote"
  | "divider";

export interface Block {
  id: string;
  parentBlockId?: string | null;
  type: BlockType;
  /** 종류별 속성 JSON 문자열 (예: {"text":"...","checked":false}) */
  content: string;
  sortOrder: number;
}

export interface Deliverable {
  id: string;
  businessId: string;
  projectId?: string | null;
  folderId?: string | null;
  title: string;
  kind: DeliverableKind;
  documentId?: string | null;
  filePath?: string | null;
  fileSize?: number | null;
  originalName?: string | null;
  status: DeliverableStatus;
  currentVersion: number;
  sortOrder: number;
  archivedAt?: Timestamp | null;
  createdAt: Timestamp;
}

export interface DeliverableVersion {
  id: string;
  version: number;
  filePath?: string | null;
  note?: string | null;
  createdAt: Timestamp;
}

export interface ServerConnection {
  id: string;
  businessId: string;
  projectId?: string | null;
  name: string;
  host: string;
  port: number;
  username: string;
  authType: AuthType;
  keyPath?: string | null;
  /** OS 키체인 참조 키 — 실제 비밀값은 DB에 없음 */
  secretRef?: string | null;
  lastUsedAt?: Timestamp | null;
  archivedAt?: Timestamp | null;
}
