import type { CSSProperties } from "react";
import type { Tab } from "./tabs";

export interface TabBarProps {
  tabs: Tab[];
  activeKey: string | null;
  onSelect: (key: string) => void;
  onClose: (key: string) => void;
}

export function TabBar({ tabs, activeKey, onSelect, onClose }: TabBarProps) {
  return (
    <div style={barStyle}>
      {tabs.map((t) => {
        const on = t.key === activeKey;
        return (
          <div key={t.key} style={{ ...chipStyle, ...(on ? chipActive : null) }} onClick={() => onSelect(t.key)}>
            <span style={{ overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap", maxWidth: 140 }}>
              {t.title}
            </span>
            <button
              aria-label={`${t.title} 닫기`}
              style={closeBtn}
              onClick={(e) => {
                e.stopPropagation();
                onClose(t.key);
              }}
            >
              ×
            </button>
          </div>
        );
      })}
    </div>
  );
}

const barStyle: CSSProperties = {
  display: "flex",
  gap: 4,
  padding: "6px 8px 0",
  overflowX: "auto",
  borderBottom: "1px solid var(--border)",
  background: "var(--bg)",
};
const chipStyle: CSSProperties = {
  display: "inline-flex",
  alignItems: "center",
  gap: 6,
  padding: "6px 8px 6px 10px",
  fontSize: 12.5,
  color: "var(--text2)",
  background: "var(--card)",
  border: "1px solid var(--border)",
  borderBottom: "none",
  borderRadius: "var(--radius-sm) var(--radius-sm) 0 0",
  cursor: "pointer",
  flex: "0 0 auto",
};
const chipActive: CSSProperties = { color: "var(--text)", background: "var(--input)", fontWeight: 600 };
const closeBtn: CSSProperties = {
  border: "none",
  background: "transparent",
  color: "var(--text2)",
  cursor: "pointer",
  fontSize: 14,
  lineHeight: 1,
  padding: "0 2px",
};
