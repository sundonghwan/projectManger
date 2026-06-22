import type { CSSProperties } from "react";
import type { SearchKind, TrashItem } from "../domain/types";
import { Icon } from "../ui/icons/Icon";

export interface TrashPanelProps {
  items: TrashItem[];
  onRestore: (item: TrashItem) => void;
  onPurge: (item: TrashItem) => void;
  onClose: () => void;
}

const KIND_LABEL: Record<SearchKind, string> = {
  business: "사업",
  project: "프로젝트",
  task: "태스크",
  document: "문서",
};

export function TrashPanel({ items, onRestore, onPurge, onClose }: TrashPanelProps) {
  return (
    <div style={overlay} onClick={onClose} data-testid="trash-overlay">
      <div style={modal} role="dialog" aria-label="휴지통" onClick={(e) => e.stopPropagation()}>
        <div style={{ display: "flex", alignItems: "center", marginBottom: 12 }}>
          <span style={{ display: "inline-flex", alignItems: "center", gap: 7, fontSize: 15, fontWeight: 600, flex: 1 }}>
            <Icon name="trash" size={16} />
            휴지통
          </span>
          <button aria-label="닫기" onClick={onClose} style={iconBtn}><Icon name="close" size={16} /></button>
        </div>
        {items.length === 0 ? (
          <div style={{ padding: "20px 0", textAlign: "center", color: "var(--text3)", fontSize: 13 }}>
            보관된 항목이 없습니다.
          </div>
        ) : (
          items.map((item) => (
            <div key={`${item.kind}:${item.id}`} style={row} data-testid={`trash-${item.kind}-${item.id}`}>
              <span style={{ fontSize: 11, color: "var(--text3)", width: 56 }}>{KIND_LABEL[item.kind]}</span>
              <span style={{ flex: 1, fontSize: 13, overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
                {item.title}
              </span>
              <button onClick={() => onRestore(item)} style={restoreBtn}>복구</button>
              <button onClick={() => onPurge(item)} style={purgeBtn} aria-label={`${item.title} 영구삭제`}>
                삭제
              </button>
            </div>
          ))
        )}
      </div>
    </div>
  );
}

const overlay: CSSProperties = {
  position: "fixed",
  inset: 0,
  background: "rgba(0,0,0,.4)",
  display: "flex",
  alignItems: "center",
  justifyContent: "center",
  zIndex: 100,
};
const modal: CSSProperties = {
  width: 480,
  maxWidth: "90vw",
  maxHeight: "80vh",
  overflowY: "auto",
  background: "var(--card)",
  border: "1px solid var(--border)",
  borderRadius: "var(--radius-lg)",
  boxShadow: "var(--shadow-modal)",
  padding: 20,
};
const row: CSSProperties = {
  display: "flex",
  alignItems: "center",
  gap: 8,
  padding: "8px 0",
  borderTop: "1px solid var(--border)",
};
const iconBtn: CSSProperties = { display: "inline-flex", alignItems: "center", justifyContent: "center", border: "none", background: "transparent", color: "var(--text2)", cursor: "pointer", padding: 0 };
const restoreBtn: CSSProperties = {
  border: "1px solid var(--border)",
  background: "var(--bg)",
  color: "var(--accent)",
  borderRadius: "var(--radius-md)",
  padding: "4px 10px",
  fontSize: 12,
  fontWeight: 600,
  cursor: "pointer",
};
const purgeBtn: CSSProperties = {
  border: "1px solid var(--border)",
  background: "var(--bg)",
  color: "var(--st-danger)",
  borderRadius: "var(--radius-md)",
  padding: "4px 10px",
  fontSize: 12,
  fontWeight: 600,
  cursor: "pointer",
};
