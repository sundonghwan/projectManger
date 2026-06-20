import type { CSSProperties } from "react";

export interface AddMenuOption {
  key: string;
  label: string;
  icon?: string;
}

export interface AddMenuProps {
  /** 화면 좌표(앵커 버튼 기준) */
  x: number;
  y: number;
  options: AddMenuOption[];
  onSelect: (key: string) => void;
  onClose: () => void;
}

/** 추가 동작 선택용 팝오버 — 무엇을 추가할지 사용자가 고른다. */
export function AddMenu({ x, y, options, onSelect, onClose }: AddMenuProps) {
  return (
    <div style={overlay} onClick={onClose} data-testid="addmenu-overlay">
      <div
        role="menu"
        aria-label="추가 메뉴"
        style={{ ...menu, left: x, top: y }}
        onClick={(e) => e.stopPropagation()}
      >
        {options.map((o) => (
          <button
            key={o.key}
            role="menuitem"
            onClick={() => {
              onSelect(o.key);
              onClose();
            }}
            style={item}
          >
            {o.icon && <span style={{ width: 16, textAlign: "center" }}>{o.icon}</span>}
            <span>{o.label}</span>
          </button>
        ))}
      </div>
    </div>
  );
}

const overlay: CSSProperties = { position: "fixed", inset: 0, zIndex: 200 };
const menu: CSSProperties = {
  position: "fixed",
  minWidth: 140,
  background: "var(--card)",
  border: "1px solid var(--border)",
  borderRadius: "var(--radius-md)",
  boxShadow: "var(--shadow-popover)",
  padding: 4,
  display: "flex",
  flexDirection: "column",
};
const item: CSSProperties = {
  display: "flex",
  alignItems: "center",
  gap: 8,
  width: "100%",
  border: "none",
  background: "transparent",
  color: "var(--text)",
  borderRadius: "var(--radius-sm)",
  padding: "7px 10px",
  fontSize: 13,
  cursor: "pointer",
  textAlign: "left",
};
