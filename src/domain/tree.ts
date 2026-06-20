// 사이드바 트리 빌더 — 평면 데이터를 계층 트리 행으로 변환.
// 구조(확정): 사업 > [대시보드, 프로젝트들 > (문서·산출물), 사업 직속 문서·산출물]
// 태스크·서버는 트리에 두지 않는다(메인 뷰 탭에서 관리).
import type { Business, Deliverable, Document, Project } from "./types";

export type TreeNodeType = "business" | "dashboard" | "project" | "document" | "deliverable";

export interface TreeRow {
  /** 고유 행 키 (예: "business:1") — React key / expanded Set 용 */
  id: string;
  /** 노드 종류 */
  type: TreeNodeType;
  /** 원본 엔티티 id (dashboard는 소속 사업 id) */
  entityId: number;
  depth: number;
  label: string;
  hasChildren: boolean;
  expanded: boolean;
}

export interface TreeInput {
  businesses: Business[];
  projects: Project[];
  documents: Document[];
  deliverables: Deliverable[];
  /** 펼쳐진 행 id 집합 */
  expanded: Set<string>;
}

/** 노드 종류 + 엔티티 id → 고유 행 키 */
export function rowId(type: TreeNodeType, entityId: number): string {
  return `${type}:${entityId}`;
}

const isActive = (e: { archivedAt?: string | null }): boolean => !e.archivedAt;
const bySortOrder = <T extends { sortOrder: number }>(a: T, b: T): number =>
  a.sortOrder - b.sortOrder;

export function buildTree(input: TreeInput): TreeRow[] {
  const { businesses, projects, documents, deliverables, expanded } = input;
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
    rows.push({
      id: rowId("dashboard", b.id),
      type: "dashboard",
      entityId: b.id,
      depth: 1,
      label: "대시보드",
      hasChildren: false,
      expanded: false,
    });

    // 2) 프로젝트들 (+ 펼쳐졌으면 그 문서·산출물)
    const bizProjects = projects
      .filter((p) => p.businessId === b.id && isActive(p))
      .sort(bySortOrder);
    for (const p of bizProjects) {
      const projChildDocs = documents
        .filter((d) => d.projectId === p.id && isActive(d))
        .sort(bySortOrder);
      const projChildDelivs = deliverables
        .filter((o) => o.projectId === p.id && isActive(o))
        .sort(bySortOrder);
      const projRowId = rowId("project", p.id);
      const projExpanded = expanded.has(projRowId);
      rows.push({
        id: projRowId,
        type: "project",
        entityId: p.id,
        depth: 1,
        label: p.name,
        hasChildren: projChildDocs.length > 0 || projChildDelivs.length > 0,
        expanded: projExpanded,
      });
      if (projExpanded) {
        for (const d of projChildDocs) rows.push(leaf("document", d.id, 2, d.title));
        for (const o of projChildDelivs) rows.push(leaf("deliverable", o.id, 2, o.title));
      }
    }

    // 3) 사업 직속 문서·산출물 (projectId == null)
    const directDocs = documents
      .filter((d) => d.businessId === b.id && d.projectId == null && isActive(d))
      .sort(bySortOrder);
    const directDelivs = deliverables
      .filter((o) => o.businessId === b.id && o.projectId == null && isActive(o))
      .sort(bySortOrder);
    for (const d of directDocs) rows.push(leaf("document", d.id, 1, d.title));
    for (const o of directDelivs) rows.push(leaf("deliverable", o.id, 1, o.title));
  }

  return rows;
}

function leaf(type: TreeNodeType, entityId: number, depth: number, label: string): TreeRow {
  return { id: rowId(type, entityId), type, entityId, depth, label, hasChildren: false, expanded: false };
}
