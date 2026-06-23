// 사이드바 트리 빌더 — 평면 데이터를 계층 트리 행으로 변환.
// 구조: 사업 > [대시보드, 문서, (문서 폴더…), 산출물, (산출물 폴더…), 프로젝트들]
// 문서·산출물 개별 항목은 트리에 두지 않는다(사이드바 도배 방지). 각각 단일 진입 노드로
// 들어가 메인 뷰의 목록에서 본다. 단, 분류 폴더(최대 2단계)는 트리에 노출해 탐색·필터한다.
// 진입 노드 선택 = 전체 항목, 폴더 선택 = 그 폴더의 직속 항목. 프로젝트는 칸반/태스크 진입용 leaf.
import type { Business, Folder, FolderKind, Project } from "./types";

export type TreeNodeType =
  | "business"
  | "dashboard"
  | "project"
  | "document"
  | "deliverable"
  | "docFolder"
  | "delivFolder";

export interface TreeRow {
  /** 고유 행 키 (예: "business:1", "docFolder:7") — React key / expanded Set 용 */
  id: string;
  /** 노드 종류 */
  type: TreeNodeType;
  /** 원본 엔티티 id (dashboard·document·deliverable 진입 노드는 소속 사업 id, 폴더는 폴더 id) */
  entityId: number;
  depth: number;
  label: string;
  hasChildren: boolean;
  expanded: boolean;
}

export interface TreeInput {
  businesses: Business[];
  projects: Project[];
  /** 산출물·문서 분류 폴더(전 사업·전 종류). 없으면 폴더 미노출. */
  folders?: Folder[];
  /** 펼쳐진 행 id 집합 */
  expanded: Set<string>;
}

/** 노드 종류 + 엔티티 id → 고유 행 키 */
export function rowId(type: TreeNodeType, entityId: number): string {
  return `${type}:${entityId}`;
}

/** 폴더 종류 → 트리 노드 타입 */
const folderNodeType = (kind: FolderKind): TreeNodeType =>
  kind === "document" ? "docFolder" : "delivFolder";

const isActive = (e: { archivedAt?: string | null }): boolean => !e.archivedAt;
const bySortOrder = <T extends { sortOrder: number }>(a: T, b: T): number =>
  a.sortOrder - b.sortOrder;

export function buildTree(input: TreeInput): TreeRow[] {
  const { businesses, projects, expanded } = input;
  const folders = input.folders ?? [];
  const rows: TreeRow[] = [];

  const activeBusinesses = businesses.filter(isActive).sort(bySortOrder);

  for (const b of activeBusinesses) {
    const bizRowId = rowId("business", b.id);
    const bizExpanded = expanded.has(bizRowId);
    rows.push({
      id: bizRowId,
      type: "business",
      entityId: b.id,
      depth: 0,
      label: b.name,
      hasChildren: true, // 대시보드가 항상 있으므로
      expanded: bizExpanded,
    });
    if (!bizExpanded) continue;

    // 1) 대시보드 의사행
    rows.push(entry("dashboard", b.id, "대시보드"));
    // 2) 문서 진입 노드 + 문서 폴더 계층
    pushEntryWithFolders(rows, "document", b.id, "문서", folders, expanded);
    // 3) 산출물 진입 노드 + 산출물 폴더 계층
    pushEntryWithFolders(rows, "deliverable", b.id, "산출물", folders, expanded);

    // 4) 프로젝트들 (칸반/태스크 진입용 leaf)
    const bizProjects = projects
      .filter((p) => p.businessId === b.id && isActive(p))
      .sort(bySortOrder);
    for (const p of bizProjects) {
      rows.push({
        id: rowId("project", p.id),
        type: "project",
        entityId: p.id,
        depth: 1,
        label: p.name,
        hasChildren: false,
        expanded: false,
      });
    }
  }

  return rows;
}

/** 진입 노드(문서/산출물)를 push 하고, 펼쳐져 있으면 그 아래 폴더 계층(최대 2단계)을 push. */
function pushEntryWithFolders(
  rows: TreeRow[],
  entryType: "document" | "deliverable",
  businessId: number,
  label: string,
  folders: Folder[],
  expanded: Set<string>,
): void {
  const kind: FolderKind = entryType === "document" ? "document" : "deliverable";
  const nodeType = folderNodeType(kind);
  const mine = folders.filter((f) => f.businessId === businessId && f.kind === kind && isActive(f));
  const roots = mine.filter((f) => f.parentId == null).sort(bySortOrder);

  const entryRowId = rowId(entryType, businessId);
  const entryExpanded = expanded.has(entryRowId);
  rows.push({
    id: entryRowId,
    type: entryType,
    entityId: businessId,
    depth: 1,
    label,
    hasChildren: roots.length > 0,
    expanded: entryExpanded,
  });
  if (!entryExpanded) return;

  for (const root of roots) {
    const children = mine.filter((f) => f.parentId === root.id).sort(bySortOrder);
    const rootRowId = rowId(nodeType, root.id);
    const rootExpanded = expanded.has(rootRowId);
    rows.push({
      id: rootRowId,
      type: nodeType,
      entityId: root.id,
      depth: 2,
      label: root.name,
      hasChildren: children.length > 0,
      expanded: rootExpanded,
    });
    if (!rootExpanded) continue;
    for (const child of children) {
      rows.push({
        id: rowId(nodeType, child.id),
        type: nodeType,
        entityId: child.id,
        depth: 3,
        label: child.name,
        hasChildren: false,
        expanded: false,
      });
    }
  }
}

/** 사업 단위 진입 의사행(대시보드) — entityId 는 소속 사업 id. */
function entry(type: TreeNodeType, businessId: number, label: string): TreeRow {
  return { id: rowId(type, businessId), type, entityId: businessId, depth: 1, label, hasChildren: false, expanded: false };
}
