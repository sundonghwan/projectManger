import { describe, it, expect } from "vitest";
import { partitionMemos } from "./memoSort";
import type { Memo } from "./types";

const memo = (id: string, pinned: number): Memo => ({
  id,
  businessId: "1",
  title: `m${id}`,
  body: "",
  color: null,
  pinned,
  sortOrder: Number(id),
  archivedAt: null,
  createdAt: "2026-06-24T00:00:00Z",
});

describe("partitionMemos", () => {
  it("고정/기타로 나누고 입력 순서를 보존한다", () => {
    const { pinned, others } = partitionMemos([memo("1", 1), memo("2", 0), memo("3", 1), memo("4", 0)]);
    expect(pinned.map((m) => m.id)).toEqual(["1", "3"]);
    expect(others.map((m) => m.id)).toEqual(["2", "4"]);
  });

  it("빈 입력", () => {
    expect(partitionMemos([])).toEqual({ pinned: [], others: [] });
  });

  it("전부 기타면 pinned 비어있음", () => {
    const { pinned, others } = partitionMemos([memo("1", 0), memo("2", 0)]);
    expect(pinned).toEqual([]);
    expect(others).toHaveLength(2);
  });
});
