import type { Business, Project } from "../domain/types";
import { businessColor } from "../ui/colors";
import { useTasks } from "../hooks/useTasks";
import { dashboardStats } from "../domain/dashboard";
import { groupByStatus } from "../domain/kanban";
import { Dashboard } from "./Dashboard";
import { Kanban } from "./Kanban";
import { TaskList } from "./TaskList";

export type ViewKind = "dashboard" | "kanban" | "list" | "doc" | "deliverables" | "ssh";

const TABS: { key: ViewKind; label: string }[] = [
  { key: "dashboard", label: "대시보드" },
  { key: "kanban", label: "칸반" },
  { key: "list", label: "리스트" },
  { key: "doc", label: "문서" },
  { key: "deliverables", label: "산출물" },
  { key: "ssh", label: "터미널" },
];

const VIEW_LABEL: Record<ViewKind, string> = {
  dashboard: "대시보드",
  kanban: "칸반",
  list: "리스트",
  doc: "문서",
  deliverables: "산출물",
  ssh: "SSH 터미널",
};

export interface MainViewProps {
  business: Business | null;
  project: Project | null;
  view: ViewKind;
  onViewChange: (v: ViewKind) => void;
  error: string | null;
}

export function MainView({ business, project, view, onViewChange, error }: MainViewProps) {
  const { tasks, error: taskError, create, move, toggleDone } = useTasks(
    business?.id ?? null,
    project?.id ?? null,
  );
  const shownError = error ?? taskError;

  return (
    <section
      style={{
        flex: 1,
        height: "100%",
        display: "flex",
        flexDirection: "column",
        minWidth: 0,
        background: "var(--bg)",
      }}
    >
      <div style={topbar}>
        {business ? (
          <div style={{ display: "flex", alignItems: "center", gap: 6 }}>
            <span
              style={{
                width: 8,
                height: 8,
                borderRadius: "50%",
                background: businessColor(business.type, business.color),
              }}
            />
            <span style={{ fontSize: 13, fontWeight: 600 }}>{business.name}</span>
            {project && (
              <>
                <span style={{ color: "var(--text2)" }}>›</span>
                <span style={{ fontSize: 13 }}>{project.name}</span>
              </>
            )}
            <span style={{ color: "var(--text2)" }}>›</span>
            <span style={{ fontSize: 13, color: "var(--text2)" }}>{VIEW_LABEL[view]}</span>
          </div>
        ) : (
          <span style={{ fontSize: 13, color: "var(--text2)" }}>선택된 항목 없음</span>
        )}
      </div>

      {business && (
        <div style={tabbar}>
          {TABS.map((t) => {
            const active = t.key === view;
            return (
              <button
                key={t.key}
                onClick={() => onViewChange(t.key)}
                style={{
                  ...tabStyle,
                  color: active ? "var(--text)" : "var(--text2)",
                  fontWeight: active ? 600 : 500,
                  borderBottom: `2px solid ${active ? "var(--accent)" : "transparent"}`,
                }}
              >
                {t.label}
              </button>
            );
          })}
        </div>
      )}

      <div style={{ flex: 1, overflow: "auto", minHeight: 0 }}>
        {shownError && <div style={errorBar}>⚠ {shownError}</div>}
        {!business ? (
          <Empty />
        ) : view === "dashboard" ? (
          <Dashboard business={business} stats={dashboardStats(tasks)} />
        ) : view === "kanban" ? (
          <Kanban columns={groupByStatus(tasks)} onMove={move} onAddTask={(s) => create(s)} />
        ) : view === "list" ? (
          <TaskList tasks={tasks} onToggleDone={toggleDone} />
        ) : (
          <div style={{ padding: "24px 28px", color: "var(--text2)" }}>
            <strong style={{ color: "var(--text)" }}>{VIEW_LABEL[view]}</strong> — 다음 단계에서 지원
          </div>
        )}
      </div>
    </section>
  );
}

function Empty() {
  return (
    <div style={{ padding: "60px 28px", textAlign: "center", color: "var(--text3)" }}>
      <div style={{ fontSize: 15, marginBottom: 8 }}>
        왼쪽에서 사업을 선택하거나 새로 만들어 시작하세요.
      </div>
    </div>
  );
}

const topbar: React.CSSProperties = {
  height: 44,
  flexShrink: 0,
  display: "flex",
  alignItems: "center",
  padding: "0 14px",
  borderBottom: "1px solid var(--border)",
};
const tabbar: React.CSSProperties = {
  height: 40,
  flexShrink: 0,
  display: "flex",
  gap: 2,
  padding: "0 10px",
  borderBottom: "1px solid var(--border)",
};
const tabStyle: React.CSSProperties = {
  padding: "0 11px",
  fontSize: 13,
  border: "none",
  background: "transparent",
  cursor: "pointer",
  marginBottom: -1,
};
const errorBar: React.CSSProperties = {
  margin: "12px 16px",
  padding: "8px 12px",
  borderRadius: "var(--radius-md)",
  background: "rgba(239,68,68,.1)",
  color: "#ef4444",
  fontSize: 13,
};
