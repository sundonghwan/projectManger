import { useEffect, useRef, useState, type CSSProperties, type KeyboardEvent } from "react";
import type { Block, BlockType } from "../domain/types";
import { parseContent } from "../domain/blockContent";
import { Icon } from "../ui/icons/Icon";

export interface BlockEditorProps {
  blocks: Block[];
  onChangeText: (block: Block, text: string) => void;
  onToggleCheck: (block: Block) => void;
  onAddBlock: (type: BlockType) => void;
  /** 특정 블록 다음에 새 블록 삽입(Enter 이어쓰기). 생성된 블록 반환. */
  onAddBlockAfter: (block: Block, type?: BlockType) => Promise<Block | null>;
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

export function BlockEditor({
  blocks,
  onChangeText,
  onToggleCheck,
  onAddBlock,
  onAddBlockAfter,
  onDelete,
}: BlockEditorProps) {
  const inputRefs = useRef<Map<number, HTMLInputElement | HTMLTextAreaElement>>(new Map());
  const [focusId, setFocusId] = useState<number | null>(null);

  const setRef = (id: number, el: HTMLInputElement | HTMLTextAreaElement | null) => {
    if (el) inputRefs.current.set(id, el);
    else inputRefs.current.delete(id);
  };

  // 새로 만든(또는 지정된) 블록 입력에 포커스 + 커서를 끝으로 이동.
  useEffect(() => {
    if (focusId == null) return;
    const el = inputRefs.current.get(focusId);
    if (el) {
      el.focus();
      const len = el.value.length;
      el.setSelectionRange(len, len);
      setFocusId(null);
    }
  }, [blocks, focusId]);

  // Enter: 같은 종류(체크리스트)면 항목 이어가기, 그 외엔 문단으로 줄바꿈. Shift+Enter는 통과.
  const handleEnter = (e: KeyboardEvent, block: Block, nextType: BlockType) => {
    if (e.key !== "Enter" || e.shiftKey) return;
    e.preventDefault();
    void onAddBlockAfter(block, nextType).then((nb) => {
      if (nb) setFocusId(nb.id);
    });
  };

  return (
    <div style={{ maxWidth: 720, margin: "0 auto", padding: "24px 24px 80px" }}>
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
                    {data.checked ? <Icon name="check" size={12} strokeWidth={2.5} /> : null}
                  </button>
                  <input
                    ref={(el) => setRef(b.id, el)}
                    aria-label="블록 텍스트"
                    value={data.text}
                    onChange={(e) => onChangeText(b, e.target.value)}
                    onKeyDown={(e) => handleEnter(e, b, "checklist")}
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
                  ref={(el) => setRef(b.id, el)}
                  aria-label="블록 텍스트"
                  value={data.text}
                  onChange={(e) => onChangeText(b, e.target.value)}
                  placeholder="코드"
                  style={codeArea}
                />
              ) : (
                <input
                  ref={(el) => setRef(b.id, el)}
                  aria-label="블록 텍스트"
                  value={data.text}
                  onChange={(e) => onChangeText(b, e.target.value)}
                  onKeyDown={(e) => handleEnter(e, b, "paragraph")}
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
              <Icon name="close" size={15} />
            </button>
          </div>
        );
      })}

      <div style={{ display: "flex", flexWrap: "wrap", gap: 6, marginTop: 16 }}>
        {ADDABLE.map((a) => (
          <button key={a.type} onClick={() => void onAddBlock(a.type)} style={addBlockBtn}>
            <Icon name="plus" size={13} /> {a.label}
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
    display: "flex",
    alignItems: "center",
    justifyContent: "center",
    borderRadius: 4,
    border: `1.5px solid ${checked ? "#22c55e" : "var(--border)"}`,
    background: checked ? "#22c55e" : "transparent",
    color: "#fff",
    cursor: "pointer",
    padding: 0,
  };
}
const deleteBtn: CSSProperties = {
  display: "inline-flex",
  alignItems: "center",
  justifyContent: "center",
  border: "none",
  background: "transparent",
  color: "var(--text3)",
  cursor: "pointer",
  lineHeight: 1,
  padding: "2px 4px",
};
const addBlockBtn: CSSProperties = {
  display: "inline-flex",
  alignItems: "center",
  gap: 4,
  border: "1px solid var(--border)",
  background: "transparent",
  color: "var(--text2)",
  borderRadius: 20,
  padding: "4px 10px",
  fontSize: 12,
  cursor: "pointer",
};
