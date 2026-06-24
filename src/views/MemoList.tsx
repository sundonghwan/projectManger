import { useRef, useState, type CSSProperties } from "react";
import type { Memo, MemoColor } from "../domain/types";
import { partitionMemos } from "../domain/memoSort";
import { MEMO_COLORS, memoBg } from "../ui/colors";
import { Icon } from "../ui/icons/Icon";

export interface MemoListProps {
  memos: Memo[];
  error: string | null;
  onCreate: (title: string, body: string) => void;
  onUpdate: (id: string, title: string, body: string) => void;
  onSetColor: (id: string, color: MemoColor | null) => void;
  onSetPinned: (id: string, pinned: boolean) => void;
  onArchive: (id: string) => void;
}

export function MemoList(props: MemoListProps) {
  const { memos, error, onCreate, onUpdate, onSetColor, onSetPinned, onArchive } = props;
  const { pinned, others } = partitionMemos(memos);
  const [editing, setEditing] = useState<Memo | null>(null);

  return (
    <div style={{ display: "flex", flexDirection: "column", height: "100%", minHeight: 0 }}>
      <div style={{ padding: "14px 20px 6px" }}>
        <span style={{ fontSize: 15, fontWeight: 600 }}>메모</span>
      </div>

      <div style={{ padding: "0 20px 10px" }}>
        <Composer onCreate={onCreate} />
      </div>

      {error && (
        <div style={errorBar}>
          <Icon name="alert" size={14} />
          <span>{error}</span>
        </div>
      )}

      <div style={{ flex: 1, overflow: "auto", minHeight: 0, padding: "0 16px 20px" }}>
        {memos.length === 0 ? (
          <div style={{ padding: "20px", color: "var(--text3)", fontSize: 13 }}>
            메모가 없습니다. 위에서 새 메모를 작성하세요.
          </div>
        ) : (
          <>
            {pinned.length > 0 && (
              <>
                <Section label="고정됨" />
                <Grid>
                  {pinned.map((m) => (
                    <Card key={m.id} memo={m} onOpen={() => setEditing(m)} onSetColor={onSetColor} onSetPinned={onSetPinned} onArchive={onArchive} />
                  ))}
                </Grid>
              </>
            )}
            {others.length > 0 && (
              <>
                {pinned.length > 0 && <Section label="기타" />}
                <Grid>
                  {others.map((m) => (
                    <Card key={m.id} memo={m} onOpen={() => setEditing(m)} onSetColor={onSetColor} onSetPinned={onSetPinned} onArchive={onArchive} />
                  ))}
                </Grid>
              </>
            )}
          </>
        )}
      </div>

      {editing && (
        <Editor
          memo={editing}
          onClose={(title, body) => {
            if (title !== editing.title || body !== editing.body) onUpdate(editing.id, title, body);
            setEditing(null);
          }}
          onSetColor={(c) => onSetColor(editing.id, c)}
          onSetPinned={(p) => onSetPinned(editing.id, p)}
          onArchive={() => {
            onArchive(editing.id);
            setEditing(null);
          }}
        />
      )}
    </div>
  );
}

function Section({ label }: { label: string }) {
  return (
    <div style={{ fontSize: 11, fontWeight: 700, letterSpacing: 0.4, color: "var(--text3)", textTransform: "uppercase", margin: "6px 4px 8px" }}>
      {label}
    </div>
  );
}

function Grid({ children }: { children: React.ReactNode }) {
  return <div style={{ columnWidth: 220, columnGap: 12 }}>{children}</div>;
}

function Composer({ onCreate }: { onCreate: (title: string, body: string) => void }) {
  const [open, setOpen] = useState(false);
  const [title, setTitle] = useState("");
  const [body, setBody] = useState("");
  const finished = useRef(false);

  const finish = () => {
    if (finished.current) return;
    finished.current = true;
    if (title.trim() || body.trim()) onCreate(title.trim(), body.trim());
    setTitle("");
    setBody("");
    setOpen(false);
    setTimeout(() => (finished.current = false), 0);
  };

  if (!open) {
    return (
      <button
        onClick={() => setOpen(true)}
        style={{ ...composerBox, color: "var(--text3)", textAlign: "left", cursor: "text" }}
        aria-label="새 메모 작성"
      >
        메모 작성…
      </button>
    );
  }
  return (
    <div style={composerBox} onBlur={(e) => { if (!e.currentTarget.contains(e.relatedTarget)) finish(); }}>
      <input
        autoFocus
        aria-label="메모 제목"
        placeholder="제목"
        value={title}
        onChange={(e) => setTitle(e.target.value)}
        style={{ ...plainInput, fontWeight: 600 }}
      />
      <textarea
        aria-label="메모 내용"
        placeholder="메모 작성…"
        value={body}
        onChange={(e) => setBody(e.target.value)}
        onKeyDown={(e) => {
          if (e.key === "Escape") { setTitle(""); setBody(""); setOpen(false); }
        }}
        rows={3}
        style={{ ...plainInput, resize: "vertical", fontFamily: "inherit" }}
      />
      <div style={{ display: "flex", justifyContent: "flex-end" }}>
        <button onClick={finish} style={addBtn}>완료</button>
      </div>
    </div>
  );
}

function Card({
  memo,
  onOpen,
  onSetColor,
  onSetPinned,
  onArchive,
}: {
  memo: Memo;
  onOpen: () => void;
  onSetColor: (id: string, color: MemoColor | null) => void;
  onSetPinned: (id: string, pinned: boolean) => void;
  onArchive: (id: string) => void;
}) {
  const [palette, setPalette] = useState(false);
  return (
    <div
      data-testid={`memo-${memo.id}`}
      style={{ ...card, background: memoBg(memo.color) }}
      onClick={onOpen}
    >
      <button
        aria-label={memo.pinned ? `${memo.title || "메모"} 고정 해제` : `${memo.title || "메모"} 고정`}
        onClick={(e) => { e.stopPropagation(); onSetPinned(memo.id, !memo.pinned); }}
        style={{ ...pinBtn, color: memo.pinned ? "var(--accent)" : "var(--text3)" }}
      >
        <Icon name="pin" size={14} />
      </button>

      {memo.title && <div style={cardTitle}>{memo.title}</div>}
      {memo.body && <div style={cardBody}>{memo.body}</div>}
      {!memo.title && !memo.body && <div style={{ ...cardBody, color: "var(--text3)" }}>(빈 메모)</div>}

      <div style={cardActions} onClick={(e) => e.stopPropagation()}>
        <div style={{ position: "relative" }}>
          <button aria-label={`${memo.title || "메모"} 색상`} onClick={() => setPalette((o) => !o)} style={actionBtn}>
            <span style={{ width: 14, height: 14, borderRadius: "50%", border: "1px solid var(--border-strong)", background: memoBg(memo.color), display: "block" }} />
          </button>
          {palette && (
            <>
              <div style={{ position: "fixed", inset: 0, zIndex: 40 }} onClick={() => setPalette(false)} />
              <div style={paletteBox}>
                {MEMO_COLORS.map((c) => (
                  <button
                    key={c}
                    aria-label={`색상 ${c}`}
                    onClick={() => { onSetColor(memo.id, c === "default" ? null : c); setPalette(false); }}
                    style={{
                      width: 22, height: 22, borderRadius: "50%", cursor: "pointer",
                      background: memoBg(c === "default" ? null : c),
                      border: `2px solid ${(memo.color ?? "default") === c ? "var(--accent)" : "var(--border-strong)"}`,
                    }}
                  />
                ))}
              </div>
            </>
          )}
        </div>
        <button aria-label={`${memo.title || "메모"} 삭제`} onClick={() => onArchive(memo.id)} style={actionBtn} title="보관(휴지통)">
          <Icon name="trash" size={14} />
        </button>
      </div>
    </div>
  );
}

function Editor({
  memo,
  onClose,
  onSetColor,
  onSetPinned,
  onArchive,
}: {
  memo: Memo;
  onClose: (title: string, body: string) => void;
  onSetColor: (c: MemoColor | null) => void;
  onSetPinned: (p: boolean) => void;
  onArchive: () => void;
}) {
  const [title, setTitle] = useState(memo.title);
  const [body, setBody] = useState(memo.body);
  return (
    <div style={overlay} onClick={() => onClose(title, body)}>
      <div role="dialog" aria-label="메모 편집" style={{ ...dialog, background: memoBg(memo.color) }} onClick={(e) => e.stopPropagation()}>
        <div style={{ display: "flex", alignItems: "center", gap: 6 }}>
          <input
            aria-label="메모 제목"
            placeholder="제목"
            value={title}
            onChange={(e) => setTitle(e.target.value)}
            style={{ ...plainInput, fontWeight: 600, flex: 1 }}
          />
          <button
            aria-label={memo.pinned ? "고정 해제" : "고정"}
            onClick={() => onSetPinned(!memo.pinned)}
            style={{ ...pinBtn, position: "static", color: memo.pinned ? "var(--accent)" : "var(--text3)" }}
          >
            <Icon name="pin" size={16} />
          </button>
        </div>
        <textarea
          autoFocus
          aria-label="메모 내용"
          placeholder="메모 작성…"
          value={body}
          onChange={(e) => setBody(e.target.value)}
          rows={8}
          style={{ ...plainInput, resize: "vertical", fontFamily: "inherit" }}
        />
        <div style={{ display: "flex", alignItems: "center", gap: 6, marginTop: 8 }}>
          {MEMO_COLORS.map((c) => (
            <button
              key={c}
              aria-label={`색상 ${c}`}
              onClick={() => onSetColor(c === "default" ? null : c)}
              style={{
                width: 20, height: 20, borderRadius: "50%", cursor: "pointer",
                background: memoBg(c === "default" ? null : c),
                border: `2px solid ${(memo.color ?? "default") === c ? "var(--accent)" : "var(--border-strong)"}`,
              }}
            />
          ))}
          <span style={{ flex: 1 }} />
          <button onClick={onArchive} style={dangerText} aria-label="메모 삭제">삭제</button>
          <button onClick={() => onClose(title, body)} style={addBtn}>닫기</button>
        </div>
      </div>
    </div>
  );
}

const composerBox: CSSProperties = {
  width: "100%",
  maxWidth: 560,
  display: "block",
  border: "1px solid var(--border)",
  background: "var(--card)",
  borderRadius: "var(--radius-md)",
  boxShadow: "var(--shadow-popover)",
  padding: 10,
  fontSize: 14,
};
const plainInput: CSSProperties = {
  width: "100%",
  border: "none",
  background: "transparent",
  color: "var(--text)",
  fontSize: 14,
  padding: "4px 2px",
  outline: "none",
};
const errorBar: CSSProperties = {
  display: "flex", alignItems: "center", gap: 6, margin: "0 20px 8px",
  padding: "6px 10px", borderRadius: "var(--radius-md)", background: "rgba(239,68,68,.1)", color: "#ef4444", fontSize: 12,
};
const card: CSSProperties = {
  position: "relative",
  breakInside: "avoid",
  display: "inline-block",
  width: "100%",
  marginBottom: 12,
  border: "1px solid var(--border)",
  borderRadius: "var(--radius-md)",
  padding: "12px 12px 8px",
  cursor: "pointer",
};
const cardTitle: CSSProperties = { fontSize: 14, fontWeight: 700, marginBottom: 4, paddingRight: 22, wordBreak: "break-word" };
const cardBody: CSSProperties = { fontSize: 13, color: "var(--text)", whiteSpace: "pre-wrap", wordBreak: "break-word", maxHeight: 220, overflow: "hidden" };
const cardActions: CSSProperties = { display: "flex", alignItems: "center", gap: 4, marginTop: 8 };
const pinBtn: CSSProperties = {
  position: "absolute", top: 8, right: 8, width: 24, height: 24,
  display: "flex", alignItems: "center", justifyContent: "center",
  border: "none", background: "transparent", cursor: "pointer", padding: 0,
};
const actionBtn: CSSProperties = {
  width: 26, height: 26, display: "flex", alignItems: "center", justifyContent: "center",
  border: "none", background: "transparent", color: "var(--text2)", borderRadius: "var(--radius-sm)", cursor: "pointer",
};
const paletteBox: CSSProperties = {
  position: "absolute", top: "calc(100% + 4px)", left: 0, zIndex: 50,
  display: "flex", gap: 6, flexWrap: "wrap", width: 180,
  background: "var(--card)", border: "1px solid var(--border)", borderRadius: "var(--radius-md)",
  boxShadow: "var(--shadow-popover)", padding: 8,
};
const addBtn: CSSProperties = {
  border: "none", background: "var(--accent)", color: "#fff", borderRadius: "var(--radius-md)",
  padding: "6px 14px", fontSize: 12, fontWeight: 600, cursor: "pointer",
};
const dangerText: CSSProperties = {
  border: "none", background: "transparent", color: "var(--st-danger)", fontSize: 12, fontWeight: 600, cursor: "pointer", padding: "6px 8px",
};
const overlay: CSSProperties = {
  position: "fixed", inset: 0, background: "rgba(0,0,0,.35)", display: "flex", alignItems: "center", justifyContent: "center", zIndex: 300,
};
const dialog: CSSProperties = {
  width: 520, maxWidth: "92%", border: "1px solid var(--border)", borderRadius: "var(--radius-lg)",
  boxShadow: "var(--shadow-modal)", padding: 16,
};
