import { useState, type CSSProperties } from "react";
import { normalizeBusinessType, type BusinessTypeOption } from "../domain/businessTypes";
import type { Business, EntityStatus } from "../domain/types";

export interface BusinessEditorInput {
  id: string;
  name: string;
  type: string;
  color?: string | null;
  status: EntityStatus;
  description?: string | null;
}

export interface BusinessEditorPopoverProps {
  business: Business;
  businessTypeOptions: BusinessTypeOption[];
  onSave: (input: BusinessEditorInput) => void;
  onClose: () => void;
}

const HEX_COLOR_RE = /^#[0-9a-f]{6}$/i;

export function BusinessEditorPopover({
  business,
  businessTypeOptions,
  onSave,
  onClose,
}: BusinessEditorPopoverProps) {
  const [name, setName] = useState(business.name);
  const [type, setType] = useState(business.type);
  const [color, setColor] = useState(business.color ?? "");
  const [status, setStatus] = useState<EntityStatus>(business.status);
  const [description, setDescription] = useState(business.description ?? "");

  const normalizedType = normalizeBusinessType(type);
  const valid = name.trim().length > 0 && normalizedType.length > 0;
  const pickerColor = HEX_COLOR_RE.test(color) ? color : "#64748b";

  const save = () => {
    if (!valid) return;
    onSave({
      id: business.id,
      name: name.trim(),
      type: normalizedType,
      color: color.trim() || null,
      status,
      description: description.trim() || null,
    });
  };

  return (
    <div style={backdrop} onClick={onClose}>
      <div role="dialog" aria-label="사업 수정" style={box} onClick={(e) => e.stopPropagation()}>
        <div style={title}>사업 수정</div>
        <label style={field}>
          <span style={label}>이름</span>
          <input aria-label="사업 이름" value={name} onChange={(e) => setName(e.target.value)} style={input} />
        </label>
        <label style={field}>
          <span style={label}>유형</span>
          <input
            aria-label="사업 유형"
            list="business-edit-type-options"
            value={type}
            onChange={(e) => setType(e.target.value)}
            style={input}
          />
          <datalist id="business-edit-type-options">
            {businessTypeOptions.map((option) => (
              <option key={option.type} value={option.type} />
            ))}
          </datalist>
        </label>
        <label style={field}>
          <span style={label}>색상</span>
          <div style={colorRow}>
            <input aria-label="색상" value={color} onChange={(e) => setColor(e.target.value)} style={input} />
            <input
              aria-label="색상 선택"
              type="color"
              value={pickerColor}
              onChange={(e) => setColor(e.target.value)}
              style={colorInput}
            />
          </div>
        </label>
        <label style={field}>
          <span style={label}>상태</span>
          <select aria-label="상태" value={status} onChange={(e) => setStatus(e.target.value as EntityStatus)} style={input}>
            <option value="active">active</option>
            <option value="onhold">onhold</option>
            <option value="done">done</option>
          </select>
        </label>
        <label style={field}>
          <span style={label}>설명</span>
          <textarea
            aria-label="설명"
            value={description}
            onChange={(e) => setDescription(e.target.value)}
            style={textarea}
          />
        </label>
        <div style={actions}>
          <button type="button" onClick={onClose} style={secondary}>
            취소
          </button>
          <button type="button" onClick={save} disabled={!valid} style={{ ...primary, opacity: valid ? 1 : 0.5 }}>
            저장
          </button>
        </div>
      </div>
    </div>
  );
}

const backdrop: CSSProperties = {
  position: "fixed",
  inset: 0,
  zIndex: 220,
  background: "rgba(15, 23, 42, 0.18)",
};
const box: CSSProperties = {
  position: "fixed",
  left: 280,
  top: 88,
  width: 300,
  background: "var(--card)",
  border: "1px solid var(--border)",
  borderRadius: "var(--radius-md)",
  boxShadow: "var(--shadow-popover)",
  padding: 12,
};
const title: CSSProperties = { fontSize: 13, fontWeight: 700, marginBottom: 10 };
const field: CSSProperties = { display: "flex", flexDirection: "column", gap: 4, marginBottom: 8 };
const label: CSSProperties = { fontSize: 11, fontWeight: 600, color: "var(--text2)" };
const input: CSSProperties = {
  minWidth: 0,
  border: "1px solid var(--border)",
  borderRadius: "var(--radius-md)",
  background: "var(--input)",
  color: "var(--text)",
  padding: "6px 8px",
  fontSize: 13,
  fontFamily: "inherit",
};
const colorRow: CSSProperties = { display: "grid", gridTemplateColumns: "1fr 44px", gap: 6, alignItems: "center" };
const colorInput: CSSProperties = {
  width: 44,
  height: 30,
  border: "1px solid var(--border)",
  borderRadius: "var(--radius-sm)",
  background: "var(--bg)",
  padding: 2,
};
const textarea: CSSProperties = { ...input, minHeight: 64, resize: "vertical" };
const actions: CSSProperties = { display: "flex", justifyContent: "flex-end", gap: 6, marginTop: 10 };
const secondary: CSSProperties = {
  border: "1px solid var(--border)",
  background: "var(--bg)",
  color: "var(--text2)",
  borderRadius: "var(--radius-md)",
  padding: "6px 10px",
  cursor: "pointer",
};
const primary: CSSProperties = {
  border: "none",
  background: "var(--accent)",
  color: "#fff",
  borderRadius: "var(--radius-md)",
  padding: "6px 12px",
  fontWeight: 600,
  cursor: "pointer",
};
