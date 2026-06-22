import { useRef, useState, type CSSProperties } from "react";
import type { Deliverable, DeliverableStatus } from "../domain/types";
import { DELIVERABLE_STATUS_COLOR, DELIVERABLE_STATUS_LABEL } from "../ui/colors";
import { formatBytes } from "../domain/format";
import { Icon } from "../ui/icons/Icon";

export interface DeliverableListProps {
  deliverables: Deliverable[];
  error: string | null;
  uploading?: boolean;
  onUpload: () => void;
  onSetStatus: (id: number, status: DeliverableStatus) => void;
  onRename: (id: number, title: string) => void;
  onOpen: (d: Deliverable) => void;
  onArchive: (id: number) => void;
}

const STATUSES: DeliverableStatus[] = ["draft", "review", "done"];

export function DeliverableList(props: DeliverableListProps) {
  const { deliverables, error, uploading, onUpload, onSetStatus, onRename, onOpen, onArchive } = props;
  const [editingId, setEditingId] = useState<number | null>(null);
  const [draft, setDraft] = useState("");
  const renameDone = useRef(false); // Enter→blur 중복 커밋 방지

  const startEdit = (d: Deliverable) => {
    renameDone.current = false;
    setEditingId(d.id);
    setDraft(d.title);
  };
  const commit = (d: Deliverable) => {
    if (renameDone.current) return;
    renameDone.current = true;
    const name = draft.trim();
    if (name && name !== d.title) onRename(d.id, name);
    setEditingId(null);
  };

  return (
    <div style={{ display: "flex", flexDirection: "column", height: "100%", minHeight: 0 }}>
      <div style={{ display: "flex", alignItems: "center", padding: "12px 20px" }}>
        <span style={{ fontSize: 15, fontWeight: 600, flex: 1 }}>산출물</span>
        <button onClick={onUpload} disabled={uploading} style={{ ...uploadBtn, opacity: uploading ? 0.6 : 1 }}>
          <Icon name="arrow-up" size={14} /> {uploading ? "업로드 중…" : "파일 업로드"}
        </button>
      </div>

      {error && (
        <div style={errorBar}>
          <Icon name="alert" size={14} />
          <span>{error}</span>
        </div>
      )}

      <div style={{ ...rowGrid, ...headerStyle }}>
        <span>상태</span>
        <span>파일명</span>
        <span>크기</span>
        <span>업로드일</span>
        <span style={{ textAlign: "right" }}>동작</span>
      </div>

      {deliverables.length === 0 ? (
        <div style={{ padding: "20px", color: "var(--text3)", fontSize: 13 }}>
          업로드된 산출물이 없습니다. “파일 업로드”로 추가하세요.
        </div>
      ) : (
        <div style={{ flex: 1, overflow: "auto", minHeight: 0 }}>
          {deliverables.map((d) => (
            <div key={d.id} data-testid={`deliv-${d.id}`} style={{ ...rowGrid, ...bodyRow }}>
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

              <span style={{ display: "flex", alignItems: "center", gap: 6, minWidth: 0 }}>
                <Icon name="document" size={15} style={{ color: "var(--text2)", flexShrink: 0 }} />
                {editingId === d.id ? (
                  <input
                    autoFocus
                    aria-label="이름 변경"
                    value={draft}
                    onChange={(e) => setDraft(e.target.value)}
                    onBlur={() => commit(d)}
                    onKeyDown={(e) => {
                      if (e.key === "Enter") commit(d);
                      else if (e.key === "Escape") setEditingId(null);
                    }}
                    style={renameInput}
                  />
                ) : (
                  <span
                    title="더블클릭하여 이름 변경"
                    onDoubleClick={() => startEdit(d)}
                    style={{ fontSize: 13.5, overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}
                  >
                    {d.title}
                  </span>
                )}
              </span>

              <span style={{ fontSize: 12, color: "var(--text2)", fontFamily: "var(--font-mono)" }}>
                {formatBytes(d.fileSize)}
              </span>
              <span style={{ fontSize: 12, color: "var(--text3)" }}>{d.createdAt.slice(0, 10)}</span>

              <span style={{ display: "flex", gap: 4, justifyContent: "flex-end" }}>
                <button
                  onClick={() => onOpen(d)}
                  disabled={!d.filePath}
                  style={{ ...iconAction, opacity: d.filePath ? 1 : 0.4 }}
                  aria-label={`${d.title} 열기`}
                  title="열기"
                >
                  열기
                </button>
                <button
                  onClick={() => onArchive(d.id)}
                  style={iconAction}
                  aria-label={`${d.title} 삭제`}
                  title="삭제(휴지통)"
                >
                  <Icon name="trash" size={14} />
                </button>
              </span>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}

const rowGrid: CSSProperties = {
  display: "grid",
  gridTemplateColumns: "78px 1fr 90px 100px 92px",
  alignItems: "center",
  gap: 8,
  padding: "0 20px",
};
const headerStyle: CSSProperties = {
  height: 32,
  flexShrink: 0,
  borderBottom: "1px solid var(--border)",
  fontSize: 12,
  fontWeight: 600,
  color: "var(--text2)",
};
const bodyRow: CSSProperties = { height: 44, borderBottom: "1px solid var(--border)" };
const uploadBtn: CSSProperties = {
  display: "inline-flex",
  alignItems: "center",
  gap: 5,
  border: "1px solid var(--border)",
  background: "var(--bg)",
  color: "var(--accent)",
  borderRadius: "var(--radius-md)",
  padding: "6px 12px",
  fontSize: 12,
  fontWeight: 600,
  cursor: "pointer",
};
const renameInput: CSSProperties = {
  flex: 1,
  minWidth: 0,
  border: "1px solid var(--accent)",
  borderRadius: 4,
  background: "var(--bg)",
  color: "var(--text)",
  fontSize: 13.5,
  padding: "1px 4px",
  fontFamily: "inherit",
};
const iconAction: CSSProperties = {
  display: "inline-flex",
  alignItems: "center",
  justifyContent: "center",
  gap: 4,
  minWidth: 28,
  height: 26,
  padding: "0 8px",
  border: "1px solid var(--border)",
  background: "var(--bg)",
  color: "var(--text2)",
  borderRadius: "var(--radius-sm)",
  fontSize: 12,
  cursor: "pointer",
};
const errorBar: CSSProperties = {
  display: "flex",
  alignItems: "center",
  gap: 6,
  margin: "0 20px 8px",
  padding: "6px 10px",
  borderRadius: "var(--radius-md)",
  background: "rgba(239,68,68,.1)",
  color: "#ef4444",
  fontSize: 12,
};
