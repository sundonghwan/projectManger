import { useEffect, useMemo, useRef, useState, type CSSProperties } from "react";
import { marked } from "marked";
import type { Document } from "../domain/types";
import { api } from "../api/client";
import { Icon } from "../ui/icons/Icon";

marked.setOptions({ gfm: true, breaks: true });

export interface DocEditorProps {
  document: Document;
}

/** 문서 본문을 마크다운 단일 텍스트로 편집(편집/미리보기 토글, 디바운스 자동 저장). */
export function DocEditor({ document }: DocEditorProps) {
  const [body, setBody] = useState(document.body ?? "");
  const [mode, setMode] = useState<"edit" | "preview">("edit");
  const [error, setError] = useState<string | null>(null);
  const timer = useRef<ReturnType<typeof setTimeout> | null>(null);
  const latest = useRef(body);
  const touched = useRef(false);

  // 마운트 시 최신 본문 로드(목록 캐시가 오래됐을 수 있음). 사용자가 이미 입력했으면 덮어쓰지 않음.
  useEffect(() => {
    let alive = true;
    api.document
      .get(document.id)
      .then((d) => {
        if (alive && !touched.current) {
          setBody(d.body);
          latest.current = d.body;
        }
      })
      .catch((e) => setError(String(e)));
    return () => {
      alive = false;
    };
  }, [document.id]);

  const flush = () => {
    if (timer.current) {
      clearTimeout(timer.current);
      timer.current = null;
    }
    api.document.setBody(document.id, latest.current).catch((e) => setError(String(e)));
  };

  // 언마운트(문서 전환/목록 복귀) 시 미저장분 저장
  useEffect(() => {
    return () => {
      if (timer.current) flush();
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [document.id]);

  const onChange = (text: string) => {
    touched.current = true;
    setBody(text);
    latest.current = text;
    if (timer.current) clearTimeout(timer.current);
    timer.current = setTimeout(() => {
      api.document.setBody(document.id, text).catch((e) => setError(String(e)));
      timer.current = null;
    }, 600);
  };

  const html = useMemo(() => marked.parse(body, { async: false }) as string, [body]);

  return (
    <div style={{ display: "flex", flexDirection: "column", height: "100%", minHeight: 0 }}>
      <div style={topbar}>
        <h1 style={{ margin: 0, fontSize: 22, fontWeight: 700, letterSpacing: "-0.4px", flex: 1, overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
          {document.title}
        </h1>
        <div style={{ display: "flex", alignItems: "center", gap: 4 }}>
          <button onClick={() => setMode("edit")} style={tabBtn(mode === "edit")}>편집</button>
          <button onClick={() => setMode("preview")} style={tabBtn(mode === "preview")}>미리보기</button>
        </div>
      </div>

      {error && (
        <div style={errorBar}>
          <Icon name="alert" size={14} />
          <span>{error}</span>
        </div>
      )}

      <div style={{ flex: 1, overflow: "auto", minHeight: 0 }}>
        {mode === "edit" ? (
          <textarea
            aria-label="문서 본문"
            value={body}
            onChange={(e) => onChange(e.target.value)}
            onBlur={flush}
            placeholder={"마크다운으로 작성하세요.\n\n# 제목\n- 목록\n- [ ] 할 일\n**굵게**, `코드`, > 인용"}
            style={textArea}
          />
        ) : (
          <div className="md-preview" style={previewBox} dangerouslySetInnerHTML={{ __html: html }} />
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
function tabBtn(active: boolean): CSSProperties {
  return {
    border: "1px solid " + (active ? "var(--accent)" : "var(--border)"),
    background: active ? "var(--accent)" : "var(--bg)",
    color: active ? "var(--on-accent)" : "var(--text2)",
    borderRadius: "var(--radius-sm)",
    padding: "4px 10px",
    fontSize: 12,
    fontWeight: 600,
    cursor: "pointer",
  };
}
const textArea: CSSProperties = {
  width: "100%",
  height: "100%",
  minHeight: 400,
  boxSizing: "border-box",
  border: "none",
  outline: "none",
  resize: "none",
  background: "var(--bg)",
  color: "var(--text)",
  fontFamily: "var(--font-mono)",
  fontSize: 14,
  lineHeight: 1.7,
  padding: "20px 24px 80px",
};
const previewBox: CSSProperties = {
  maxWidth: 760,
  margin: "0 auto",
  padding: "20px 24px 80px",
  color: "var(--text)",
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
