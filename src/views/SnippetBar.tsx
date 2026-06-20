import { useState, type CSSProperties } from "react";
import type { CommandSnippet } from "../domain/types";

export interface SnippetBarProps {
  snippets: CommandSnippet[];
  onRun: (command: string) => void;
  onCreate: (name: string, command: string) => void;
  onDelete: (id: number) => void;
}

/** 터미널 명령 스니펫 바 — 즐겨찾는 명령 실행/추가/삭제. */
export function SnippetBar({ snippets, onRun, onCreate, onDelete }: SnippetBarProps) {
  const [adding, setAdding] = useState(false);
  const [name, setName] = useState("");
  const [command, setCommand] = useState("");

  const submit = () => {
    if (!name.trim() || !command.trim()) return;
    onCreate(name.trim(), command.trim());
    setName("");
    setCommand("");
    setAdding(false);
  };

  return (
    <div style={bar}>
      {snippets.map((s) => (
        <span key={s.id} style={chip} data-testid={`snippet-${s.id}`}>
          <button onClick={() => onRun(s.command)} style={runBtn} title={s.command}>
            {s.name}
          </button>
          <button onClick={() => onDelete(s.id)} aria-label={`${s.name} 삭제`} style={delBtn}>
            ×
          </button>
        </span>
      ))}
      {adding ? (
        <span style={{ display: "inline-flex", gap: 4 }}>
          <input aria-label="스니펫 이름" placeholder="이름" value={name} onChange={(e) => setName(e.target.value)} style={input} />
          <input aria-label="스니펫 명령" placeholder="명령" value={command} onChange={(e) => setCommand(e.target.value)} style={{ ...input, width: 160 }} />
          <button onClick={submit} style={runBtn}>저장</button>
        </span>
      ) : (
        <button onClick={() => setAdding(true)} style={addBtn} aria-label="스니펫 추가">+ 스니펫</button>
      )}
    </div>
  );
}

const bar: CSSProperties = {
  display: "flex",
  alignItems: "center",
  flexWrap: "wrap",
  gap: 6,
  padding: "6px 14px",
  borderBottom: "1px solid #26261f",
  background: "#141412",
};
const chip: CSSProperties = { display: "inline-flex", alignItems: "center", border: "1px solid #33332a", borderRadius: 5 };
const runBtn: CSSProperties = { background: "transparent", border: "none", color: "#a9c7a0", fontSize: 11, padding: "3px 8px", cursor: "pointer", fontFamily: "var(--font-mono)" };
const delBtn: CSSProperties = { background: "transparent", border: "none", color: "#8a8a80", fontSize: 12, padding: "0 6px 0 0", cursor: "pointer" };
const addBtn: CSSProperties = { background: "transparent", border: "1px dashed #33332a", borderRadius: 5, color: "#8a8a80", fontSize: 11, padding: "3px 9px", cursor: "pointer" };
const input: CSSProperties = { background: "#0d0d0c", border: "1px solid #33332a", borderRadius: 4, color: "#d8d8cf", fontSize: 11, padding: "3px 6px", width: 90, fontFamily: "var(--font-mono)" };
