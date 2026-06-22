import type { CSSProperties } from "react";
import type { Deliverable, DeliverableStatus, DeliverableVersion } from "../domain/types";
import { DELIVERABLE_STATUS_COLOR, DELIVERABLE_STATUS_LABEL } from "../ui/colors";
import { Icon } from "../ui/icons/Icon";

export interface DeliverableListProps {
  deliverables: Deliverable[];
  selectedId: number | null;
  versions: DeliverableVersion[];
  onSelect: (id: number) => void;
  onCreate: () => void;
  onSetStatus: (id: number, status: DeliverableStatus) => void;
  onAddVersion: (id: number) => void;
  onArchive: (id: number) => void;
}

const STATUSES: DeliverableStatus[] = ["draft", "review", "done"];

export function DeliverableList(props: DeliverableListProps) {
  const { deliverables, selectedId, versions, onSelect, onCreate, onSetStatus, onAddVersion, onArchive } = props;
  const selected = deliverables.find((d) => d.id === selectedId) ?? null;

  return (
    <div style={{ display: "flex", height: "100%", minHeight: 0 }}>
      <div style={{ flex: 1, minWidth: 0, overflow: "auto" }}>
        <div style={{ display: "flex", alignItems: "center", padding: "12px 20px" }}>
          <span style={{ fontSize: 15, fontWeight: 600, flex: 1 }}>산출물</span>
          <button onClick={onCreate} style={addBtn}><Icon name="plus" size={14} /> 산출물</button>
        </div>
        <div style={{ ...rowGrid, ...headerStyle }}>
          <span>이름</span>
          <span>종류</span>
          <span>상태</span>
          <span>버전</span>
        </div>
        {deliverables.length === 0 ? (
          <div style={{ padding: "16px 20px", color: "var(--text3)", fontSize: 13 }}>
            산출물이 없습니다. “+ 산출물”로 추가하세요.
          </div>
        ) : (
          deliverables.map((d) => (
            <div
              key={d.id}
              data-testid={`deliv-${d.id}`}
              onClick={() => onSelect(d.id)}
              style={{
                ...rowGrid,
                ...bodyRow,
                background: d.id === selectedId ? "var(--sel)" : "transparent",
                boxShadow: d.id === selectedId ? "inset 2px 0 0 var(--accent)" : "none",
              }}
            >
              <span style={{ display: "flex", alignItems: "center", gap: 6, fontSize: 13.5, minWidth: 0 }}>
                <Icon
                  name={d.kind === "document" ? "document" : "deliverable"}
                  size={15}
                  style={{ color: "var(--text2)" }}
                />
                <span style={{ overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>{d.title}</span>
              </span>
              <span style={{ fontSize: 12, color: "var(--text2)" }}>
                {d.kind === "document" ? "문서" : "파일"}
              </span>
              <span onClick={(e) => e.stopPropagation()}>
                <select
                  aria-label={`${d.title} 상태`}
                  value={d.status}
                  onChange={(e) => onSetStatus(d.id, e.target.value as DeliverableStatus)}
                  style={{
                    fontSize: 11,
                    fontWeight: 600,
                    border: "none",
                    borderRadius: 4,
                    padding: "2px 4px",
                    background: DELIVERABLE_STATUS_COLOR[d.status] + "22",
                    color: DELIVERABLE_STATUS_COLOR[d.status],
                  }}
                >
                  {STATUSES.map((s) => (
                    <option key={s} value={s}>
                      {DELIVERABLE_STATUS_LABEL[s]}
                    </option>
                  ))}
                </select>
              </span>
              <span style={{ fontSize: 12, fontWeight: 600, fontFamily: "var(--font-mono)" }}>
                v{d.currentVersion}
              </span>
            </div>
          ))
        )}
      </div>

      <div style={detailPanel}>
        {!selected ? (
          <div style={{ padding: 18, color: "var(--text3)", fontSize: 13 }}>
            산출물을 선택하면 버전 기록이 표시됩니다.
          </div>
        ) : (
          <>
            <div style={{ padding: "16px 18px", borderBottom: "1px solid var(--border)" }}>
              <div style={{ fontSize: 15, fontWeight: 600, marginBottom: 10 }}>{selected.title}</div>
              <div style={{ display: "flex", gap: 8 }}>
                <button onClick={() => onAddVersion(selected.id)} style={primaryBtn}>새 버전</button>
                <button onClick={() => onArchive(selected.id)} style={dangerBtn}>보관</button>
              </div>
            </div>
            <div style={{ padding: "14px 18px" }}>
              <div style={{ fontSize: 12, fontWeight: 600, color: "var(--text2)", marginBottom: 12 }}>
                버전 히스토리
              </div>
              {versions.map((v) => (
                <div key={v.id} style={{ display: "flex", gap: 10, paddingBottom: 12 }} data-testid={`ver-${v.version}`}>
                  <span style={{ fontSize: 13, fontWeight: 600, fontFamily: "var(--font-mono)", width: 32 }}>
                    v{v.version}
                  </span>
                  <span style={{ fontSize: 12.5, color: "var(--text2)", flex: 1 }}>{v.note ?? "-"}</span>
                </div>
              ))}
            </div>
          </>
        )}
      </div>
    </div>
  );
}

const rowGrid: CSSProperties = {
  display: "grid",
  gridTemplateColumns: "1fr 70px 90px 60px",
  alignItems: "center",
  padding: "0 20px",
};
const headerStyle: CSSProperties = {
  height: 32,
  borderBottom: "1px solid var(--border)",
  fontSize: 12,
  fontWeight: 600,
  color: "var(--text2)",
};
const bodyRow: CSSProperties = { height: 40, borderBottom: "1px solid var(--border)", cursor: "pointer" };
const detailPanel: CSSProperties = {
  width: 300,
  flex: "none",
  borderLeft: "1px solid var(--border)",
  background: "var(--sidebar)",
  overflow: "auto",
};
const addBtn: CSSProperties = {
  display: "inline-flex",
  alignItems: "center",
  gap: 4,
  border: "1px solid var(--border)",
  background: "var(--bg)",
  color: "var(--accent)",
  borderRadius: "var(--radius-md)",
  padding: "5px 12px",
  fontSize: 12,
  fontWeight: 600,
  cursor: "pointer",
};
const primaryBtn: CSSProperties = {
  flex: 1,
  border: "none",
  background: "var(--accent)",
  color: "#fff",
  borderRadius: "var(--radius-md)",
  padding: "7px 0",
  fontSize: 12,
  fontWeight: 600,
  cursor: "pointer",
};
const dangerBtn: CSSProperties = {
  border: "1px solid var(--border)",
  background: "var(--bg)",
  color: "var(--st-danger)",
  borderRadius: "var(--radius-md)",
  padding: "7px 14px",
  fontSize: 12,
  fontWeight: 600,
  cursor: "pointer",
};
