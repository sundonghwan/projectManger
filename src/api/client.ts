// Tauri invoke 래퍼 — 프론트는 이 클라이언트로만 백엔드와 통신.
import { invoke } from "@tauri-apps/api/core";
import type {
  Block,
  Business,
  BusinessType,
  CommandSnippet,
  DateString,
  Deliverable,
  DeliverableStatus,
  DocumentAsset,
  DocumentEditorBodyFormat,
  Document,
  EntityStatus,
  Folder,
  FolderKind,
  Label,
  Memo,
  MemoColor,
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
  id: string;
  name: string;
  type: BusinessType;
  status: EntityStatus;
  color?: string | null;
  description?: string | null;
}
export interface ProjectCreateInput {
  businessId: string;
  name: string;
}
export interface ProjectUpdateInput {
  id: string;
  name: string;
  status: EntityStatus;
  description?: string | null;
  dueDate?: DateString | null;
}
export interface TaskCreateInput {
  businessId: string;
  projectId?: string | null;
  title: string;
  priority?: Priority;
}
export interface TaskUpdateInput {
  id: string;
  title: string;
  priority: Priority;
  dueDate?: DateString | null;
  description?: string | null;
}
export interface TaskMoveInput {
  id: string;
  status: TaskStatus;
  sortOrder: number;
}
export interface DocumentEditorBodyInput {
  body: string;
  editorBody: string;
  editorBodyFormat: DocumentEditorBodyFormat;
  collaborationState?: string | null;
}

export const api = {
  business: {
    list: () => invoke<Business[]>("business_list"),
    create: (input: BusinessCreateInput) => invoke<Business>("business_create", { input }),
    update: (input: BusinessUpdateInput) => invoke<Business>("business_update", { input }),
    rename: (id: string, name: string) => invoke<Business>("business_rename", { id, name }),
    archive: (id: string) => invoke<void>("business_archive", { id }),
  },
  project: {
    list: (businessId: string) => invoke<Project[]>("project_list", { businessId }),
    create: (input: ProjectCreateInput) => invoke<Project>("project_create", { input }),
    update: (input: ProjectUpdateInput) => invoke<Project>("project_update", { input }),
    rename: (id: string, name: string) => invoke<Project>("project_rename", { id, name }),
    archive: (id: string) => invoke<void>("project_archive", { id }),
  },
  task: {
    list: (businessId: string, projectId?: string | null) =>
      invoke<Task[]>("task_list", { businessId, projectId: projectId ?? null }),
    create: (input: TaskCreateInput) => invoke<Task>("task_create", { input }),
    update: (input: TaskUpdateInput) => invoke<Task>("task_update", { input }),
    move: (input: TaskMoveInput) => invoke<Task>("task_move", { input }),
    archive: (id: string) => invoke<void>("task_archive", { id }),
  },
  document: {
    list: (businessId: string) => invoke<Document[]>("document_list", { businessId }),
    create: (input: DocumentCreateInput) => invoke<Document>("document_create", { input }),
    get: (id: string) => invoke<Document>("document_get", { id }),
    rename: (id: string, title: string) => invoke<Document>("document_rename", { id, title }),
    /** 문서를 폴더로 이동(folderId=null 이면 미분류). */
    move: (id: string, folderId: string | null) =>
      invoke<Document>("document_move", { id, folderId: folderId ?? null }),
    /** 본문(마크다운) 저장. */
    setBody: (id: string, body: string) => invoke<void>("document_set_body", { id, body }),
    setEditorBody: (id: string, input: DocumentEditorBodyInput) =>
      invoke<void>("document_set_editor_body", {
        id,
        body: input.body,
        editorBody: input.editorBody,
        editorBodyFormat: input.editorBodyFormat,
        collaborationState: input.collaborationState ?? null,
      }),
    uploadAsset: (documentId: string, fileName: string, bytes: number[]) =>
      invoke<DocumentAsset>("document_asset_upload", { documentId, fileName, bytes }),
    archive: (id: string) => invoke<void>("document_archive", { id }),
  },
  block: {
    list: (documentId: string) => invoke<Block[]>("block_list", { documentId }),
    create: (input: BlockCreateInput) => invoke<Block>("block_create", { input }),
    update: (input: BlockUpdateInput) => invoke<Block>("block_update", { input }),
    delete: (id: string) => invoke<void>("block_delete", { id }),
  },
  label: {
    list: () => invoke<Label[]>("label_list"),
    create: (name: string, color?: string | null) =>
      invoke<Label>("label_create", { input: { name, color: color ?? null } }),
    assign: (taskId: string, labelId: string) =>
      invoke<void>("label_assign", { input: { taskId, labelId } }),
    unassign: (taskId: string, labelId: string) =>
      invoke<void>("label_unassign", { input: { taskId, labelId } }),
    map: (businessId: string) => invoke<TaskLabel[]>("task_label_map", { businessId }),
  },
  deliverable: {
    list: (businessId: string) => invoke<Deliverable[]>("deliverable_list", { businessId }),
    /** 다중 파일 업로드 — 앱 데이터 폴더로 복사 후 생성된 산출물 목록 반환. folderId 지정 시 해당 폴더에 배치. */
    upload: (businessId: string, projectId: string | null, paths: string[], folderId?: string | null) =>
      invoke<Deliverable[]>("deliverable_upload", {
        businessId,
        projectId: projectId ?? null,
        folderId: folderId ?? null,
        paths,
      }),
    rename: (id: string, title: string) =>
      invoke<Deliverable>("deliverable_rename", { id, title }),
    setStatus: (id: string, status: DeliverableStatus) =>
      invoke<Deliverable>("deliverable_set_status", { id, status }),
    /** 산출물을 폴더로 이동(folderId=null 이면 미분류). */
    move: (id: string, folderId: string | null) =>
      invoke<Deliverable>("deliverable_move", { id, folderId: folderId ?? null }),
    /** 복사 보관된 파일을 백엔드 경계 검증 후 OS 기본 앱으로 연다. */
    open: (id: string) => invoke<void>("deliverable_open", { id }),
    archive: (id: string) => invoke<void>("deliverable_archive", { id }),
  },
  memo: {
    list: (businessId: string) => invoke<Memo[]>("memo_list", { businessId }),
    create: (businessId: string, title: string, body: string) =>
      invoke<Memo>("memo_create", { input: { businessId, title, body } }),
    update: (id: string, title: string, body: string) =>
      invoke<Memo>("memo_update", { input: { id, title, body } }),
    setColor: (id: string, color: MemoColor | null) =>
      invoke<Memo>("memo_set_color", { id, color: color ?? null }),
    setPinned: (id: string, pinned: boolean) => invoke<Memo>("memo_set_pinned", { id, pinned }),
    archive: (id: string) => invoke<void>("memo_archive", { id }),
  },
  folder: {
    /** 사업의 모든(문서·산출물) 폴더. 프론트가 kind/parentId 로 분기. */
    list: (businessId: string) => invoke<Folder[]>("folder_list", { businessId }),
    create: (input: { businessId: string; kind: FolderKind; parentId?: string | null; name: string }) =>
      invoke<Folder>("folder_create", {
        input: { businessId: input.businessId, kind: input.kind, parentId: input.parentId ?? null, name: input.name },
      }),
    rename: (id: string, name: string) => invoke<Folder>("folder_rename", { id, name }),
    remove: (id: string) => invoke<void>("folder_delete", { id }),
  },
  server: {
    list: (businessId: string) => invoke<ServerConnection[]>("server_list", { businessId }),
    create: (input: {
      businessId: string;
      projectId?: string | null;
      name: string;
      host: string;
      port: number;
      username: string;
      authType: AuthType;
      keyPath?: string | null;
    }) => invoke<ServerConnection>("server_create", { input }),
    update: (input: {
      id: string;
      name: string;
      host: string;
      port: number;
      username: string;
      authType: AuthType;
      keyPath?: string | null;
    }) => invoke<ServerConnection>("server_update", { input }),
    archive: (id: string) => invoke<void>("server_archive", { id }),
    setSecret: (id: string, secret: string) => invoke<void>("server_set_secret", { id, secret }),
    clearSecret: (id: string) => invoke<void>("server_clear_secret", { id }),
    hasSecret: (id: string) => invoke<boolean>("server_has_secret", { id }),
  },
  snippet: {
    list: (serverId: string) => invoke<CommandSnippet[]>("snippet_list", { serverId }),
    create: (serverId: string, name: string, command: string) =>
      invoke<CommandSnippet>("snippet_create", { input: { serverId, name, command } }),
    delete: (id: string) => invoke<void>("snippet_delete", { id }),
  },
  ssh: {
    connect: (id: string) => invoke<void>("ssh_connect", { id }),
    write: (id: string, data: string) => invoke<void>("ssh_write", { id, data }),
    resize: (id: string, rows: number, cols: number) =>
      invoke<void>("ssh_resize", { id, rows, cols }),
    disconnect: (id: string) => invoke<void>("ssh_disconnect", { id }),
    /** 이 호스트가 이미 신뢰(known_hosts 등록)되어 있는지. */
    hostStatus: (id: string) => invoke<boolean>("ssh_host_status", { id }),
    /** ssh-keyscan 으로 호스트 공개키·지문을 가져온다(아직 신뢰 X). */
    scanHost: (id: string) => invoke<{ fingerprint: string; keyLines: string }>("ssh_scan_host", { id }),
    /** 사용자가 지문 확인 후 수락한 호스트 키를 앱 known_hosts 에 등록. */
    trustHost: (keyLines: string) => invoke<void>("ssh_trust_host", { keyLines }),
  },
  sftp: {
    list: (id: string, path: string) => invoke<SftpEntry[]>("sftp_list", { id, path }),
  },
  template: {
    list: () => invoke<Template[]>("template_list"),
    create: (name: string, kind: "project" | "document", payload: string) =>
      invoke<Template>("template_create", { input: { name, kind, payload } }),
    delete: (id: string) => invoke<void>("template_delete", { id }),
    applyProject: (templateId: string, businessId: string) =>
      invoke<string>("template_apply_project", { templateId, businessId }),
    applyDocument: (templateId: string, businessId: string, projectId?: string | null) =>
      invoke<string>("template_apply_document", { templateId, businessId, projectId: projectId ?? null }),
  },
  recurring: {
    list: (businessId: string) => invoke<RecurringTask[]>("recurring_list", { businessId }),
    create: (input: {
      businessId: string;
      projectId?: string | null;
      title: string;
      priority: number;
      intervalDays: number;
      nextRun: string;
    }) => invoke<RecurringTask>("recurring_create", { input }),
    setActive: (id: string, active: boolean) =>
      invoke<void>("recurring_set_active", { id, active }),
    delete: (id: string) => invoke<void>("recurring_delete", { id }),
    generate: (today: string) => invoke<number>("recurring_generate", { today }),
  },
  search: (query: string) => invoke<SearchHit[]>("search", { query }),
  trash: {
    list: () => invoke<TrashItem[]>("trash_list"),
    restore: (kind: SearchKind, id: string) => invoke<void>("trash_restore", { kind, id }),
    purge: (kind: SearchKind, id: string) => invoke<void>("trash_purge", { kind, id }),
  },
  vault: {
    /** 현재 vault 폴더(null = 기본 위치). */
    path: () => invoke<string | null>("vault_path"),
    /** vault 폴더 지정 후 Store 재오픈. */
    set: (path: string) => invoke<void>("vault_set", { path }),
  },
};

export interface DocumentCreateInput {
  businessId: string;
  projectId?: string | null;
  folderId?: string | null;
  title: string;
}
export interface BlockCreateInput {
  documentId: string;
  type: string;
  content: string;
  sortOrder: number;
}
export interface BlockUpdateInput {
  id: string;
  type: string;
  content: string;
}
