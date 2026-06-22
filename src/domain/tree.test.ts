import { describe, it, expect } from "vitest";
import { buildTree, rowId, type TreeInput } from "./tree";
import type { Business, Project } from "./types";

// --- 테스트 픽스처 헬퍼 (필요한 필드만 채우고 나머지 기본값) ---
const biz = (o: Partial<Business> & Pick<Business, "id" | "name">): Business => ({
  type: "si",
  status: "active",
  sortOrder: 0,
  archivedAt: null,
  ...o,
});
const proj = (o: Partial<Project> & Pick<Project, "id" | "businessId" | "name">): Project => ({
  status: "active",
  sortOrder: 0,
  archivedAt: null,
  ...o,
});

const empty: TreeInput = {
  businesses: [],
  projects: [],
  expanded: new Set(),
};

describe("buildTree", () => {
  it("빈 입력이면 빈 배열을 반환한다", () => {
    expect(buildTree(empty)).toEqual([]);
  });

  it("접힌 사업은 행 1개만 만든다 (하위 미노출)", () => {
    const rows = buildTree({ ...empty, businesses: [biz({ id: 1, name: "SI사업 A" })] });
    expect(rows).toHaveLength(1);
    expect(rows[0]).toMatchObject({ id: "business:1", type: "business", depth: 0, hasChildren: true, expanded: false });
  });

  it("펼친 사업은 대시보드·문서·산출물 진입 의사행(depth1)을 노출한다", () => {
    const rows = buildTree({
      ...empty,
      businesses: [biz({ id: 1, name: "A" })],
      expanded: new Set([rowId("business", 1)]),
    });
    expect(rows.map((r) => r.id)).toEqual(["business:1", "dashboard:1", "document:1", "deliverable:1"]);
    // 문서·산출물 진입 노드의 entityId 는 사업 id, hasChildren=false
    expect(rows[2]).toMatchObject({ type: "document", depth: 1, label: "문서", entityId: 1, hasChildren: false });
    expect(rows[3]).toMatchObject({ type: "deliverable", depth: 1, label: "산출물", entityId: 1, hasChildren: false });
  });

  it("프로젝트는 진입 노드 뒤에 depth1 leaf(확장 불가)로 노출한다", () => {
    const rows = buildTree({
      ...empty,
      businesses: [biz({ id: 1, name: "A" })],
      projects: [proj({ id: 10, businessId: 1, name: "P1" })],
      expanded: new Set([rowId("business", 1)]),
    });
    const p = rows.find((r) => r.id === "project:10")!;
    expect(p).toMatchObject({ type: "project", depth: 1, hasChildren: false });
    // 진입 의사행들보다 뒤에 온다
    expect(rows.indexOf(p)).toBeGreaterThan(rows.findIndex((r) => r.id === "deliverable:1"));
  });

  it("archived 항목은 제외한다", () => {
    const rows = buildTree({
      ...empty,
      businesses: [
        biz({ id: 1, name: "A" }),
        biz({ id: 2, name: "보관됨", archivedAt: "2026-01-01T00:00:00.000Z" }),
      ],
      projects: [proj({ id: 10, businessId: 1, name: "보관프로젝트", archivedAt: "2026-01-01T00:00:00.000Z" })],
      expanded: new Set([rowId("business", 1)]),
    });
    expect(rows.find((r) => r.id === "business:2")).toBeUndefined();
    expect(rows.find((r) => r.id === "project:10")).toBeUndefined();
  });

  it("같은 레벨은 sortOrder 오름차순으로 정렬한다", () => {
    const rows = buildTree({
      ...empty,
      businesses: [
        biz({ id: 1, name: "둘째", sortOrder: 2 }),
        biz({ id: 2, name: "첫째", sortOrder: 1 }),
      ],
    });
    expect(rows.map((r) => r.label)).toEqual(["첫째", "둘째"]);
  });
});

describe("rowId", () => {
  it("타입과 id를 결합한 고유 키를 만든다", () => {
    expect(rowId("business", 1)).toBe("business:1");
    expect(rowId("project", 10)).toBe("project:10");
    expect(rowId("document", 1)).toBe("document:1");
    expect(rowId("deliverable", 1)).toBe("deliverable:1");
  });
});
