import { useState, type CSSProperties } from "react";
import type { BusinessType } from "../domain/types";
import { TYPE_COLOR, TYPE_LABEL } from "../ui/colors";

/** 노드에 추가할 수 있는 하위 항목 종류 */
export type AddKind = "project" | "document" | "deliverable";

const KIND_META: Record<AddKind, { label: string; icon: string }> = {
  project: { label: "프로젝트", icon: "📁" },
  document: { label: "문서", icon: "📄" },
  deliverable: { label: "산출물", icon: "📦" },
};

const BIZ_TYPES: BusinessType[] = ["si", "internal", "ops", "etc"];

export interface CreatePopoverProps {
  x: number;
  y: number;
  variant: "business" | "child";
  /** child 변형에서 선택 가능한 종류 */
  allowedKinds?: AddKind[];
  onCreateBusiness?: (type: BusinessType, name: string) => void;
  onCreateChild?: (kind: AddKind, name: string) => void;
  onClose: () => void;
}

/** 추가 생성 폼 — 사용자가 이름(과 사업 유형)을 직접 입력해서 만든다. */
export function CreatePopover(props: CreatePopoverProps) {
  const { x, y, variant, allowedKinds = [], onCreateBusiness, onCreateChild, onClose } = props;
  const [name, setName] = useState("");
  const [type, setType] = useState<BusinessType>("etc");
  const [kind, setKind] = useState<AddKind>(allowedKinds[0] ?? "document");

  const title = variant === "business" ? "새 사업" : "새 항목";

  const submit = () => {
    const n = name.trim();
    if (!n) return;
    if (variant === "business") onCreateBusiness?.(type, n);
    else onCreateChild?.(kind, n);
    onClose();
  };

  return (
    <div style={overlay} onClick={onClose} data-testid="create-overlay">
      <div
        role="dialog"
        aria-label={title}
        style={{ ...box, left: x, top: y }}
        onClick={(e) => e.stopPropagation()}
      >
        <div style={{ fontSize: 12, fontWeight: 600, color: "var(--text2)", marginBottom: 8 }}>
          {title}
        </div>

        {variant === "business" ? (
          <div style={chips}>
            {BIZ_TYPES.map((t) => {
              const active = t === type;
              return (
                <button
                  key={t}
                  aria-pressed={active}
                  onClick={() => setType(t)}
                  style={chip(active, TYPE_COLOR[t])}
                >
                  <span style={{ width: 7, height: 7, borderRadius: "50%", background: TYPE_COLOR[t] }} />
                  {TYPE_LABEL[t]}
                </button>
              );
            })}
          </div>
        ) : (
          <div style={chips}>
            {allowedKinds.map((k) => {
              const active = k === kind;
              return (
                <button
                  key={k}
                  aria-pressed={active}
                  onClick={() => setKind(k)}
                  style={chip(active, "var(--accent)")}
                >
                  <span>{KIND_META[k].icon}</span>
                  {KIND_META[k].label}
                </button>
              );
            })}
          </div>
        )}

        <div style={{ display: "flex", gap: 6, marginTop: 10 }}>
          <input
            autoFocus
            aria-label="이름"
            placeholder={variant === "business" ? "사업 이름" : `${KIND_META[kind].label} 이름`}
            value={name}
            onChange={(e) => setName(e.target.value)}
            onKeyDown={(e) => {
              if (e.key === "Enter") submit();
              else if (e.key === "Escape") onClose();
            }}
            style={input}
          />
          <button onClick={submit} disabled={!name.trim()} style={createBtn(!name.trim())}>
            만들기
          </button>
        </div>
      </div>
    </div>
  );
}

const overlay: CSSProperties = { position: "fixed", inset: 0, zIndex: 200 };
const box: CSSProperties = {
  position: "fixed",
  width: 260,
  background: "var(--card)",
  border: "1px solid var(--border)",
  borderRadius: "var(--radius-md)",
  boxShadow: "var(--shadow-popover)",
  padding: 12,
};
const chips: CSSProperties = { display: "flex", flexWrap: "wrap", gap: 6 };
function chip(active: boolean, color: string): CSSProperties {
  return {
    display: "flex",
    alignItems: "center",
    gap: 5,
    fontSize: 12,
    padding: "4px 9px",
    borderRadius: 20,
    border: `1px solid ${active ? color : "var(--border)"}`,
    background: active ? color + "22" : "transparent",
    color: active ? color : "var(--text2)",
    cursor: "pointer",
  };
}
const input: CSSProperties = {
  flex: 1,
  minWidth: 0,
  border: "1px solid var(--border)",
  borderRadius: "var(--radius-md)",
  background: "var(--input)",
  color: "var(--text)",
  padding: "6px 9px",
  fontSize: 13,
  fontFamily: "inherit",
};
function createBtn(disabled: boolean): CSSProperties {
  return {
    border: "none",
    background: disabled ? "var(--border)" : "var(--accent)",
    color: "#fff",
    borderRadius: "var(--radius-md)",
    padding: "6px 12px",
    fontSize: 12,
    fontWeight: 600,
    cursor: disabled ? "default" : "pointer",
  };
}
