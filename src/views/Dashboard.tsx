import type { CSSProperties } from "react";
import type { Business } from "../domain/types";
import type { DashboardStats } from "../domain/dashboard";
import { TASK_STATUS_COLOR, TASK_STATUS_LABEL, TYPE_LABEL, businessColor } from "../ui/colors";

export interface DashboardProps {
  business: Business;
  stats: DashboardStats;
}

export function Dashboard({ business, stats }: DashboardProps) {
  const pct = Math.round(stats.doneRatio * 100);
  const typeLabel = TYPE_LABEL[business.type] ?? business.type;
  return (
    <div style={{ padding: "24px 28px", maxWidth: 1000 }}>
      <div style={{ display: "flex", alignItems: "center", gap: 12, marginBottom: 4 }}>
        <h1 style={{ margin: 0, fontSize: 24, fontWeight: 700 }}>{business.name}</h1>
        <span
          style={{
            fontSize: 12,
            fontWeight: 600,
            padding: "3px 9px",
            borderRadius: 4,
            background: businessColor(business.type, business.color) + "22",
            color: businessColor(business.type, business.color),
          }}
        >
          {typeLabel}
        </span>
      </div>
      {business.description && (
        <div style={{ color: "var(--text2)", fontSize: 13, marginBottom: 20 }}>
          {business.description}
        </div>
      )}

      <div style={{ display: "grid", gridTemplateColumns: "repeat(3, 1fr)", gap: 14, marginTop: 12 }}>
        {/* 상태 카운트 */}
        <div style={{ ...card, gridColumn: "span 2" }}>
          <div style={cardTitle}>태스크 상태</div>
          <div style={{ display: "grid", gridTemplateColumns: "repeat(4, 1fr)", gap: 10 }}>
            {(["todo", "doing", "review", "done"] as const).map((s) => (
              <div key={s} style={countTile} data-testid={`count-${s}`}>
                <div style={{ display: "flex", alignItems: "center", gap: 5, color: "var(--text2)", fontSize: 12, marginBottom: 6 }}>
                  <span style={{ width: 7, height: 7, borderRadius: "50%", background: TASK_STATUS_COLOR[s] }} />
                  {TASK_STATUS_LABEL[s]}
                </div>
                <div style={{ fontSize: 22, fontWeight: 700 }}>{stats.counts[s]}</div>
              </div>
            ))}
          </div>
        </div>

        {/* 진행률 */}
        <div style={{ ...card, display: "flex", flexDirection: "column" }}>
          <div style={cardTitle}>전체 진행률</div>
          <div style={{ fontSize: 30, fontWeight: 700 }} data-testid="progress-pct">
            {pct}%
          </div>
          <div style={{ flex: 1 }} />
          <div style={{ height: 8, borderRadius: 6, background: "var(--hover)", overflow: "hidden", marginTop: 12 }}>
            <div style={{ width: `${pct}%`, height: "100%", background: "var(--accent)" }} />
          </div>
        </div>

        {/* 마감 임박 */}
        <div style={{ ...card, gridColumn: "span 3" }}>
          <div style={cardTitle}>마감 임박</div>
          {stats.upcoming.length === 0 ? (
            <div style={{ color: "var(--text3)", fontSize: 13 }}>예정된 마감이 없습니다.</div>
          ) : (
            stats.upcoming.slice(0, 5).map((t) => (
              <div
                key={t.id}
                style={{ display: "flex", alignItems: "center", gap: 8, padding: "7px 0", borderTop: "1px solid var(--border)" }}
              >
                <span style={{ flex: 1, fontSize: 13 }}>{t.title}</span>
                <span style={{ fontSize: 12, fontWeight: 600, color: "var(--text2)" }}>{t.dueDate}</span>
              </div>
            ))
          )}
        </div>
      </div>
    </div>
  );
}

const card: CSSProperties = {
  background: "var(--card)",
  border: "1px solid var(--border)",
  borderRadius: 6,
  padding: 16,
};
const cardTitle: CSSProperties = { fontSize: 13, fontWeight: 600, marginBottom: 14 };
const countTile: CSSProperties = { borderRadius: 6, background: "var(--hover)", padding: 12 };
