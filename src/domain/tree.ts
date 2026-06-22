// 사이드바 트리 빌더 — 평면 데이터를 계층 트리 행으로 변환.
// 구조(확정): 사업 > [대시보드, 문서, 산출물, 프로젝트들]
// 문서·산출물 개별 항목은 트리에 두지 않는다(사이드바 도배 방지). 각각 단일 진입 노드로
// 들어가 메인 뷰의 목록에서 본다. 프로젝트는 칸반/태스크 진입용 leaf. 태스크·서버도
// 트리에 두지 않는다(메인 뷰 탭에서 관리).
import type { Business, Project } from "./types";

export type TreeNodeType = "business" | "dashboard" | "project" | "document" | "deliverable";

export interface TreeRow {
  /** 고유 행 키 (예: "business:1") — React key / expanded Set 용 */
  id: string;
  /** 노드 종류 */
  type: TreeNodeType;
  /** 원본 엔티티 id (dashboard·document·deliverable 진입 노드는 소속 사업 id) */
  entityId: number;
  depth: number;
  label: string;
  hasChildren: boolean;
  expanded: boolean;
}

export interface TreeInput {
  businesses: Business[];
  projects: Project[];
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
  const { businesses, projects, expanded } = input;
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
    // 2) 문서 의사행 — 개별 문서는 메인 뷰 목록에서 보며, 트리엔 단일 진입 노드만.
    rows.push(entry("document", b.id, "문서"));
    // 3) 산출물 의사행 — 업로드 파일도 메인 뷰 목록에서 본다.
    rows.push(entry("deliverable", b.id, "산출물"));

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

/** 사업 단위 진입 의사행(대시보드/문서/산출물) — entityId 는 소속 사업 id. */
function entry(type: TreeNodeType, businessId: number, label: string): TreeRow {
  return { id: rowId(type, businessId), type, entityId: businessId, depth: 1, label, hasChildren: false, expanded: false };
}
