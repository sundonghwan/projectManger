// Tauri invoke 래퍼 — 프론트는 이 클라이언트로만 백엔드와 통신.
import { invoke } from "@tauri-apps/api/core";
import { openPath } from "@tauri-apps/plugin-opener";
import type {
  Block,
  Business,
  BusinessType,
  CommandSnippet,
  DateString,
  Deliverable,
  DeliverableStatus,
  Document,
  EntityStatus,
  Folder,
  FolderKind,
  Label,
  Priority,
  AuthType,
  Project,
  RecurringTask,
  SearchHit,
  SearchKind,
  ServerConnection,
  SftpEntry,
  Task,
  TaskLabel,
  TaskStatus,
  Template,
  TrashItem,
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
    get: (id: number) => invoke<Document>("document_get", { id }),
    rename: (id: number, title: string) => invoke<Document>("document_rename", { id, title }),
    /** 문서를 폴더로 이동(folderId=null 이면 미분류). */
    move: (id: number, folderId: number | null) =>
      invoke<Document>("document_move", { id, folderId: folderId ?? null }),
    /** 본문(마크다운) 저장. */
    setBody: (id: number, body: string) => invoke<void>("document_set_body", { id, body }),
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
    /** path 미지정 시 앱 데이터 폴더의 backup.json 에서 가져오기(추가). */
    importJson: (path?: string | null) => invoke<void>("import_json", { path: path ?? null }),
  },
  deliverable: {
    list: (businessId: number) => invoke<Deliverable[]>("deliverable_list", { businessId }),
    /** 다중 파일 업로드 — 앱 데이터 폴더로 복사 후 생성된 산출물 목록 반환. folderId 지정 시 해당 폴더에 배치. */
    upload: (businessId: number, projectId: number | null, paths: string[], folderId?: number | null) =>
      invoke<Deliverable[]>("deliverable_upload", {
        businessId,
        projectId: projectId ?? null,
        folderId: folderId ?? null,
        paths,
      }),
    rename: (id: number, title: string) =>
      invoke<Deliverable>("deliverable_rename", { id, title }),
    setStatus: (id: number, status: DeliverableStatus) =>
      invoke<Deliverable>("deliverable_set_status", { id, status }),
    /** 산출물을 폴더로 이동(folderId=null 이면 미분류). */
    move: (id: number, folderId: number | null) =>
      invoke<Deliverable>("deliverable_move", { id, folderId: folderId ?? null }),
    /** 복사 보관된 파일을 OS 기본 앱으로 연다. */
    open: (path: string) => openPath(path),
    archive: (id: number) => invoke<void>("deliverable_archive", { id }),
  },
  folder: {
    /** 사업의 모든(문서·산출물) 폴더. 프론트가 kind/parentId 로 분기. */
    list: (businessId: number) => invoke<Folder[]>("folder_list", { businessId }),
    create: (input: { businessId: number; kind: FolderKind; parentId?: number | null; name: string }) =>
      invoke<Folder>("folder_create", {
        input: { businessId: input.businessId, kind: input.kind, parentId: input.parentId ?? null, name: input.name },
      }),
    rename: (id: number, name: string) => invoke<Folder>("folder_rename", { id, name }),
    remove: (id: number) => invoke<void>("folder_delete", { id }),
  },
  server: {
    list: (businessId: number) => invoke<ServerConnection[]>("server_list", { businessId }),
    create: (input: {
      businessId: number;
      projectId?: number | null;
      name: string;
      host: string;
      port: number;
      username: string;
      authType: AuthType;
      keyPath?: string | null;
    }) => invoke<ServerConnection>("server_create", { input }),
    update: (input: {
      id: number;
      name: string;
      host: string;
      port: number;
      username: string;
      authType: AuthType;
      keyPath?: string | null;
    }) => invoke<ServerConnection>("server_update", { input }),
    archive: (id: number) => invoke<void>("server_archive", { id }),
    setSecret: (id: number, secret: string) => invoke<void>("server_set_secret", { id, secret }),
    clearSecret: (id: number) => invoke<void>("server_clear_secret", { id }),
    hasSecret: (id: number) => invoke<boolean>("server_has_secret", { id }),
  },
  snippet: {
    list: (serverId: number) => invoke<CommandSnippet[]>("snippet_list", { serverId }),
    create: (serverId: number, name: string, command: string) =>
      invoke<CommandSnippet>("snippet_create", { input: { serverId, name, command } }),
    delete: (id: number) => invoke<void>("snippet_delete", { id }),
  },
  ssh: {
    connect: (id: number) => invoke<void>("ssh_connect", { id }),
    write: (id: number, data: string) => invoke<void>("ssh_write", { id, data }),
    resize: (id: number, rows: number, cols: number) =>
      invoke<void>("ssh_resize", { id, rows, cols }),
    disconnect: (id: number) => invoke<void>("ssh_disconnect", { id }),
  },
  sftp: {
    list: (id: number, path: string) => invoke<SftpEntry[]>("sftp_list", { id, path }),
  },
  template: {
    list: () => invoke<Template[]>("template_list"),
    create: (name: string, kind: "project" | "document", payload: string) =>
      invoke<Template>("template_create", { input: { name, kind, payload } }),
    delete: (id: number) => invoke<void>("template_delete", { id }),
    applyProject: (templateId: number, businessId: number) =>
      invoke<number>("template_apply_project", { templateId, businessId }),
    applyDocument: (templateId: number, businessId: number, projectId?: number | null) =>
      invoke<number>("template_apply_document", { templateId, businessId, projectId: projectId ?? null }),
  },
  recurring: {
    list: (businessId: number) => invoke<RecurringTask[]>("recurring_list", { businessId }),
    create: (input: {
      businessId: number;
      projectId?: number | null;
      title: string;
      priority: number;
      intervalDays: number;
      nextRun: string;
    }) => invoke<RecurringTask>("recurring_create", { input }),
    setActive: (id: number, active: boolean) =>
      invoke<void>("recurring_set_active", { id, active }),
    delete: (id: number) => invoke<void>("recurring_delete", { id }),
    generate: (today: string) => invoke<number>("recurring_generate", { today }),
  },
  search: (query: string) => invoke<SearchHit[]>("search", { query }),
  trash: {
    list: () => invoke<TrashItem[]>("trash_list"),
    restore: (kind: SearchKind, id: number) => invoke<void>("trash_restore", { kind, id }),
    purge: (kind: SearchKind, id: number) => invoke<void>("trash_purge", { kind, id }),
  },
};

export interface DocumentCreateInput {
  businessId: number;
  projectId?: number | null;
  folderId?: number | null;
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
