import { describe, it, expect } from "vitest";
import { buildTree, rowId, type TreeInput } from "./tree";
import type { Business, Folder, Project } from "./types";

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
    const rows = buildTree({ ...empty, businesses: [biz({ id: "1", name: "SI사업 A" })] });
    expect(rows).toHaveLength(1);
    expect(rows[0]).toMatchObject({ id: "business:1", type: "business", depth: 0, hasChildren: true, expanded: false });
  });

  it("펼친 사업은 대시보드·문서·산출물 진입 의사행(depth1)을 노출한다", () => {
    const rows = buildTree({
      ...empty,
      businesses: [biz({ id: "1", name: "A" })],
      expanded: new Set([rowId("business", "1")]),
    });
    expect(rows.map((r) => r.id)).toEqual(["business:1", "dashboard:1", "document:1", "deliverable:1"]);
    // 문서·산출물 진입 노드의 entityId 는 사업 id, hasChildren=false
    expect(rows[2]).toMatchObject({ type: "document", depth: 1, label: "문서", entityId: "1", hasChildren: false });
    expect(rows[3]).toMatchObject({ type: "deliverable", depth: 1, label: "산출물", entityId: "1", hasChildren: false });
  });

  it("프로젝트는 진입 노드 뒤에 depth1 leaf(확장 불가)로 노출한다", () => {
    const rows = buildTree({
      ...empty,
      businesses: [biz({ id: "1", name: "A" })],
      projects: [proj({ id: "10", businessId: "1", name: "P1" })],
      expanded: new Set([rowId("business", "1")]),
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
        biz({ id: "1", name: "A" }),
        biz({ id: "2", name: "보관됨", archivedAt: "2026-01-01T00:00:00.000Z" }),
      ],
      projects: [proj({ id: "10", businessId: "1", name: "보관프로젝트", archivedAt: "2026-01-01T00:00:00.000Z" })],
      expanded: new Set([rowId("business", "1")]),
    });
    expect(rows.find((r) => r.id === "business:2")).toBeUndefined();
    expect(rows.find((r) => r.id === "project:10")).toBeUndefined();
  });

  it("같은 레벨은 sortOrder 오름차순으로 정렬한다", () => {
    const rows = buildTree({
      ...empty,
      businesses: [
        biz({ id: "1", name: "둘째", sortOrder: 2 }),
        biz({ id: "2", name: "첫째", sortOrder: 1 }),
      ],
    });
    expect(rows.map((r) => r.label)).toEqual(["첫째", "둘째"]);
  });
});

describe("rowId", () => {
  it("타입과 id를 결합한 고유 키를 만든다", () => {
    expect(rowId("business", "1")).toBe("business:1");
    expect(rowId("project", "10")).toBe("project:10");
    expect(rowId("document", "1")).toBe("document:1");
    expect(rowId("deliverable", "1")).toBe("deliverable:1");
  });
});

const fold = (o: Partial<Folder> & Pick<Folder, "id" | "name" | "kind">): Folder => ({
  businessId: "1",
  parentId: null,
  sortOrder: 0,
  archivedAt: null,
  ...o,
});

describe("buildTree 폴더", () => {
  it("폴더가 있으면 진입 노드 hasChildren=true, 펼치면 루트 폴더(depth2)를 노출한다", () => {
    const rows = buildTree({
      ...empty,
      businesses: [biz({ id: "1", name: "A" })],
      folders: [fold({ id: "7", name: "보고서", kind: "deliverable" })],
      expanded: new Set([rowId("business", "1"), rowId("deliverable", "1")]),
    });
    const entry = rows.find((r) => r.id === "deliverable:1")!;
    expect(entry).toMatchObject({ hasChildren: true, expanded: true });
    const folderRow = rows.find((r) => r.id === "delivFolder:7")!;
    expect(folderRow).toMatchObject({ type: "delivFolder", depth: 2, label: "보고서", entityId: "7" });
  });

  it("진입 노드를 접으면 폴더는 노출되지 않는다", () => {
    const rows = buildTree({
      ...empty,
      businesses: [biz({ id: "1", name: "A" })],
      folders: [fold({ id: "7", name: "보고서", kind: "deliverable" })],
      expanded: new Set([rowId("business", "1")]), // deliverable 진입 노드는 접힘
    });
    expect(rows.find((r) => r.id === "delivFolder:7")).toBeUndefined();
    expect(rows.find((r) => r.id === "deliverable:1")).toMatchObject({ hasChildren: true, expanded: false });
  });

  it("루트 폴더를 펼치면 하위 폴더(depth3)를 노출한다", () => {
    const rows = buildTree({
      ...empty,
      businesses: [biz({ id: "1", name: "A" })],
      folders: [
        fold({ id: "7", name: "보고서", kind: "document" }),
        fold({ id: "8", name: "1차", kind: "document", parentId: "7" }),
      ],
      expanded: new Set([rowId("business", "1"), rowId("document", "1"), rowId("docFolder", "7")]),
    });
    const root = rows.find((r) => r.id === "docFolder:7")!;
    expect(root).toMatchObject({ hasChildren: true, expanded: true });
    const sub = rows.find((r) => r.id === "docFolder:8")!;
    expect(sub).toMatchObject({ type: "docFolder", depth: 3, label: "1차" });
  });

  it("문서 폴더와 산출물 폴더는 각자 진입 노드 아래로만 들어간다", () => {
    const rows = buildTree({
      ...empty,
      businesses: [biz({ id: "1", name: "A" })],
      folders: [
        fold({ id: "1", name: "문서폴더", kind: "document" }),
        fold({ id: "2", name: "산출물폴더", kind: "deliverable" }),
      ],
      expanded: new Set([rowId("business", "1"), rowId("document", "1"), rowId("deliverable", "1")]),
    });
    expect(rows.find((r) => r.id === "docFolder:1")).toBeDefined();
    expect(rows.find((r) => r.id === "delivFolder:2")).toBeDefined();
    // 문서 진입 노드 아래에 산출물 폴더가 끼지 않는다
    const docIdx = rows.findIndex((r) => r.id === "document:1");
    const delivIdx = rows.findIndex((r) => r.id === "deliverable:1");
    const docFolderIdx = rows.findIndex((r) => r.id === "docFolder:1");
    expect(docFolderIdx).toBeGreaterThan(docIdx);
    expect(docFolderIdx).toBeLessThan(delivIdx);
  });
});
