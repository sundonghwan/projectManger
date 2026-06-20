import { describe, it, expect } from "vitest";
import { groupLabelsByTask } from "./labels";
import type { TaskLabel } from "./types";

const tl = (taskId: number, labelId: number, name: string): TaskLabel => ({
  taskId,
  labelId,
  name,
  color: "#3b82f6",
});

describe("groupLabelsByTask", () => {
  it("빈 입력이면 빈 맵", () => {
    expect(groupLabelsByTask([])).toEqual({});
  });

  it("taskId별로 라벨을 모은다", () => {
    const map = groupLabelsByTask([tl(1, 10, "백엔드"), tl(1, 11, "긴급"), tl(2, 10, "백엔드")]);
    expect(map[1].map((l) => l.name)).toEqual(["백엔드", "긴급"]);
    expect(map[2].map((l) => l.name)).toEqual(["백엔드"]);
  });
});
