import type { Task, TaskStatus } from "./types";

export interface DashboardStats {
  counts: Record<TaskStatus, number>;
  total: number;
  /** 완료 비율 0..1 */
  doneRatio: number;
  /** 미완료 + 마감 있는 태스크, 마감 오름차순 */
  upcoming: Task[];
}

export function dashboardStats(tasks: Task[]): DashboardStats {
  const counts = { todo: 0, doing: 0, review: 0, done: 0 } as Record<TaskStatus, number>;
  for (const t of tasks) counts[t.status] += 1;
  const total = tasks.length;
  const doneRatio = total === 0 ? 0 : counts.done / total;
  const upcoming = tasks
    .filter((t) => t.status !== "done" && t.dueDate)
    .sort((a, b) => (a.dueDate! < b.dueDate! ? -1 : a.dueDate! > b.dueDate! ? 1 : 0));
  return { counts, total, doneRatio, upcoming };
}
