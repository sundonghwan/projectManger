import type { Task, TaskStatus } from "./types";

export const KANBAN_STATUSES: TaskStatus[] = ["todo", "doing", "review", "done"];

export interface KanbanColumn {
  status: TaskStatus;
  tasks: Task[];
}

/** 태스크를 4개 상태 컬럼으로 분배. 각 컬럼은 sortOrder 오름차순. */
export function groupByStatus(tasks: Task[]): KanbanColumn[] {
  return KANBAN_STATUSES.map((status) => ({
    status,
    tasks: tasks
      .filter((t) => t.status === status)
      .sort((a, b) => a.sortOrder - b.sortOrder),
  }));
}
