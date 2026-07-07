import { useEffect, useMemo, useRef, useState, type CSSProperties } from "react";
import type { Document } from "../domain/types";
import type { DocumentEditorBodyInput } from "../api/client";
import { api } from "../api/client";
import { Icon } from "../ui/icons/Icon";
import { BlockDocumentEditor } from "./document-editor/BlockDocumentEditor";
import { buildInitialEditorSource, prepareEditorSavePayload } from "./document-editor/documentBody";

type SaveState = "saved" | "saving";

export interface DocEditorProps {
  document: Document;
}

/** 문서를 BlockNote 기반 라이브 블록 에디터로 편집하고 Markdown body를 외부 공유용으로 함께 저장한다. */
export function DocEditor({ document }: DocEditorProps) {
  const [loaded, setLoaded] = useState<Document | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [saveState, setSaveState] = useState<SaveState>("saved");
  const timer = useRef<ReturnType<typeof setTimeout> | null>(null);
  const latest = useRef<DocumentEditorBodyInput | null>(null);
  const touched = useRef(false);

  useEffect(() => {
    let alive = true;
    setLoaded(null);
    setError(null);
    touched.current = false;
    latest.current = null;
    api.document
      .get(document.id)
      .then((d) => {
        if (alive && !touched.current) {
          setLoaded(d);
        }
      })
      .catch((e) => setError(String(e)));
    return () => {
      alive = false;
    };
  }, [document.id]);

  const source = useMemo(() => (loaded ? buildInitialEditorSource(loaded) : null), [loaded]);
  const displayError = error ?? source?.warning ?? null;

  const save = (payload: DocumentEditorBodyInput) => {
    api.document
      .setEditorBody(document.id, payload)
      .then(() => setSaveState("saved"))
      .catch((e) => setError(String(e)));
  };

  const flush = () => {
    if (timer.current && latest.current) {
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

  const onChange = (payload: DocumentEditorBodyInput) => {
    touched.current = true;
    latest.current = payload;
    setSaveState("saving");
    if (timer.current) clearTimeout(timer.current);
    timer.current = setTimeout(() => {
      timer.current = null;
      save(payload);
    }, 600);
  };

  return (
    <div style={{ display: "flex", flexDirection: "column", height: "100%", minHeight: 0 }}>
      <div style={topbar}>
        <h1 style={{ margin: 0, fontSize: 22, fontWeight: 700, flex: 1, overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
          {document.title}
        </h1>
        <span style={{ fontSize: 11, color: "var(--text3)", whiteSpace: "nowrap" }}>
          {saveState === "saving" ? "저장 중..." : "저장됨"}
        </span>
      </div>

      {displayError && (
        <div style={errorBar}>
          <Icon name="alert" size={14} />
          <span>{displayError}</span>
        </div>
      )}

      <div style={{ flex: 1, minHeight: 0 }}>
        {loaded && source ? (
          <BlockDocumentEditor
            key={`${loaded.id}:${source.kind}:${source.markdown}`}
            documentId={loaded.id}
            initialBlocks={source.kind === "blocks" ? source.blocks : null}
            initialMarkdown={source.markdown}
            collaborationState={loaded.collaborationState}
            onChange={(value) => onChange(prepareEditorSavePayload(value))}
            onBlur={flush}
            onWarning={setError}
          />
        ) : (
          <div style={loadingBox}>문서를 여는 중...</div>
        )}
      </div>
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
