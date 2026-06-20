import type { CSSProperties, DragEvent } from "react";
import type { KanbanColumn } from "../domain/kanban";
import type { TaskStatus } from "../domain/types";
import { computeSortOrder } from "../domain/sortOrder";
import { TASK_STATUS_COLOR, TASK_STATUS_LABEL, priorityColor, priorityLabel } from "../ui/colors";

export interface KanbanProps {
  columns: KanbanColumn[];
  onMove: (taskId: number, status: TaskStatus, sortOrder: number) => void;
  onAddTask: (status: TaskStatus) => void;
}

export function Kanban({ columns, onMove, onAddTask }: KanbanProps) {
  const handleDrop = (col: KanbanColumn) => (e: DragEvent) => {
    e.preventDefault();
    const id = Number(e.dataTransfer.getData("text/plain"));
    if (!id) return;
    const last = col.tasks[col.tasks.length - 1];
    const sortOrder = computeSortOrder(last ? last.sortOrder : null, null);
    onMove(id, col.status, sortOrder);
  };

  return (
    <div style={{ display: "flex", gap: 14, height: "100%", padding: "18px 20px", minHeight: 0 }}>
      {columns.map((col) => (
        <div
          key={col.status}
          data-testid={`col-${col.status}`}
          onDragOver={(e) => e.preventDefault()}
          onDrop={handleDrop(col)}
          style={columnStyle}
        >
          <div style={colHeader}>
            <span
              style={{
                width: 8,
                height: 8,
                borderRadius: "50%",
                background: TASK_STATUS_COLOR[col.status],
              }}
            />
            <span style={{ fontSize: 13, fontWeight: 600 }}>{TASK_STATUS_LABEL[col.status]}</span>
            <span style={countBadge}>{col.tasks.length}</span>
            <span style={{ flex: 1 }} />
            <button
              aria-label={`${TASK_STATUS_LABEL[col.status]}에 태스크 추가`}
              onClick={() => onAddTask(col.status)}
              style={addBtn}
            >
              +
            </button>
          </div>
          <div style={{ flex: 1, overflowY: "auto", padding: "0 8px 8px", display: "flex", flexDirection: "column", gap: 8 }}>
            {col.tasks.map((t) => (
              <div
                key={t.id}
                draggable
                onDragStart={(e) => e.dataTransfer.setData("text/plain", String(t.id))}
                data-testid={`card-${t.id}`}
                style={cardStyle}
              >
                <div style={{ fontSize: 13.5, fontWeight: 500, marginBottom: 9, lineHeight: 1.35 }}>
                  {t.title}
                </div>
                <div style={{ display: "flex", alignItems: "center", flexWrap: "wrap", gap: 5 }}>
                  {t.priority > 0 && (
                    <span
                      style={{
                        fontSize: 11,
                        fontWeight: 600,
                        padding: "2px 7px",
                        borderRadius: 4,
                        background: priorityColor(t.priority) + "22",
                        color: priorityColor(t.priority),
                      }}
                    >
                      {priorityLabel(t.priority)}
                    </span>
                  )}
                  <span style={{ flex: 1 }} />
                  {t.dueDate && (
                    <span style={{ fontSize: 11, fontWeight: 600, color: "var(--text2)" }}>
                      {t.dueDate}
                    </span>
                  )}
                </div>
              </div>
            ))}
          </div>
        </div>
      ))}
    </div>
  );
}

const columnStyle: CSSProperties = {
  flex: 1,
  minWidth: 0,
  background: "var(--sidebar)",
  border: "1px solid var(--border)",
  borderRadius: 8,
  display: "flex",
  flexDirection: "column",
  maxHeight: "100%",
};
const colHeader: CSSProperties = {
  display: "flex",
  alignItems: "center",
  gap: 7,
  padding: "11px 12px 9px",
};
const countBadge: CSSProperties = {
  fontSize: 12,
  color: "var(--text2)",
  background: "var(--hover)",
  borderRadius: 20,
  padding: "1px 7px",
  fontWeight: 600,
};
const addBtn: CSSProperties = {
  border: "none",
  background: "transparent",
  color: "var(--text2)",
  fontSize: 14,
  cursor: "pointer",
};
const cardStyle: CSSProperties = {
  background: "var(--card)",
  border: "1px solid var(--border)",
  borderRadius: 6,
  padding: "10px 11px",
  cursor: "grab",
  boxShadow: "0 1px 2px rgba(0,0,0,.04)",
};
