import type { Task } from "../domain/types";
import { buildTimeline } from "../domain/timeline";
import { TASK_STATUS_COLOR, TASK_STATUS_LABEL } from "../ui/colors";

export interface TimelineViewProps {
  tasks: Task[];
}

export function TimelineView({ tasks }: TimelineViewProps) {
  const { items, minDate, maxDate } = buildTimeline(tasks);

  if (items.length === 0) {
    return (
      <div style={{ padding: "24px 28px", color: "var(--text3)", fontSize: 13 }}>
        마감일이 있는 태스크가 없습니다. 태스크에 마감을 지정하면 타임라인에 표시됩니다.
      </div>
    );
  }

  return (
    <div style={{ padding: "24px 28px", maxWidth: 900 }}>
      <div style={{ display: "flex", justifyContent: "space-between", fontSize: 12, color: "var(--text2)", marginBottom: 10 }}>
        <span>{minDate}</span>
        <span>{maxDate}</span>
      </div>
      {items.map((it) => (
        <div key={it.id} style={{ display: "flex", alignItems: "center", gap: 10, height: 32 }} data-testid={`tl-${it.id}`}>
          <span style={{ width: 180, fontSize: 13, overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
            {it.title}
          </span>
          <div style={{ flex: 1, position: "relative", height: 8, background: "var(--hover)", borderRadius: 6 }}>
            <div
              title={`${TASK_STATUS_LABEL[it.status]} · ${it.dueDate}`}
              style={{
                position: "absolute",
                left: `calc(${it.ratio * 100}% - 6px)`,
                top: -3,
                width: 14,
                height: 14,
                borderRadius: "50%",
                background: TASK_STATUS_COLOR[it.status],
                border: "2px solid var(--bg)",
              }}
            />
          </div>
          <span style={{ width: 84, fontSize: 12, color: "var(--text2)", textAlign: "right" }}>{it.dueDate}</span>
        </div>
      ))}
    </div>
  );
}
