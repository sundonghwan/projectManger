import { useState, type CSSProperties, type ReactNode } from "react";
import type { TreeRow } from "../domain/tree";
import type { BusinessType } from "../domain/types";
import { TYPE_COLOR, TYPE_LABEL } from "../ui/colors";

export interface SidebarProps {
  rows: TreeRow[];
  selectedId: string | null;
  /** 트리 위에 렌더할 헤더 슬롯 (예: 전역 검색) */
  header?: ReactNode;
  /** 유형 필터: 선택된 유형 집합(비어 있으면 전체) */
  typeFilter?: Set<BusinessType>;
  onToggleType?: (type: BusinessType) => void;
  /** 사업 id → 표시 색상 (유형 컬러 또는 커스텀) */
  colorFor: (businessEntityId: number) => string;
  onSelect: (row: TreeRow) => void;
  onToggle: (row: TreeRow) => void;
  onAddBusiness: () => void;
  onAddChild: (row: TreeRow) => void;
  /** 더블클릭 인라인 이름변경 (사업/프로젝트/문서) */
  onRename?: (row: TreeRow, name: string) => void;
}

const RENAMEABLE = new Set<TreeRow["type"]>(["business", "project", "document"]);

const ICON: Partial<Record<TreeRow["type"], string>> = {
  dashboard: "📊",
  document: "📄",
  deliverable: "📦",
};

function projectIcon(expanded: boolean): string {
  return expanded ? "📂" : "📁";
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
    header,
    typeFilter,
    onToggleType,
  } = props;
  const [editingId, setEditingId] = useState<string | null>(null);
  const [draft, setDraft] = useState("");

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
      {header}

      <div style={{ padding: "4px 8px 6px" }}>
        <button onClick={onAddBusiness} style={addBusinessStyle} aria-label="사업 추가">
          <span style={{ width: 16, textAlign: "center", fontWeight: 600 }}>+</span>
          <span>사업 추가</span>
        </button>
      </div>

      <div style={{ flex: 1, overflowY: "auto", padding: "0 8px 8px" }}>
        {rows.length === 0 && (
          <div style={{ padding: "12px 10px", color: "var(--text3)", fontSize: 13 }}>
            아직 사업이 없습니다. “사업 추가”로 시작하세요.
          </div>
        )}
        {rows.map((row) => {
          const selected = row.id === selectedId;
          const canAddChild = row.type === "business" || row.type === "project";
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
                {row.expanded ? "▼" : "▶"}
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
                  {row.type === "project" ? projectIcon(row.expanded) : ICON[row.type]}
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
                  onClick={(e) => {
                    e.stopPropagation();
                    onAddChild(row);
                  }}
                  style={addChildStyle}
                >
                  +
                </button>
              )}
            </div>
          );
        })}
      </div>

      {onToggleType && (
        <div style={filterSection}>
          <div style={{ fontSize: 11, fontWeight: 600, color: "var(--text2)", marginBottom: 8 }}>
            유형 필터
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
      )}
    </aside>
  );
}

const filterSection: CSSProperties = {
  flex: "none",
  borderTop: "1px solid var(--border)",
  padding: "10px 14px 12px",
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
  border: "none",
  background: "transparent",
  cursor: "pointer",
  color: "var(--text2)",
  fontSize: 9,
  padding: 0,
};

const dotWrap: CSSProperties = {
  width: 16,
  flexShrink: 0,
  display: "flex",
  alignItems: "center",
  justifyContent: "center",
};
const iconWrap: CSSProperties = { width: 16, flexShrink: 0, textAlign: "center", fontSize: 13 };
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
  border: "none",
  background: "transparent",
  borderRadius: "var(--radius-sm)",
  cursor: "pointer",
  color: "var(--text2)",
  fontSize: 13,
  flexShrink: 0,
};
