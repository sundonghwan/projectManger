import type { Label, TaskLabel } from "./types";

/** 태스크-라벨 조인 행을 taskId → Label[] 맵으로 그룹화. */
export function groupLabelsByTask(rows: TaskLabel[]): Record<number, Label[]> {
  const map: Record<number, Label[]> = {};
  for (const r of rows) {
    (map[r.taskId] ??= []).push({ id: r.labelId, name: r.name, color: r.color });
  }
  return map;
}
