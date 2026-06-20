// 도메인 타입 — docs/02-데이터모델.md / schema.sql 기준.
// 계층: 사업(business) > 프로젝트(project). 문서·산출물·서버는 사업 또는 프로젝트에 부착.

export type BusinessType = "si" | "internal" | "ops" | "etc";

/** 사업/프로젝트 진행 상태 */
export type EntityStatus = "active" | "onhold" | "done";

/** 태스크 상태 — 칸반 컬럼과 1:1 */
export type TaskStatus = "todo" | "doing" | "review" | "done";

/** 산출물 상태 (작성중/검토/완료) */
export type DeliverableStatus = "draft" | "review" | "done";

export type DeliverableKind = "file" | "document";

/** 우선순위: 0 없음 · 1 낮음 · 2 보통 · 3 높음 · 4 긴급 */
export type Priority = 0 | 1 | 2 | 3 | 4;

export type AuthType = "key" | "password" | "agent";

/** ISO8601 UTC 문자열 (예: 2026-06-20T09:14:02.000Z) */
export type Timestamp = string;
/** YYYY-MM-DD */
export type DateString = string;

export interface Business {
  id: number;
  name: string;
  type: BusinessType;
  color?: string | null;
  description?: string | null;
  status: EntityStatus;
  sortOrder: number;
  archivedAt?: Timestamp | null;
}

export interface Project {
  id: number;
  businessId: number;
  name: string;
  description?: string | null;
  status: EntityStatus;
  startDate?: DateString | null;
  dueDate?: DateString | null;
  sortOrder: number;
  archivedAt?: Timestamp | null;
}

export interface Task {
  id: number;
  businessId: number;
  projectId?: number | null;
  parentTaskId?: number | null;
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
  id: number;
  businessId: number;
  projectId?: number | null;
  title: string;
  icon?: string | null;
  sortOrder: number;
  archivedAt?: Timestamp | null;
}

export interface Deliverable {
  id: number;
  businessId: number;
  projectId?: number | null;
  title: string;
  kind: DeliverableKind;
  status: DeliverableStatus;
  currentVersion: number;
  sortOrder: number;
  archivedAt?: Timestamp | null;
}

export interface ServerConnection {
  id: number;
  businessId: number;
  projectId?: number | null;
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
