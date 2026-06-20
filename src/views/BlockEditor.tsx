import type { CSSProperties } from "react";
import type { Block, BlockType } from "../domain/types";
import { parseContent } from "../domain/blockContent";

export interface BlockEditorProps {
  blocks: Block[];
  onChangeText: (block: Block, text: string) => void;
  onToggleCheck: (block: Block) => void;
  onAddBlock: (type: BlockType) => void;
  onDelete: (block: Block) => void;
}

const ADDABLE: { type: BlockType; label: string }[] = [
  { type: "paragraph", label: "텍스트" },
  { type: "heading", label: "제목" },
  { type: "checklist", label: "체크리스트" },
  { type: "code", label: "코드" },
  { type: "quote", label: "인용" },
  { type: "divider", label: "구분선" },
];

export function BlockEditor({ blocks, onChangeText, onToggleCheck, onAddBlock, onDelete }: BlockEditorProps) {
  return (
    <div style={{ maxWidth: 720, margin: "0 auto", padding: "24px 24px 80px" }}>
      {blocks.length === 0 && (
        <div style={{ color: "var(--text3)", fontSize: 14, marginBottom: 12 }}>
          빈 문서입니다. 아래에서 블록을 추가하세요.
        </div>
      )}

      {blocks.map((b) => {
        const data = parseContent(b.content);
        return (
          <div key={b.id} data-testid={`block-${b.id}`} style={blockRow}>
            <div style={{ flex: 1, minWidth: 0 }}>
              {b.type === "divider" ? (
                <hr style={{ border: "none", borderTop: "1px solid var(--border)", margin: "12px 0" }} />
              ) : b.type === "checklist" ? (
                <div style={{ display: "flex", alignItems: "center", gap: 9 }}>
                  <button
                    role="checkbox"
                    aria-checked={data.checked}
                    aria-label="체크 토글"
                    onClick={() => onToggleCheck(b)}
                    style={checkboxStyle(data.checked)}
                  >
                    {data.checked ? "✓" : ""}
                  </button>
                  <input
                    aria-label="블록 텍스트"
                    value={data.text}
                    onChange={(e) => onChangeText(b, e.target.value)}
                    placeholder="할 일"
                    style={{
                      ...textInput,
                      textDecoration: data.checked ? "line-through" : "none",
                      color: data.checked ? "var(--text2)" : "var(--text)",
                    }}
                  />
                </div>
              ) : b.type === "code" ? (
                <textarea
                  aria-label="블록 텍스트"
                  value={data.text}
                  onChange={(e) => onChangeText(b, e.target.value)}
                  placeholder="코드"
                  style={codeArea}
                />
              ) : (
                <input
                  aria-label="블록 텍스트"
                  value={data.text}
                  onChange={(e) => onChangeText(b, e.target.value)}
                  placeholder={b.type === "heading" ? "제목" : "텍스트"}
                  style={{
                    ...textInput,
                    fontSize: b.type === "heading" ? 20 : 15,
                    fontWeight: b.type === "heading" ? 700 : 400,
                    fontStyle: b.type === "quote" ? "italic" : "normal",
                    color: b.type === "quote" ? "var(--text2)" : "var(--text)",
                    borderLeft: b.type === "quote" ? "3px solid var(--accent)" : "none",
                    paddingLeft: b.type === "quote" ? 12 : 0,
                  }}
                />
              )}
            </div>
            <button aria-label="블록 삭제" onClick={() => onDelete(b)} style={deleteBtn}>
              ×
            </button>
          </div>
        );
      })}

      <div style={{ display: "flex", flexWrap: "wrap", gap: 6, marginTop: 16 }}>
        {ADDABLE.map((a) => (
          <button key={a.type} onClick={() => onAddBlock(a.type)} style={addBlockBtn}>
            + {a.label}
          </button>
        ))}
      </div>
    </div>
  );
}

const blockRow: CSSProperties = {
  display: "flex",
  alignItems: "flex-start",
  gap: 6,
  padding: "2px 0",
};
const textInput: CSSProperties = {
  width: "100%",
  border: "none",
  outline: "none",
  background: "transparent",
  color: "var(--text)",
  fontSize: 15,
  lineHeight: 1.6,
  padding: "3px 0",
  fontFamily: "inherit",
};
const codeArea: CSSProperties = {
  width: "100%",
  minHeight: 60,
  border: "1px solid var(--border)",
  borderRadius: 6,
  background: "var(--sidebar)",
  color: "var(--text)",
  fontFamily: "var(--font-mono)",
  fontSize: 13,
  padding: "10px 12px",
  resize: "vertical",
};
function checkboxStyle(checked: boolean): CSSProperties {
  return {
    width: 17,
    height: 17,
    flexShrink: 0,
    borderRadius: 4,
    border: `1.5px solid ${checked ? "#22c55e" : "var(--border)"}`,
    background: checked ? "#22c55e" : "transparent",
    color: "#fff",
    fontSize: 11,
    cursor: "pointer",
    padding: 0,
  };
}
const deleteBtn: CSSProperties = {
  border: "none",
  background: "transparent",
  color: "var(--text3)",
  cursor: "pointer",
  fontSize: 16,
  lineHeight: 1,
  padding: "2px 4px",
};
const addBlockBtn: CSSProperties = {
  border: "1px solid var(--border)",
  background: "transparent",
  color: "var(--text2)",
  borderRadius: 20,
  padding: "4px 10px",
  fontSize: 12,
  cursor: "pointer",
};
