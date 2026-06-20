import { describe, it, expect } from "vitest";
import { STEP, MIN_GAP, computeSortOrder, isTooClose, reindex } from "./sortOrder";

describe("computeSortOrder", () => {
  it("빈 위치(앞뒤 모두 없음)에는 STEP을 부여한다", () => {
    expect(computeSortOrder(null, null)).toBe(STEP);
  });

  it("맨 앞에 삽입하면 첫 항목보다 STEP 작은 값", () => {
    expect(computeSortOrder(null, 10)).toBe(10 - STEP);
  });

  it("맨 뒤에 삽입하면 마지막 항목보다 STEP 큰 값", () => {
    expect(computeSortOrder(10, null)).toBe(10 + STEP);
  });

  it("두 항목 사이에는 중간값을 부여한다", () => {
    expect(computeSortOrder(10, 20)).toBe(15);
    expect(computeSortOrder(0, 1)).toBe(0.5);
  });

  it("중간값은 항상 두 경계 사이에 들어간다", () => {
    const mid = computeSortOrder(1, 2);
    expect(mid).toBeGreaterThan(1);
    expect(mid).toBeLessThan(2);
  });
});

describe("isTooClose", () => {
  it("경계 간격이 MIN_GAP보다 작으면 true (정밀도 소진)", () => {
    expect(isTooClose(1, 1 + MIN_GAP / 2)).toBe(true);
  });

  it("간격이 충분하면 false", () => {
    expect(isTooClose(1, 2)).toBe(false);
  });

  it("경계 한쪽이 없으면(맨 앞/뒤) false", () => {
    expect(isTooClose(null, 1)).toBe(false);
    expect(isTooClose(1, null)).toBe(false);
  });

  it("반복 중간 삽입으로 간격이 소진되는 상황을 감지한다", () => {
    let before = 1;
    const after = 2;
    let mid = before;
    for (let i = 0; i < 60; i++) {
      mid = computeSortOrder(before, after);
      before = mid;
    }
    // 60번 반으로 쪼개면 결국 너무 가까워진다
    expect(isTooClose(before, after)).toBe(true);
  });
});

describe("reindex", () => {
  it("표시 순서대로 STEP 간격의 새 sort_order를 부여한다", () => {
    expect(reindex([7, 3, 9])).toEqual([
      { id: 7, sortOrder: STEP },
      { id: 3, sortOrder: STEP * 2 },
      { id: 9, sortOrder: STEP * 3 },
    ]);
  });

  it("빈 배열은 빈 결과", () => {
    expect(reindex([])).toEqual([]);
  });

  it("재배치 결과는 순증가한다", () => {
    const result = reindex([5, 1, 4, 2, 3]);
    const orders = result.map((r) => r.sortOrder);
    for (let i = 1; i < orders.length; i++) {
      expect(orders[i]).toBeGreaterThan(orders[i - 1]);
    }
  });
});
