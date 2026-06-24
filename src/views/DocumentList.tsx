import { useRef, useState, type CSSProperties } from "react";
import type { Document, Folder } from "../domain/types";
import { folderOptions } from "../domain/folderOptions";
import { Icon } from "../ui/icons/Icon";

export interface DocumentListProps {
  documents: Document[];
  error: string | null;
  /** 이동 드롭다운에 쓸 폴더 목록(이 사업·문서). 없으면 폴더 열 미노출. */
  folders?: Folder[];
  /** 현재 보고 있는 폴더 id (표시용) */
  currentFolderId?: string | null;
  /** 사용자가 입력한 제목으로 새 문서 생성. */
  onCreate: (title: string) => void;
  onOpen: (id: string) => void;
  onRename: (id: string, title: string) => void;
  /** 폴더 이동 (folderId=null 이면 미분류). 제공 시 폴더 열을 노출한다. */
  onMove?: (id: string, folderId: string | null) => void;
  onArchive: (id: string) => void;
}

export function DocumentList(props: DocumentListProps) {
  const { documents, error, folders = [], onCreate, onOpen, onRename, onMove, onArchive } = props;
  const showFolders = !!onMove;
  const opts = folderOptions(folders);
  const cols = showFolders ? "1fr 150px 100px 92px" : "1fr 100px 92px";
  const grid = { ...rowGrid, gridTemplateColumns: cols };
  const [editingId, setEditingId] = useState<string | null>(null);
  const [draft, setDraft] = useState("");
  const renameDone = useRef(false); // Enter→blur 중복 커밋 방지
  // 새 문서 인라인 이름 입력
  const [creating, setCreating] = useState(false);
  const [newName, setNewName] = useState("");
  const finishedRef = useRef(false); // Enter/blur 중복 커밋 방지

  const startCreate = () => {
    finishedRef.current = false;
    setNewName("");
    setCreating(true);
  };
  const finishCreate = (save: boolean) => {
    if (finishedRef.current) return;
    finishedRef.current = true;
    setCreating(false);
    const name = newName.trim();
    if (save && name) onCreate(name);
    setNewName("");
  };

  const startEdit = (d: Document) => {
    renameDone.current = false;
    setEditingId(d.id);
    setDraft(d.title);
  };
  const commit = (d: Document) => {
    if (renameDone.current) return;
    renameDone.current = true;
    const name = draft.trim();
    if (name && name !== d.title) onRename(d.id, name);
    setEditingId(null);
  };

  return (
    <div style={{ display: "flex", flexDirection: "column", height: "100%", minHeight: 0 }}>
      <div style={{ display: "flex", alignItems: "center", padding: "12px 20px" }}>
        <span style={{ fontSize: 15, fontWeight: 600, flex: 1 }}>문서</span>
        <button onClick={startCreate} style={createBtn}>
          <Icon name="plus" size={14} /> 새 문서
        </button>
      </div>

      {error && (
        <div style={errorBar}>
          <Icon name="alert" size={14} />
          <span>{error}</span>
        </div>
      )}

      <div style={{ ...grid, ...headerStyle }}>
        <span>제목</span>
        {showFolders && <span>폴더</span>}
        <span>생성일</span>
        <span style={{ textAlign: "right" }}>동작</span>
      </div>

      {creating && (
        <div style={{ ...grid, ...bodyRow }}>
          <span style={{ display: "flex", alignItems: "center", gap: 6, minWidth: 0 }}>
            <Icon name="document" size={15} style={{ color: "var(--text2)", flexShrink: 0 }} />
            <input
              autoFocus
              aria-label="새 문서 이름"
              value={newName}
              placeholder="문서 이름 (Enter로 생성)"
              onChange={(e) => setNewName(e.target.value)}
              onKeyDown={(e) => {
                if (e.key === "Enter") finishCreate(true);
                else if (e.key === "Escape") finishCreate(false);
              }}
              onBlur={() => finishCreate(true)}
              style={renameInput}
            />
          </span>
          {showFolders && <span />}
          <span />
          <span />
        </div>
      )}

      {documents.length === 0 && !creating ? (
        <div style={{ padding: "20px", color: "var(--text3)", fontSize: 13 }}>
          문서가 없습니다. “새 문서”로 추가하세요.
        </div>
      ) : (
        <div style={{ flex: 1, overflow: "auto", minHeight: 0 }}>
          {documents.map((d) => (
            <div key={d.id} data-testid={`doc-${d.id}`} style={{ ...grid, ...bodyRow }}>
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
                    title="클릭하여 편집 · 더블클릭하여 이름 변경"
                    onClick={() => onOpen(d.id)}
                    onDoubleClick={() => startEdit(d)}
                    style={{
                      fontSize: 13.5,
                      overflow: "hidden",
                      textOverflow: "ellipsis",
                      whiteSpace: "nowrap",
                      cursor: "pointer",
                    }}
                  >
                    {d.title}
                  </span>
                )}
              </span>

              {showFolders && (
                <span onClick={(e) => e.stopPropagation()}>
                  <select
                    aria-label={`${d.title} 폴더`}
                    value={d.folderId ?? ""}
                    onChange={(e) => onMove!(d.id, e.target.value === "" ? null : e.target.value)}
                    style={folderSelect}
                  >
                    <option value="">미분류</option>
                    {opts.map((o) => (
                      <option key={o.id} value={o.id}>
                        {o.depth > 0 ? " " : ""}{o.label}
                      </option>
                    ))}
                  </select>
                </span>
              )}

              <span style={{ fontSize: 12, color: "var(--text3)" }}>{d.createdAt.slice(0, 10)}</span>

              <span style={{ display: "flex", gap: 4, justifyContent: "flex-end" }}>
                <button
                  onClick={() => onOpen(d.id)}
                  style={iconAction}
                  aria-label={`${d.title} 편집`}
                  title="편집"
                >
                  편집
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
  gridTemplateColumns: "1fr 100px 92px",
  alignItems: "center",
  gap: 8,
  padding: "0 20px",
};
const folderSelect: CSSProperties = {
  width: "100%",
  fontSize: 11.5,
  border: "1px solid var(--border)",
  borderRadius: 4,
  padding: "2px 4px",
  background: "var(--bg)",
  color: "var(--text2)",
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
const createBtn: CSSProperties = {
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
