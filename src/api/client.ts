// Tauri invoke 래퍼 — 프론트는 이 클라이언트로만 백엔드와 통신.
import { invoke } from "@tauri-apps/api/core";
import type {
  Block,
  Business,
  BusinessType,
  DateString,
  Document,
  EntityStatus,
  Label,
  Priority,
  Project,
  SearchHit,
  Task,
  TaskLabel,
  TaskStatus,
} from "../domain/types";

export interface BusinessCreateInput {
  name: string;
  type: BusinessType;
  color?: string | null;
}
export interface BusinessUpdateInput {
  id: number;
  name: string;
  type: BusinessType;
  status: EntityStatus;
  color?: string | null;
  description?: string | null;
}
export interface ProjectCreateInput {
  businessId: number;
  name: string;
}
export interface ProjectUpdateInput {
  id: number;
  name: string;
  status: EntityStatus;
  description?: string | null;
  dueDate?: DateString | null;
}
export interface TaskCreateInput {
  businessId: number;
  projectId?: number | null;
  title: string;
  priority?: Priority;
}
export interface TaskUpdateInput {
  id: number;
  title: string;
  priority: Priority;
  dueDate?: DateString | null;
  description?: string | null;
}
export interface TaskMoveInput {
  id: number;
  status: TaskStatus;
  sortOrder: number;
}

export const api = {
  business: {
    list: () => invoke<Business[]>("business_list"),
    create: (input: BusinessCreateInput) => invoke<Business>("business_create", { input }),
    update: (input: BusinessUpdateInput) => invoke<Business>("business_update", { input }),
    rename: (id: number, name: string) => invoke<Business>("business_rename", { id, name }),
    archive: (id: number) => invoke<void>("business_archive", { id }),
  },
  project: {
    list: (businessId: number) => invoke<Project[]>("project_list", { businessId }),
    create: (input: ProjectCreateInput) => invoke<Project>("project_create", { input }),
    update: (input: ProjectUpdateInput) => invoke<Project>("project_update", { input }),
    rename: (id: number, name: string) => invoke<Project>("project_rename", { id, name }),
    archive: (id: number) => invoke<void>("project_archive", { id }),
  },
  task: {
    list: (businessId: number, projectId?: number | null) =>
      invoke<Task[]>("task_list", { businessId, projectId: projectId ?? null }),
    create: (input: TaskCreateInput) => invoke<Task>("task_create", { input }),
    update: (input: TaskUpdateInput) => invoke<Task>("task_update", { input }),
    move: (input: TaskMoveInput) => invoke<Task>("task_move", { input }),
    archive: (id: number) => invoke<void>("task_archive", { id }),
  },
  document: {
    list: (businessId: number) => invoke<Document[]>("document_list", { businessId }),
    create: (input: DocumentCreateInput) => invoke<Document>("document_create", { input }),
    rename: (id: number, title: string) => invoke<Document>("document_rename", { id, title }),
    archive: (id: number) => invoke<void>("document_archive", { id }),
  },
  block: {
    list: (documentId: number) => invoke<Block[]>("block_list", { documentId }),
    create: (input: BlockCreateInput) => invoke<Block>("block_create", { input }),
    update: (input: BlockUpdateInput) => invoke<Block>("block_update", { input }),
    delete: (id: number) => invoke<void>("block_delete", { id }),
  },
  label: {
    list: () => invoke<Label[]>("label_list"),
    create: (name: string, color?: string | null) =>
      invoke<Label>("label_create", { input: { name, color: color ?? null } }),
    assign: (taskId: number, labelId: number) =>
      invoke<void>("label_assign", { input: { taskId, labelId } }),
    unassign: (taskId: number, labelId: number) =>
      invoke<void>("label_unassign", { input: { taskId, labelId } }),
    map: (businessId: number) => invoke<TaskLabel[]>("task_label_map", { businessId }),
  },
  backup: {
    /** path 미지정 시 앱 데이터 폴더의 backup.json. 저장 경로 반환. */
    exportJson: (path?: string | null) => invoke<string>("export_json", { path: path ?? null }),
  },
  search: (query: string) => invoke<SearchHit[]>("search", { query }),
};

export interface DocumentCreateInput {
  businessId: number;
  projectId?: number | null;
  title: string;
}
export interface BlockCreateInput {
  documentId: number;
  type: string;
  content: string;
  sortOrder: number;
}
export interface BlockUpdateInput {
  id: number;
  type: string;
  content: string;
}
