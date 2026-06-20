import { describe, it, expect } from "vitest";
import { KANBAN_STATUSES, groupByStatus } from "./kanban";
import type { Task } from "./types";

const task = (o: Partial<Task> & Pick<Task, "id" | "status">): Task => ({
  businessId: 1,
  title: `t${o.id}`,
  priority: 2,
  sortOrder: 0,
  ...o,
});

describe("groupByStatus", () => {
  it("항상 4개 컬럼을 순서대로 반환한다", () => {
    const cols = groupByStatus([]);
    expect(cols.map((c) => c.status)).toEqual(KANBAN_STATUSES);
    expect(cols.every((c) => c.tasks.length === 0)).toBe(true);
  });

  it("태스크를 상태별 컬럼에 분배한다", () => {
    const cols = groupByStatus([
      task({ id: 1, status: "todo" }),
      task({ id: 2, status: "doing" }),
      task({ id: 3, status: "doing" }),
      task({ id: 4, status: "done" }),
    ]);
    const byStatus = Object.fromEntries(cols.map((c) => [c.status, c.tasks.length]));
    expect(byStatus).toEqual({ todo: 1, doing: 2, review: 0, done: 1 });
  });

  it("각 컬럼 내부는 sortOrder 오름차순", () => {
    const cols = groupByStatus([
      task({ id: 1, status: "todo", sortOrder: 3 }),
      task({ id: 2, status: "todo", sortOrder: 1 }),
      task({ id: 3, status: "todo", sortOrder: 2 }),
    ]);
    const todo = cols.find((c) => c.status === "todo")!;
    expect(todo.tasks.map((t) => t.id)).toEqual([2, 3, 1]);
  });
});
