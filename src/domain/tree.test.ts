import { describe, it, expect } from "vitest";
import { buildTree, rowId, type TreeInput } from "./tree";
import type { Business, Project, Document, Deliverable } from "./types";

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
const doc = (o: Partial<Document> & Pick<Document, "id" | "businessId" | "title">): Document => ({
  sortOrder: 0,
  archivedAt: null,
  projectId: null,
  ...o,
});
const deliv = (
  o: Partial<Deliverable> & Pick<Deliverable, "id" | "businessId" | "title">,
): Deliverable => ({
  kind: "file",
  status: "draft",
  currentVersion: 1,
  sortOrder: 0,
  archivedAt: null,
  projectId: null,
  ...o,
});

const empty: TreeInput = {
  businesses: [],
  projects: [],
  documents: [],
  deliverables: [],
  expanded: new Set(),
};

describe("buildTree", () => {
  it("빈 입력이면 빈 배열을 반환한다", () => {
    expect(buildTree(empty)).toEqual([]);
  });

  it("접힌 사업은 행 1개만 만든다 (대시보드/하위 미노출)", () => {
    const rows = buildTree({
      ...empty,
      businesses: [biz({ id: 1, name: "SI사업 A" })],
    });
    expect(rows).toHaveLength(1);
    expect(rows[0]).toMatchObject({
      id: "business:1",
      type: "business",
      depth: 0,
      label: "SI사업 A",
      hasChildren: true,
      expanded: false,
    });
  });

  it("펼친 사업은 대시보드 의사행(depth1)을 먼저 노출한다", () => {
    const rows = buildTree({
      ...empty,
      businesses: [biz({ id: 1, name: "A" })],
      expanded: new Set([rowId("business", 1)]),
    });
    expect(rows.map((r) => r.id)).toEqual(["business:1", "dashboard:1"]);
    expect(rows[1]).toMatchObject({ type: "dashboard", depth: 1, hasChildren: false });
  });

  it("펼친 사업 아래 프로젝트는 depth1, 접혀 있으면 하위 미노출", () => {
    const rows = buildTree({
      ...empty,
      businesses: [biz({ id: 1, name: "A" })],
      projects: [proj({ id: 10, businessId: 1, name: "P1" })],
      documents: [doc({ id: 100, businessId: 1, projectId: 10, title: "문서" })],
      expanded: new Set([rowId("business", 1)]),
    });
    const proj1 = rows.find((r) => r.id === "project:10")!;
    expect(proj1).toMatchObject({ type: "project", depth: 1, hasChildren: true, expanded: false });
    // 프로젝트가 접혀 있으므로 그 문서는 노출되지 않음
    expect(rows.find((r) => r.id === "document:100")).toBeUndefined();
  });

  it("펼친 프로젝트는 문서·산출물을 depth2로 노출한다", () => {
    const rows = buildTree({
      ...empty,
      businesses: [biz({ id: 1, name: "A" })],
      projects: [proj({ id: 10, businessId: 1, name: "P1" })],
      documents: [doc({ id: 100, businessId: 1, projectId: 10, title: "요건정의서" })],
      deliverables: [deliv({ id: 200, businessId: 1, projectId: 10, title: "제안서" })],
      expanded: new Set([rowId("business", 1), rowId("project", 10)]),
    });
    const d = rows.find((r) => r.id === "document:100")!;
    const o = rows.find((r) => r.id === "deliverable:200")!;
    expect(d).toMatchObject({ type: "document", depth: 2, label: "요건정의서" });
    expect(o).toMatchObject({ type: "deliverable", depth: 2, label: "제안서" });
    // 문서가 산출물보다 먼저
    expect(rows.indexOf(d)).toBeLessThan(rows.indexOf(o));
  });

  it("사업 직속 문서/산출물(projectId=null)은 프로젝트 뒤에 depth1로 노출", () => {
    const rows = buildTree({
      ...empty,
      businesses: [biz({ id: 1, name: "A" })],
      projects: [proj({ id: 10, businessId: 1, name: "P1" })],
      documents: [doc({ id: 100, businessId: 1, projectId: null, title: "사업 직속 문서" })],
      expanded: new Set([rowId("business", 1)]),
    });
    const direct = rows.find((r) => r.id === "document:100")!;
    expect(direct).toMatchObject({ type: "document", depth: 1 });
    expect(rows.indexOf(direct)).toBeGreaterThan(rows.indexOf(rows.find((r) => r.id === "project:10")!));
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

  it("프로젝트가 비어 있으면 hasChildren=false", () => {
    const rows = buildTree({
      ...empty,
      businesses: [biz({ id: 1, name: "A" })],
      projects: [proj({ id: 10, businessId: 1, name: "빈 프로젝트" })],
      expanded: new Set([rowId("business", 1)]),
    });
    expect(rows.find((r) => r.id === "project:10")).toMatchObject({ hasChildren: false });
  });
});

describe("rowId", () => {
  it("타입과 id를 결합한 고유 키를 만든다", () => {
    expect(rowId("business", 1)).toBe("business:1");
    expect(rowId("project", 10)).toBe("project:10");
    expect(rowId("dashboard", 1)).toBe("dashboard:1");
  });
});
