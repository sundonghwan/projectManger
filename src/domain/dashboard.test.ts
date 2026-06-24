import { describe, it, expect } from "vitest";
import { dashboardStats } from "./dashboard";
import type { Task } from "./types";

const task = (o: Partial<Task> & Pick<Task, "id" | "status">): Task => ({
  businessId: "1",
  title: `t${o.id}`,
  priority: 2,
  sortOrder: 0,
  ...o,
});

describe("dashboardStats", () => {
  it("빈 목록이면 전부 0, doneRatio 0", () => {
    const s = dashboardStats([]);
    expect(s.total).toBe(0);
    expect(s.counts).toEqual({ todo: 0, doing: 0, review: 0, done: 0 });
    expect(s.doneRatio).toBe(0);
    expect(s.upcoming).toEqual([]);
  });

  it("상태별 카운트와 doneRatio를 계산", () => {
    const s = dashboardStats([
      task({ id: "1", status: "todo" }),
      task({ id: "2", status: "done" }),
      task({ id: "3", status: "done" }),
      task({ id: "4", status: "doing" }),
    ]);
    expect(s.counts).toEqual({ todo: 1, doing: 1, review: 0, done: 2 });
    expect(s.total).toBe(4);
    expect(s.doneRatio).toBeCloseTo(0.5);
  });

  it("upcoming은 미완료+마감 있는 태스크를 마감 오름차순으로", () => {
    const s = dashboardStats([
      task({ id: "1", status: "todo", dueDate: "2026-07-10" }),
      task({ id: "2", status: "todo", dueDate: "2026-07-01" }),
      task({ id: "3", status: "done", dueDate: "2026-06-01" }), // 완료 → 제외
      task({ id: "4", status: "doing" }), // 마감 없음 → 제외
    ]);
    expect(s.upcoming.map((t) => t.id)).toEqual(["2", "1"]);
  });
});
