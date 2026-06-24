import { useState, type CSSProperties, type ReactNode } from "react";
import type { TreeRow } from "../domain/tree";
import type { BusinessType } from "../domain/types";
import { TYPE_COLOR, TYPE_LABEL } from "../ui/colors";
import { Icon, type IconName } from "../ui/icons/Icon";
import { CreatePopover, type AddKind } from "./CreatePopover";

export type { AddKind };

export interface SidebarProps {
  rows: TreeRow[];
  selectedId: string | null;
  /** 트리 위에 렌더할 헤더 슬롯 (예: 전역 검색) */
  header?: ReactNode;
  /** 유형 필터: 선택된 유형 집합(비어 있으면 전체) */
  typeFilter?: Set<BusinessType>;
  onToggleType?: (type: BusinessType) => void;
  /** 사업 id → 표시 색상 (유형 컬러 또는 커스텀) */
  colorFor: (businessEntityId: string) => string;
  onSelect: (row: TreeRow) => void;
  onToggle: (row: TreeRow) => void;
  /** 사용자가 고른 유형·이름으로 새 사업 생성 */
  onAddBusiness: (type: BusinessType, name: string) => void;
  /** 사용자가 고른 종류·이름으로 하위 항목 생성 */
  onAddChild: (row: TreeRow, kind: AddKind, name: string) => void;
  /** 더블클릭 인라인 이름변경 (사업/프로젝트/문서) */
  onRename?: (row: TreeRow, name: string) => void;
  /** 보관(소프트 삭제) — 사업/프로젝트/문서/산출물 */
  onArchive?: (row: TreeRow) => void;
  /** 사이드바 접기 — 제공 시 상단에 접기 버튼 노출 */
  onCollapse?: () => void;
}

// 문서·산출물은 트리에선 단일 진입 노드이므로 이름변경·보관 대상이 아니다(각 목록에서 처리).
const RENAMEABLE = new Set<TreeRow["type"]>(["business", "project", "docFolder", "delivFolder"]);
const ARCHIVABLE = new Set<TreeRow["type"]>(["business", "project", "docFolder", "delivFolder"]);
const FOLDER_TYPES = new Set<TreeRow["type"]>(["docFolder", "delivFolder"]);

const ICON: Partial<Record<TreeRow["type"], IconName>> = {
  dashboard: "dashboard",
  document: "document",
  deliverable: "deliverable",
};

function projectIcon(expanded: boolean): IconName {
  return expanded ? "folder-open" : "folder";
}

export function Sidebar(props: SidebarProps) {
  const {
    rows,
    selectedId,
    colorFor,
    onSelect,
    onToggle,
    onAddBusiness,
    onAddChild,
    onRename,
    onArchive,
    onCollapse,
    header,
    typeFilter,
    onToggleType,
  } = props;
  const [editingId, setEditingId] = useState<string | null>(null);
  const [draft, setDraft] = useState("");
  const [filterOpen, setFilterOpen] = useState(false);
  const activeFilterCount = typeFilter?.size ?? 0;
  // 추가 메뉴: 사업 유형 선택 또는 노드 하위 종류 선택
  const [menu, setMenu] = useState<
    | { x: number; y: number; ctx: { kind: "business" } | { kind: "child"; row: TreeRow } }
    | null
  >(null);

  const openMenu = (e: React.MouseEvent, ctx: { kind: "business" } | { kind: "child"; row: TreeRow }) => {
    e.stopPropagation();
    const r = (e.currentTarget as HTMLElement).getBoundingClientRect();
    setMenu({ x: r.left, y: r.bottom + 4, ctx });
  };

  // 추가 메뉴 종류: 사업→프로젝트, 문서/산출물 진입 노드→폴더, 루트 폴더(depth2)→하위 폴더.
  const allowedKinds: AddKind[] = (() => {
    if (!menu || menu.ctx.kind !== "child") return [];
    const r = menu.ctx.row;
    if (r.type === "business") return ["project"];
    if (r.type === "document" || r.type === "deliverable") return ["folder"];
    if ((r.type === "docFolder" || r.type === "delivFolder") && r.depth === 2) return ["folder"];
    return [];
  })();

  const startEdit = (row: TreeRow) => {
    setEditingId(row.id);
    setDraft(row.label);
  };
  const commit = (row: TreeRow) => {
    const name = draft.trim();
    if (name && name !== row.label) onRename?.(row, name);
    setEditingId(null);
  };

  return (
    <aside
      aria-label="사이드바"
      style={{
        width: "var(--sidebar-width)",
        height: "100%",
        background: "var(--sidebar)",
        borderRight: "1px solid var(--border)",
        display: "flex",
        flexDirection: "column",
      }}
    >
      {onCollapse && (
        <div style={{ display: "flex", justifyContent: "flex-end", padding: "6px 8px 0" }}>
          <button onClick={onCollapse} aria-label="사이드바 접기" title="사이드바 접기" style={collapseBtn}>
            <Icon name="chevron-right" size={15} style={{ transform: "rotate(180deg)" }} />
          </button>
        </div>
      )}
      {header}

      <div style={{ padding: "4px 8px 6px", display: "flex", gap: 6, alignItems: "center" }}>
        <button onClick={(e) => openMenu(e, { kind: "business" })} style={{ ...addBusinessStyle, flex: 1 }} aria-label="사업 추가">
          <Icon name="plus" size={16} />
          <span>사업 추가</span>
        </button>
        {onToggleType && (
          <div style={{ position: "relative" }}>
            <button
              onClick={() => setFilterOpen((o) => !o)}
              aria-label="유형 필터"
              aria-pressed={activeFilterCount > 0}
              title="유형 필터"
              style={{ ...filterBtnStyle, color: activeFilterCount > 0 ? "var(--accent)" : "var(--text2)" }}
            >
              <Icon name="filter" size={15} />
              {activeFilterCount > 0 && <span style={filterDot} />}
            </button>
            {filterOpen && (
              <>
                <div style={filterBackdrop} onClick={() => setFilterOpen(false)} />
                <div style={filterPopover} role="dialog" aria-label="유형 필터">
                  <div style={{ display: "flex", alignItems: "center", marginBottom: 8 }}>
                    <span style={{ fontSize: 11, fontWeight: 600, color: "var(--text2)", flex: 1 }}>유형 필터</span>
                    {activeFilterCount > 0 && (
                      <button
                        onClick={() => {
                          (Object.keys(TYPE_LABEL) as BusinessType[]).forEach((t) => {
                            if (typeFilter?.has(t)) onToggleType(t);
                          });
                        }}
                        style={{ fontSize: 11, color: "var(--text3)", background: "transparent", border: "none", cursor: "pointer" }}
                      >
                        초기화
                      </button>
                    )}
                  </div>
                  <div style={{ display: "flex", flexWrap: "wrap", gap: 6 }}>
                    {(Object.keys(TYPE_LABEL) as BusinessType[]).map((t) => {
                      const active = typeFilter?.has(t) ?? false;
                      return (
                        <button
                          key={t}
                          onClick={() => onToggleType(t)}
                          aria-pressed={active}
                          style={{
                            display: "flex",
                            alignItems: "center",
                            gap: 5,
                            fontSize: 12,
                            padding: "3px 9px",
                            borderRadius: 20,
                            border: `1px solid ${active ? TYPE_COLOR[t] : "var(--border)"}`,
                            background: active ? TYPE_COLOR[t] + "22" : "transparent",
                            color: active ? TYPE_COLOR[t] : "var(--text2)",
                            cursor: "pointer",
                          }}
                        >
                          <span style={{ width: 7, height: 7, borderRadius: "50%", background: TYPE_COLOR[t] }} />
                          {TYPE_LABEL[t]}
                        </button>
                      );
                    })}
                  </div>
                </div>
              </>
            )}
          </div>
        )}
      </div>

      <div style={{ flex: 1, overflowY: "auto", padding: "0 8px 8px" }}>
        {rows.length === 0 && (
          <div style={{ padding: "12px 10px", color: "var(--text3)", fontSize: 13 }}>
            아직 사업이 없습니다. “사업 추가”로 시작하세요.
          </div>
        )}
        {rows.map((row) => {
          const selected = row.id === selectedId;
          // +: 사업(프로젝트), 문서/산출물 진입 노드(루트 폴더), 루트 폴더(depth2, 하위 폴더)
          const canAddChild =
            row.type === "business" ||
            row.type === "document" ||
            row.type === "deliverable" ||
            (FOLDER_TYPES.has(row.type) && row.depth === 2);
          return (
            <div
              key={row.id}
              role="treeitem"
              aria-selected={selected}
              aria-expanded={row.hasChildren ? row.expanded : undefined}
              data-row-id={row.id}
              onClick={() => onSelect(row)}
              style={rowStyle(selected)}
            >
              <div style={{ width: row.depth * 16, flexShrink: 0 }} />
              <button
                aria-label={row.expanded ? "접기" : "펼치기"}
                onClick={(e) => {
                  e.stopPropagation();
                  if (row.hasChildren) onToggle(row);
                }}
                style={{
                  ...caretStyle,
                  visibility: row.hasChildren ? "visible" : "hidden",
                }}
              >
                <Icon name={row.expanded ? "chevron-down" : "chevron-right"} size={12} />
              </button>

              {row.type === "business" ? (
                <span style={dotWrap}>
                  <span
                    data-testid="biz-dot"
                    style={{
                      width: 9,
                      height: 9,
                      borderRadius: "50%",
                      background: colorFor(row.entityId),
                    }}
                  />
                </span>
              ) : (
                <span style={iconWrap}>
                  {row.type === "project" || FOLDER_TYPES.has(row.type) ? (
                    <Icon name={projectIcon(row.expanded)} size={15} />
                  ) : ICON[row.type] ? (
                    <Icon name={ICON[row.type]!} size={15} />
                  ) : null}
                </span>
              )}

              {editingId === row.id ? (
                <input
                  autoFocus
                  aria-label="이름 변경"
                  value={draft}
                  onChange={(e) => setDraft(e.target.value)}
                  onClick={(e) => e.stopPropagation()}
                  onBlur={() => commit(row)}
                  onKeyDown={(e) => {
                    if (e.key === "Enter") commit(row);
                    else if (e.key === "Escape") setEditingId(null);
                  }}
                  style={renameInput}
                />
              ) : (
                <span
                  style={{ ...labelStyle, fontWeight: row.type === "business" ? 600 : 400 }}
                  onDoubleClick={(e) => {
                    if (onRename && RENAMEABLE.has(row.type)) {
                      e.stopPropagation();
                      startEdit(row);
                    }
                  }}
                >
                  {row.label}
                </span>
              )}

              {canAddChild && (
                <button
                  aria-label="하위 추가"
                  onClick={(e) => openMenu(e, { kind: "child", row })}
                  style={addChildStyle}
                >
                  <Icon name="plus" size={14} />
                </button>
              )}
              {onArchive && ARCHIVABLE.has(row.type) && (
                <button
                  aria-label={`${row.label} 보관`}
                  title="보관(휴지통으로)"
                  onClick={(e) => {
                    e.stopPropagation();
                    onArchive(row);
                  }}
                  style={archiveStyle}
                >
                  <Icon name="trash" size={13} />
                </button>
              )}
            </div>
          );
        })}
      </div>

      {menu && (
        <CreatePopover
          x={menu.x}
          y={menu.y}
          variant={menu.ctx.kind === "business" ? "business" : "child"}
          allowedKinds={allowedKinds}
          onCreateBusiness={(type, name) => onAddBusiness(type, name)}
          onCreateChild={(kind, name) =>
            menu.ctx.kind === "child" && onAddChild(menu.ctx.row, kind, name)
          }
          onClose={() => setMenu(null)}
        />
      )}
    </aside>
  );
}

const filterBtnStyle: CSSProperties = {
  position: "relative",
  width: 30,
  height: 30,
  flexShrink: 0,
  display: "flex",
  alignItems: "center",
  justifyContent: "center",
  border: "1px solid var(--border)",
  background: "var(--bg)",
  borderRadius: "var(--radius-md)",
  cursor: "pointer",
};
const filterDot: CSSProperties = {
  position: "absolute",
  top: 4,
  right: 4,
  width: 6,
  height: 6,
  borderRadius: "50%",
  background: "var(--accent)",
};
const filterBackdrop: CSSProperties = { position: "fixed", inset: 0, zIndex: 150 };
const filterPopover: CSSProperties = {
  position: "absolute",
  top: "calc(100% + 6px)",
  right: 0,
  zIndex: 160,
  width: 210,
  background: "var(--card)",
  border: "1px solid var(--border)",
  borderRadius: "var(--radius-md)",
  boxShadow: "var(--shadow-popover)",
  padding: 10,
};

const collapseBtn: CSSProperties = {
  width: 26,
  height: 24,
  display: "flex",
  alignItems: "center",
  justifyContent: "center",
  border: "1px solid var(--border)",
  background: "var(--bg)",
  color: "var(--text2)",
  borderRadius: "var(--radius-sm)",
  cursor: "pointer",
  padding: 0,
};
const addBusinessStyle: CSSProperties = {
  height: 30,
  width: "100%",
  display: "flex",
  alignItems: "center",
  gap: 8,
  padding: "0 8px",
  border: "none",
  background: "transparent",
  borderRadius: "var(--radius-md)",
  cursor: "pointer",
  color: "var(--text2)",
  fontSize: 13,
  fontWeight: 500,
};

function rowStyle(selected: boolean): CSSProperties {
  return {
    height: "var(--row-height)",
    display: "flex",
    alignItems: "center",
    gap: 2,
    paddingRight: 6,
    borderRadius: "var(--radius-md)",
    cursor: "pointer",
    background: selected ? "var(--sel)" : "transparent",
    boxShadow: selected ? "inset 2px 0 0 var(--accent)" : "none",
    color: "var(--text)",
  };
}

const caretStyle: CSSProperties = {
  width: 18,
  height: "var(--row-height)",
  flexShrink: 0,
  display: "flex",
  alignItems: "center",
  justifyContent: "center",
  border: "none",
  background: "transparent",
  cursor: "pointer",
  color: "var(--text3)",
  padding: 0,
};

const dotWrap: CSSProperties = {
  width: 16,
  flexShrink: 0,
  display: "flex",
  alignItems: "center",
  justifyContent: "center",
};
const iconWrap: CSSProperties = {
  width: 16,
  flexShrink: 0,
  display: "flex",
  alignItems: "center",
  justifyContent: "center",
  color: "var(--text2)",
};
const labelStyle: CSSProperties = {
  flex: 1,
  overflow: "hidden",
  textOverflow: "ellipsis",
  whiteSpace: "nowrap",
  fontSize: 13.5,
  letterSpacing: "-0.2px",
};
const renameInput: CSSProperties = {
  flex: 1,
  minWidth: 0,
  border: "1px solid var(--accent)",
  borderRadius: 4,
  background: "var(--bg)",
  color: "var(--text)",
  fontSize: 13.5,
  padding: "1px 4px",
  fontFamily: "inherit",
};
const addChildStyle: CSSProperties = {
  width: 20,
  height: 20,
  display: "flex",
  alignItems: "center",
  justifyContent: "center",
  border: "none",
  background: "transparent",
  borderRadius: "var(--radius-sm)",
  cursor: "pointer",
  color: "var(--text2)",
  flexShrink: 0,
};
const archiveStyle: CSSProperties = {
  width: 20,
  height: 20,
  display: "flex",
  alignItems: "center",
  justifyContent: "center",
  border: "none",
  background: "transparent",
  borderRadius: "var(--radius-sm)",
  cursor: "pointer",
  color: "var(--text2)",
  flexShrink: 0,
};
