import { describe, it, expect } from "vitest";
import { folderOptions } from "./folderOptions";
import type { Folder } from "./types";

const f = (o: Partial<Folder> & Pick<Folder, "id" | "name">): Folder => ({
  businessId: 1,
  kind: "deliverable",
  parentId: null,
  sortOrder: 0,
  archivedAt: null,
  ...o,
});

describe("folderOptions", () => {
  it("루트 폴더 바로 뒤에 그 하위 폴더를 배치한다 (depth 표시)", () => {
    const opts = folderOptions([
      f({ id: 1, name: "보고서", sortOrder: 1 }),
      f({ id: 2, name: "1차", parentId: 1, sortOrder: 1 }),
      f({ id: 3, name: "2차", parentId: 1, sortOrder: 2 }),
      f({ id: 4, name: "계약서", sortOrder: 2 }),
    ]);
    expect(opts).toEqual([
      { id: 1, label: "보고서", depth: 0 },
      { id: 2, label: "1차", depth: 1 },
      { id: 3, label: "2차", depth: 1 },
      { id: 4, label: "계약서", depth: 0 },
    ]);
  });

  it("sortOrder 오름차순으로 정렬한다", () => {
    const opts = folderOptions([
      f({ id: 1, name: "둘째", sortOrder: 2 }),
      f({ id: 2, name: "첫째", sortOrder: 1 }),
    ]);
    expect(opts.map((o) => o.label)).toEqual(["첫째", "둘째"]);
  });

  it("보관된 폴더는 제외한다", () => {
    const opts = folderOptions([
      f({ id: 1, name: "활성" }),
      f({ id: 2, name: "보관", archivedAt: "2026-01-01T00:00:00.000Z" }),
    ]);
    expect(opts.map((o) => o.id)).toEqual([1]);
  });
});
