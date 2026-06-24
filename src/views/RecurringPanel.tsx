import { useState, type CSSProperties } from "react";
import type { RecurringTask } from "../domain/types";

export interface RecurringFormData {
  title: string;
  priority: number;
  intervalDays: number;
  nextRun: string;
}

export interface RecurringPanelProps {
  items: RecurringTask[];
  onCreate: (data: RecurringFormData) => void;
  onToggle: (item: RecurringTask) => void;
  onDelete: (id: string) => void;
  onGenerate: () => void;
}

export function RecurringPanel({ items, onCreate, onToggle, onDelete, onGenerate }: RecurringPanelProps) {
  const [title, setTitle] = useState("");
  const [intervalDays, setIntervalDays] = useState(7);
  const [nextRun, setNextRun] = useState("");

  const submit = () => {
    if (!title.trim() || !nextRun) return;
    onCreate({ title: title.trim(), priority: 2, intervalDays, nextRun });
    setTitle("");
    setNextRun("");
  };

  return (
    <div>
      <div style={{ display: "flex", alignItems: "center", marginBottom: 8 }}>
        <span style={{ fontSize: 13, fontWeight: 600, flex: 1 }}>반복 태스크</span>
        <button onClick={onGenerate} style={secondaryBtn}>지금 생성</button>
      </div>
      <div style={{ display: "flex", gap: 6, marginBottom: 10, flexWrap: "wrap" }}>
        <input aria-label="반복 제목" placeholder="제목" value={title} onChange={(e) => setTitle(e.target.value)} style={{ ...input, flex: 2 }} />
        <input aria-label="반복 간격" type="number" value={intervalDays} onChange={(e) => setIntervalDays(Number(e.target.value))} style={{ ...input, width: 64 }} />
        <input aria-label="다음 실행" type="date" value={nextRun} onChange={(e) => setNextRun(e.target.value)} style={{ ...input, width: 140 }} />
        <button onClick={submit} style={primaryBtn}>추가</button>
      </div>
      {items.length === 0 ? (
        <div style={{ color: "var(--text3)", fontSize: 12 }}>등록된 반복이 없습니다.</div>
      ) : (
        items.map((r) => (
          <div key={r.id} style={row} data-testid={`recurring-${r.id}`}>
            <span style={{ flex: 1, fontSize: 13, opacity: r.active ? 1 : 0.5 }}>
              {r.title} <span style={{ color: "var(--text3)", fontSize: 11 }}>· {r.intervalDays}일마다 · 다음 {r.nextRun}</span>
            </span>
            <button onClick={() => onToggle(r)} style={secondaryBtn}>{r.active ? "일시중지" : "재개"}</button>
            <button onClick={() => onDelete(r.id)} style={dangerBtn} aria-label={`${r.title} 삭제`}>삭제</button>
          </div>
        ))
      )}
    </div>
  );
}

const input: CSSProperties = { border: "1px solid var(--border)", borderRadius: "var(--radius-md)", background: "var(--input)", color: "var(--text)", padding: "6px 9px", fontSize: 13, fontFamily: "inherit" };
const row: CSSProperties = { display: "flex", alignItems: "center", gap: 6, padding: "7px 0", borderTop: "1px solid var(--border)" };
const primaryBtn: CSSProperties = { border: "none", background: "var(--accent)", color: "#fff", borderRadius: "var(--radius-md)", padding: "6px 12px", fontSize: 12, fontWeight: 600, cursor: "pointer" };
const secondaryBtn: CSSProperties = { border: "1px solid var(--border)", background: "var(--bg)", color: "var(--text)", borderRadius: "var(--radius-md)", padding: "5px 10px", fontSize: 12, cursor: "pointer" };
const dangerBtn: CSSProperties = { border: "1px solid var(--border)", background: "var(--bg)", color: "var(--st-danger)", borderRadius: "var(--radius-md)", padding: "5px 10px", fontSize: 12, cursor: "pointer" };
