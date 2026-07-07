import { useId, useState, type CSSProperties } from "react";
import { normalizeBusinessType, type BusinessTypeOption } from "../domain/businessTypes";
import type { BusinessType } from "../domain/types";
import { Icon, type IconName } from "../ui/icons/Icon";

/** 노드에 추가할 수 있는 하위 항목 종류 */
export type AddKind = "project" | "document" | "deliverable" | "folder";

const KIND_META: Record<AddKind, { label: string; icon: IconName }> = {
  project: { label: "프로젝트", icon: "folder" },
  document: { label: "문서", icon: "document" },
  deliverable: { label: "산출물", icon: "deliverable" },
  folder: { label: "폴더", icon: "folder" },
};

export interface CreatePopoverProps {
  x: number;
  y: number;
  variant: "business" | "child";
  businessTypeOptions?: BusinessTypeOption[];
  /** child 변형에서 선택 가능한 종류 */
  allowedKinds?: AddKind[];
  onCreateBusiness?: (type: BusinessType, name: string) => void;
  onCreateChild?: (kind: AddKind, name: string) => void;
  onClose: () => void;
}

/** 추가 생성 폼 — 사용자가 이름(과 사업 유형)을 직접 입력해서 만든다. */
export function CreatePopover(props: CreatePopoverProps) {
  const {
    x,
    y,
    variant,
    businessTypeOptions = [],
    allowedKinds = [],
    onCreateBusiness,
    onCreateChild,
    onClose,
  } = props;
  const [name, setName] = useState("");
  const [type, setType] = useState<BusinessType>("");
  const [kind, setKind] = useState<AddKind>(allowedKinds[0] ?? "document");
  const datalistId = useId();

  const title = variant === "business" ? "새 사업" : "새 항목";
  const normalizedType = normalizeBusinessType(type);
  const disabled = !name.trim() || (variant === "business" && !normalizedType);

  const submit = () => {
    const n = name.trim();
    if (!n) return;
    if (variant === "business") {
      if (!normalizedType) return;
      onCreateBusiness?.(normalizedType, n);
    } else {
      onCreateChild?.(kind, n);
    }
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
          <>
            {businessTypeOptions.length > 0 && (
              <div style={chips}>
                {businessTypeOptions.map((option) => {
                  const active = option.type === normalizedType;
                  return (
                    <button
                      key={option.type}
                      aria-pressed={active}
                      onClick={() => setType(option.type)}
                      style={chip(active, option.color)}
                    >
                      <span style={{ width: 7, height: 7, borderRadius: "50%", background: option.color }} />
                      {option.label}
                    </button>
                  );
                })}
              </div>
            )}
            <input
              autoFocus
              aria-label="사업 유형"
              placeholder="사업 유형"
              list={datalistId}
              value={type}
              onChange={(e) => setType(e.target.value)}
              onKeyDown={(e) => {
                if (e.key === "Enter") submit();
                else if (e.key === "Escape") onClose();
              }}
              style={{ ...input, width: "100%", marginTop: businessTypeOptions.length > 0 ? 10 : 0 }}
            />
            <datalist id={datalistId}>
              {businessTypeOptions.map((option) => (
                <option key={option.type} value={option.type} />
              ))}
            </datalist>
          </>
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
                  <Icon name={KIND_META[k].icon} size={14} />
                  {KIND_META[k].label}
                </button>
              );
            })}
          </div>
        )}

        <div style={{ display: "flex", gap: 6, marginTop: 10 }}>
          <input
            autoFocus={variant !== "business"}
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
          <button onClick={submit} disabled={disabled} style={createBtn(disabled)}>
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
