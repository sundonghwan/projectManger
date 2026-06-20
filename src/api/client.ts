// Tauri invoke 래퍼 — 프론트는 이 클라이언트로만 백엔드와 통신.
import { invoke } from "@tauri-apps/api/core";
import type {
  Business,
  BusinessType,
  DateString,
  EntityStatus,
  Priority,
  Project,
  Task,
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
    archive: (id: number) => invoke<void>("business_archive", { id }),
  },
  project: {
    list: (businessId: number) => invoke<Project[]>("project_list", { businessId }),
    create: (input: ProjectCreateInput) => invoke<Project>("project_create", { input }),
    update: (input: ProjectUpdateInput) => invoke<Project>("project_update", { input }),
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
};
