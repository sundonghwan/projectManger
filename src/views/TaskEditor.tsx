import { useState, type CSSProperties } from "react";
import type { Label, Priority, Task } from "../domain/types";
import { priorityLabel } from "../ui/colors";
import { Icon } from "../ui/icons/Icon";
import { LabelChips } from "./LabelChips";

export interface TaskPatch {
  title: string;
  priority: Priority;
  dueDate: string | null;
  description: string | null;
}

export interface TaskEditorProps {
  task: Task;
  labels: Label[];
  onSave: (patch: TaskPatch) => void;
  onAddLabel: (name: string, color: string) => void;
  onRemoveLabel: (label: Label) => void;
  onArchive: () => void;
  onClose: () => void;
}

const LABEL_COLORS = ["#3b82f6", "#22c55e", "#f97316", "#ef4444", "#a855f7", "#94a3b8"];

export function TaskEditor({ task, labels, onSave, onAddLabel, onRemoveLabel, onArchive, onClose }: TaskEditorProps) {
  const [title, setTitle] = useState(task.title);
  const [priority, setPriority] = useState<Priority>(task.priority);
  const [dueDate, setDueDate] = useState(task.dueDate ?? "");
  const [description, setDescription] = useState(task.description ?? "");
  const [labelName, setLabelName] = useState("");
  const [labelColor, setLabelColor] = useState(LABEL_COLORS[0]);

  const save = () => {
    onSave({
      title: title.trim() || task.title,
      priority,
      dueDate: dueDate || null,
      description: description || null,
    });
  };

  const addLabel = () => {
    if (!labelName.trim()) return;
    onAddLabel(labelName.trim(), labelColor);
    setLabelName("");
  };

  return (
    <div style={overlay} data-testid="task-editor-overlay">
      <div style={modal} role="dialog" aria-label="태스크 편집" onClick={(e) => e.stopPropagation()}>
        <div style={{ display: "flex", alignItems: "center", marginBottom: 14 }}>
          <span style={{ fontSize: 15, fontWeight: 600, flex: 1 }}>태스크 편집</span>
          <button aria-label="닫기" onClick={onClose} style={iconBtn}><Icon name="close" size={16} /></button>
        </div>

        <label style={fieldLabel}>제목</label>
        <input aria-label="제목" value={title} onChange={(e) => setTitle(e.target.value)} style={input} />

        <div style={{ display: "flex", gap: 12 }}>
          <div style={{ flex: 1 }}>
            <label style={fieldLabel}>우선순위</label>
            <select
              aria-label="우선순위"
              value={priority}
              onChange={(e) => setPriority(Number(e.target.value) as Priority)}
              style={input}
            >
              {[0, 1, 2, 3, 4].map((p) => (
                <option key={p} value={p}>
                  {priorityLabel(p as Priority)}
                </option>
              ))}
            </select>
          </div>
          <div style={{ flex: 1 }}>
            <label style={fieldLabel}>마감</label>
            <input
              aria-label="마감"
              type="date"
              value={dueDate}
              onChange={(e) => setDueDate(e.target.value)}
              style={input}
            />
          </div>
        </div>

        <label style={fieldLabel}>설명</label>
        <textarea
          aria-label="설명"
          value={description}
          onChange={(e) => setDescription(e.target.value)}
          style={{ ...input, minHeight: 70, resize: "vertical" }}
        />

        <label style={fieldLabel}>라벨</label>
        <div style={{ marginBottom: 8 }}>
          <LabelChips labels={labels} onRemove={onRemoveLabel} />
        </div>
        <div style={{ display: "flex", gap: 6, marginBottom: 16 }}>
          <input
            aria-label="라벨 이름"
            value={labelName}
            onChange={(e) => setLabelName(e.target.value)}
            placeholder="새 라벨"
            style={{ ...input, flex: 1, marginBottom: 0 }}
          />
          <select
            aria-label="라벨 색상"
            value={labelColor}
            onChange={(e) => setLabelColor(e.target.value)}
            style={{ ...input, width: 70, marginBottom: 0 }}
          >
            {LABEL_COLORS.map((c) => (
              <option key={c} value={c}>
                {c}
              </option>
            ))}
          </select>
          <button onClick={addLabel} style={secondaryBtn}>추가</button>
        </div>

        <div style={{ display: "flex", gap: 8 }}>
          <button onClick={save} style={primaryBtn}>저장</button>
          <button onClick={onArchive} style={dangerBtn}>보관</button>
        </div>
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
  width: 460,
  maxWidth: "90vw",
  background: "var(--card)",
  border: "1px solid var(--border)",
  borderRadius: "var(--radius-lg)",
  boxShadow: "var(--shadow-modal)",
  padding: 20,
};
const fieldLabel: CSSProperties = {
  display: "block",
  fontSize: 12,
  fontWeight: 600,
  color: "var(--text2)",
  margin: "10px 0 4px",
};
const input: CSSProperties = {
  width: "100%",
  border: "1px solid var(--border)",
  borderRadius: "var(--radius-md)",
  background: "var(--input)",
  color: "var(--text)",
  padding: "7px 10px",
  fontSize: 13,
  marginBottom: 4,
  fontFamily: "inherit",
};
const iconBtn: CSSProperties = { display: "inline-flex", alignItems: "center", justifyContent: "center", border: "none", background: "transparent", color: "var(--text2)", cursor: "pointer", padding: 0 };
const primaryBtn: CSSProperties = {
  flex: 1,
  border: "none",
  background: "var(--accent)",
  color: "#fff",
  borderRadius: "var(--radius-md)",
  padding: "9px 0",
  fontSize: 13,
  fontWeight: 600,
  cursor: "pointer",
};
const secondaryBtn: CSSProperties = {
  border: "1px solid var(--border)",
  background: "var(--bg)",
  color: "var(--text)",
  borderRadius: "var(--radius-md)",
  padding: "0 12px",
  fontSize: 13,
  cursor: "pointer",
};
const dangerBtn: CSSProperties = {
  border: "1px solid var(--border)",
  background: "var(--bg)",
  color: "var(--st-danger)",
  borderRadius: "var(--radius-md)",
  padding: "9px 16px",
  fontSize: 13,
  fontWeight: 600,
  cursor: "pointer",
};
