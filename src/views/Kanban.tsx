import type { CSSProperties, DragEvent } from "react";
import { KANBAN_STATUSES, type KanbanColumn } from "../domain/kanban";
import type { Label, TaskStatus } from "../domain/types";
import { computeSortOrder } from "../domain/sortOrder";
import { TASK_STATUS_COLOR, TASK_STATUS_LABEL, priorityColor, priorityLabel } from "../ui/colors";
import { LabelChips } from "./LabelChips";

export interface KanbanProps {
  columns: KanbanColumn[];
  onMove: (taskId: string, status: TaskStatus, sortOrder: number) => void;
  onAddTask: (status: TaskStatus) => void;
  labelsByTask?: Record<string, Label[]>;
  onCardClick?: (taskId: string) => void;
}

export function Kanban({ columns, onMove, onAddTask, labelsByTask = {}, onCardClick }: KanbanProps) {
  const moveBetween = (taskId: string, col: KanbanColumn, targetTaskId: string, position: "before" | "after") => {
    if (taskId === targetTaskId) return;
    const tasks = col.tasks.filter((t) => t.id !== taskId);
    const targetIndex = tasks.findIndex((t) => t.id === targetTaskId);
    if (targetIndex < 0) return;
    const before = position === "before" ? tasks[targetIndex - 1] : tasks[targetIndex];
    const after = position === "before" ? tasks[targetIndex] : tasks[targetIndex + 1];
    onMove(taskId, col.status, computeSortOrder(before?.sortOrder ?? null, after?.sortOrder ?? null));
  };

  const handleDrop = (col: KanbanColumn) => (e: DragEvent) => {
    e.preventDefault();
    const id = e.dataTransfer.getData("text/plain");
    if (!id) return;
    const last = col.tasks[col.tasks.length - 1];
    const sortOrder = computeSortOrder(last ? last.sortOrder : null, null);
    onMove(id, col.status, sortOrder);
  };

  const moveToStatus = (taskId: string, status: TaskStatus) => {
    const target = columns.find((col) => col.status === status);
    const last = target?.tasks[target.tasks.length - 1];
    const sortOrder = computeSortOrder(last ? last.sortOrder : null, null);
    onMove(taskId, status, sortOrder);
  };

  const handleCardDrop = (col: KanbanColumn, targetTaskId: string) => (e: DragEvent<HTMLDivElement>) => {
    e.preventDefault();
    e.stopPropagation();
    const id = e.dataTransfer.getData("text/plain");
    if (!id) return;
    const rect = e.currentTarget.getBoundingClientRect();
    const pointerY = e.clientY || e.pageY || e.screenY;
    const position = pointerY < rect.top + rect.height / 2 ? "before" : "after";
    moveBetween(id, col, targetTaskId, position);
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
                onDragOver={(e) => e.preventDefault()}
                onDrop={handleCardDrop(col, t.id)}
                onClick={() => onCardClick?.(t.id)}
                data-testid={`card-${t.id}`}
                style={cardStyle}
              >
                <div style={{ fontSize: 13.5, fontWeight: 500, marginBottom: 9, lineHeight: 1.35 }}>
                  {t.title}
                </div>
                <LabelChips labels={labelsByTask[t.id]} />
                <div style={{ display: "flex", alignItems: "center", flexWrap: "wrap", gap: 5, marginTop: 6 }}>
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
                  <select
                    aria-label={`${t.title} 상태`}
                    value={t.status}
                    onClick={(e) => e.stopPropagation()}
                    onMouseDown={(e) => e.stopPropagation()}
                    onChange={(e) => {
                      const next = e.target.value as TaskStatus;
                      if (next !== t.status) moveToStatus(t.id, next);
                    }}
                    style={statusSelect}
                  >
                    {KANBAN_STATUSES.map((status) => (
                      <option key={status} value={status}>
                        {TASK_STATUS_LABEL[status]}
                      </option>
                    ))}
                  </select>
                  {(t.startDate || t.dueDate) && (
                    <span style={dateRange}>
                      {t.startDate ?? ""}
                      {t.startDate && t.dueDate ? " → " : ""}
                      {t.dueDate ?? ""}
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
const statusSelect: CSSProperties = {
  border: "1px solid var(--border)",
  borderRadius: 5,
  background: "var(--input)",
  color: "var(--text2)",
  fontSize: 11,
  fontWeight: 600,
  padding: "2px 5px",
  cursor: "pointer",
};
const dateRange: CSSProperties = {
  fontSize: 11,
  fontWeight: 600,
  color: "var(--text2)",
};
