import type { CSSProperties } from "react";
import type { Task } from "../domain/types";
import { TASK_STATUS_COLOR, TASK_STATUS_LABEL, priorityColor, priorityLabel } from "../ui/colors";

export interface TaskListProps {
  tasks: Task[];
  onToggleDone: (task: Task) => void;
}

const COLS = "34px 1fr 96px 84px 90px";

export function TaskList({ tasks, onToggleDone }: TaskListProps) {
  return (
    <div>
      <div style={{ ...rowGrid, ...headerStyle }}>
        <span />
        <span>제목</span>
        <span>상태</span>
        <span>우선순위</span>
        <span>마감</span>
      </div>
      {tasks.length === 0 && (
        <div style={{ padding: "16px 20px", color: "var(--text3)", fontSize: 13 }}>
          태스크가 없습니다.
        </div>
      )}
      {tasks.map((t) => {
        const done = t.status === "done";
        return (
          <div key={t.id} style={{ ...rowGrid, ...bodyRow }} data-testid={`row-${t.id}`}>
            <span>
              <button
                role="checkbox"
                aria-checked={done}
                aria-label={`${t.title} 완료 토글`}
                onClick={() => onToggleDone(t)}
                style={{
                  width: 15,
                  height: 15,
                  borderRadius: 4,
                  border: `1.5px solid ${done ? "#22c55e" : "var(--border)"}`,
                  background: done ? "#22c55e" : "transparent",
                  color: "#fff",
                  fontSize: 10,
                  cursor: "pointer",
                  padding: 0,
                }}
              >
                {done ? "✓" : ""}
              </button>
            </span>
            <span
              style={{
                fontSize: 13.5,
                overflow: "hidden",
                textOverflow: "ellipsis",
                whiteSpace: "nowrap",
                color: done ? "var(--text2)" : "var(--text)",
                textDecoration: done ? "line-through" : "none",
              }}
            >
              {t.title}
            </span>
            <span>
              <span
                style={{
                  fontSize: 11,
                  fontWeight: 600,
                  padding: "2px 8px",
                  borderRadius: 4,
                  background: TASK_STATUS_COLOR[t.status] + "22",
                  color: TASK_STATUS_COLOR[t.status],
                }}
              >
                {TASK_STATUS_LABEL[t.status]}
              </span>
            </span>
            <span style={{ fontSize: 12, fontWeight: 600, color: priorityColor(t.priority), display: "flex", alignItems: "center", gap: 5 }}>
              {t.priority > 0 && (
                <>
                  <span style={{ width: 6, height: 6, borderRadius: "50%", background: priorityColor(t.priority) }} />
                  {priorityLabel(t.priority)}
                </>
              )}
            </span>
            <span style={{ fontSize: 12, color: "var(--text2)" }}>{t.dueDate ?? ""}</span>
          </div>
        );
      })}
    </div>
  );
}

const rowGrid: CSSProperties = {
  display: "grid",
  gridTemplateColumns: COLS,
  alignItems: "center",
  padding: "0 20px",
};
const headerStyle: CSSProperties = {
  height: 34,
  borderBottom: "1px solid var(--border)",
  fontSize: 12,
  fontWeight: 600,
  color: "var(--text2)",
  position: "sticky",
  top: 0,
  background: "var(--bg)",
};
const bodyRow: CSSProperties = { height: 38, borderBottom: "1px solid var(--border)" };
