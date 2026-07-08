import { useEffect, useMemo, useRef, useState, type CSSProperties } from "react";
import DOMPurify from "dompurify";
import { marked } from "marked";
import type { Document } from "../domain/types";
import { api } from "../api/client";
import { Icon } from "../ui/icons/Icon";

type SaveState = "saved" | "saving";

export interface DocEditorProps {
  document: Document;
}

/** Markdown 원문을 저장하고, 오른쪽 패널에서 즉시 렌더링한다. */
export function DocEditor({ document }: DocEditorProps) {
  const [markdown, setMarkdown] = useState(document.body ?? "");
  const [loaded, setLoaded] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [saveState, setSaveState] = useState<SaveState>("saved");
  const timer = useRef<ReturnType<typeof setTimeout> | null>(null);
  const latest = useRef(document.body ?? "");

  useEffect(() => {
    let alive = true;
    setLoaded(false);
    setError(null);
    setSaveState("saved");
    latest.current = document.body ?? "";
    setMarkdown(document.body ?? "");
    api.document
      .get(document.id)
      .then((d) => {
        if (!alive) return;
        const body = d.body ?? "";
        latest.current = body;
        setMarkdown(body);
        setLoaded(true);
      })
      .catch((e) => {
        if (!alive) return;
        setError(String(e));
        setLoaded(true);
      });
    return () => {
      alive = false;
    };
  }, [document.id, document.body]);

  const rendered = useMemo(() => {
    const html = marked.parse(markdown, {
      async: false,
      breaks: true,
      gfm: true,
    }) as string;
    return DOMPurify.sanitize(html);
  }, [markdown]);

  const save = (body: string) => {
    setSaveState("saving");
    api.document
      .setBody(document.id, body)
      .then(() => setSaveState("saved"))
      .catch((e) => {
        setError(String(e));
        setSaveState("saved");
      });
  };

  const flush = () => {
    if (timer.current) {
      clearTimeout(timer.current);
      timer.current = null;
      save(latest.current);
    }
  };

  useEffect(() => {
    return () => {
      if (timer.current) flush();
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [document.id]);

  const onBodyChange = (body: string) => {
    latest.current = body;
    setMarkdown(body);
    setSaveState("saving");
    if (timer.current) clearTimeout(timer.current);
    timer.current = setTimeout(() => {
      timer.current = null;
      save(body);
    }, 600);
  };

  const openExportFolder = async () => {
    flush();
    try {
      await api.document.showExportFolder(document.id);
    } catch (e) {
      setError(String(e));
    }
  };

  return (
    <div style={{ display: "flex", flexDirection: "column", height: "100%", minHeight: 0 }}>
      <div style={topbar}>
        <h1 style={titleStyle}>{document.title}</h1>
        <button
          onClick={() => void openExportFolder()}
          style={folderBtn}
          aria-label={`${document.title} 폴더 열기`}
          title="Markdown export 폴더 열기"
        >
          <Icon name="folder" size={14} />
          폴더
        </button>
        <span style={{ fontSize: 11, color: "var(--text3)", whiteSpace: "nowrap" }}>
          {saveState === "saving" ? "저장 중..." : "저장됨"}
        </span>
      </div>

      {error && (
        <div style={errorBar}>
          <Icon name="alert" size={14} />
          <span>{error}</span>
        </div>
      )}

      {loaded ? (
        <div style={editorGrid}>
          <section style={pane}>
            <div style={paneHeader}>Markdown</div>
            <textarea
              aria-label="Markdown 문서 본문"
              value={markdown}
              onChange={(e) => onBodyChange(e.currentTarget.value)}
              onBlur={flush}
              spellCheck={false}
              style={textareaStyle}
            />
          </section>
          <section style={pane}>
            <div style={paneHeader}>렌더링</div>
            <div
              className="md-preview"
              aria-label="Markdown 렌더링 결과"
              style={previewStyle}
              dangerouslySetInnerHTML={{ __html: rendered }}
            />
          </section>
        </div>
      ) : (
        <div style={loadingBox}>문서를 여는 중...</div>
      )}
    </div>
  );
}

const topbar: CSSProperties = {
  flexShrink: 0,
  display: "flex",
  alignItems: "center",
  gap: 10,
  padding: "14px 24px 12px",
  borderBottom: "1px solid var(--border)",
};
const titleStyle: CSSProperties = {
  margin: 0,
  fontSize: 22,
  fontWeight: 700,
  flex: 1,
  overflow: "hidden",
  textOverflow: "ellipsis",
  whiteSpace: "nowrap",
};
const folderBtn: CSSProperties = {
  display: "inline-flex",
  alignItems: "center",
  gap: 5,
  height: 28,
  padding: "0 9px",
  border: "1px solid var(--border)",
  borderRadius: "var(--radius-sm)",
  background: "var(--bg)",
  color: "var(--text2)",
  fontSize: 12,
  cursor: "pointer",
};
const editorGrid: CSSProperties = {
  flex: 1,
  minHeight: 0,
  display: "grid",
  gridTemplateColumns: "minmax(320px, 1fr) minmax(320px, 1fr)",
  borderTop: "1px solid var(--border)",
};
const pane: CSSProperties = {
  minWidth: 0,
  minHeight: 0,
  display: "flex",
  flexDirection: "column",
  borderRight: "1px solid var(--border)",
};
const paneHeader: CSSProperties = {
  flexShrink: 0,
  height: 32,
  display: "flex",
  alignItems: "center",
  padding: "0 14px",
  borderBottom: "1px solid var(--border)",
  color: "var(--text2)",
  fontSize: 12,
  fontWeight: 700,
};
const textareaStyle: CSSProperties = {
  flex: 1,
  minHeight: 0,
  resize: "none",
  border: "none",
  outline: "none",
  padding: "18px 20px 80px",
  background: "var(--bg)",
  color: "var(--text)",
  fontFamily: "var(--font-mono)",
  fontSize: 14,
  lineHeight: 1.65,
};
const previewStyle: CSSProperties = {
  flex: 1,
  minHeight: 0,
  overflow: "auto",
  padding: "18px 24px 80px",
};
const loadingBox: CSSProperties = {
  padding: "20px 24px",
  color: "var(--text3)",
  fontSize: 14,
};
const errorBar: CSSProperties = {
  display: "flex",
  alignItems: "center",
  gap: 6,
  margin: "10px 24px 0",
  padding: "6px 10px",
  borderRadius: "var(--radius-md)",
  background: "rgba(239,68,68,.1)",
  color: "#ef4444",
  fontSize: 12,
};
