import { describe, it, expect } from "vitest";
import { buildTimeline } from "./timeline";
import type { Task } from "./types";

const task = (o: Partial<Task> & Pick<Task, "id">): Task => ({
  businessId: "1",
  title: `t${o.id}`,
  status: "todo",
  priority: 2,
  sortOrder: 0,
  ...o,
});

describe("buildTimeline", () => {
  it("마감 있는 태스크만 포함하고 날짜순 정렬", () => {
    const r = buildTimeline([
      task({ id: "1", dueDate: "2026-07-10" }),
      task({ id: "2", dueDate: "2026-07-01" }),
      task({ id: "3" }), // 마감 없음 → 제외
    ]);
    expect(r.items.map((i) => i.id)).toEqual(["2", "1"]);
    expect(r.minDate).toBe("2026-07-01");
    expect(r.maxDate).toBe("2026-07-10");
  });

  it("ratio는 0~1 범위, 최소=0 최대=1", () => {
    const r = buildTimeline([
      task({ id: "1", dueDate: "2026-07-01" }),
      task({ id: "2", dueDate: "2026-07-11" }),
      task({ id: "3", dueDate: "2026-07-06" }),
    ]);
    const byId = Object.fromEntries(r.items.map((i) => [i.id, i.ratio]));
    expect(byId["1"]).toBeCloseTo(0);
    expect(byId["2"]).toBeCloseTo(1);
    expect(byId["3"]).toBeCloseTo(0.5);
  });

  it("모두 같은 날짜면 ratio 0.5", () => {
    const r = buildTimeline([
      task({ id: "1", dueDate: "2026-07-01" }),
      task({ id: "2", dueDate: "2026-07-01" }),
    ]);
    expect(r.items.every((i) => i.ratio === 0.5)).toBe(true);
  });

  it("빈 입력은 빈 결과", () => {
    const r = buildTimeline([]);
    expect(r.items).toEqual([]);
    expect(r.minDate).toBeNull();
  });
});
