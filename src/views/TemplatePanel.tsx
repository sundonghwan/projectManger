import { useState, type CSSProperties } from "react";
import type { Template } from "../domain/types";

export interface TemplateFormData {
  name: string;
  kind: "project" | "document";
  payload: string;
}

export interface TemplatePanelProps {
  templates: Template[];
  onApply: (t: Template) => void;
  onCreate: (data: TemplateFormData) => void;
  onDelete: (id: string) => void;
}

const PROJECT_SAMPLE = '{"tasks":[{"title":"킥오프","priority":3}],"documents":[{"title":"요건정의서"}]}';
const DOCUMENT_SAMPLE = '{"blocks":[{"type":"heading","content":"{\\"text\\":\\"개요\\"}"}]}';

export function TemplatePanel({ templates, onApply, onCreate, onDelete }: TemplatePanelProps) {
  const [name, setName] = useState("");
  const [kind, setKind] = useState<"project" | "document">("project");
  const [payload, setPayload] = useState(PROJECT_SAMPLE);

  const submit = () => {
    if (!name.trim()) return;
    onCreate({ name: name.trim(), kind, payload });
    setName("");
  };

  return (
    <div>
      <div style={{ fontSize: 13, fontWeight: 600, marginBottom: 8 }}>템플릿</div>
      <div style={{ display: "flex", gap: 6, marginBottom: 6, flexWrap: "wrap" }}>
        <input aria-label="템플릿 이름" placeholder="이름" value={name} onChange={(e) => setName(e.target.value)} style={{ ...input, flex: 1 }} />
        <select
          aria-label="템플릿 종류"
          value={kind}
          onChange={(e) => {
            const k = e.target.value as "project" | "document";
            setKind(k);
            setPayload(k === "project" ? PROJECT_SAMPLE : DOCUMENT_SAMPLE);
          }}
          style={{ ...input, width: 100 }}
        >
          <option value="project">프로젝트</option>
          <option value="document">문서</option>
        </select>
        <button onClick={submit} style={primaryBtn}>추가</button>
      </div>
      <textarea aria-label="템플릿 payload" value={payload} onChange={(e) => setPayload(e.target.value)} style={{ ...input, width: "100%", minHeight: 50, fontFamily: "var(--font-mono)", fontSize: 11, marginBottom: 10 }} />

      {templates.length === 0 ? (
        <div style={{ color: "var(--text3)", fontSize: 12 }}>등록된 템플릿이 없습니다.</div>
      ) : (
        templates.map((t) => (
          <div key={t.id} style={row} data-testid={`template-${t.id}`}>
            <span style={{ flex: 1, fontSize: 13 }}>
              {t.name} <span style={{ color: "var(--text3)", fontSize: 11 }}>· {t.kind === "project" ? "프로젝트" : "문서"}</span>
            </span>
            <button onClick={() => onApply(t)} style={primaryBtn}>적용</button>
            <button onClick={() => onDelete(t.id)} style={dangerBtn} aria-label={`${t.name} 삭제`}>삭제</button>
          </div>
        ))
      )}
    </div>
  );
}

const input: CSSProperties = { border: "1px solid var(--border)", borderRadius: "var(--radius-md)", background: "var(--input)", color: "var(--text)", padding: "6px 9px", fontSize: 13, fontFamily: "inherit" };
const row: CSSProperties = { display: "flex", alignItems: "center", gap: 6, padding: "7px 0", borderTop: "1px solid var(--border)" };
const primaryBtn: CSSProperties = { border: "none", background: "var(--accent)", color: "#fff", borderRadius: "var(--radius-md)", padding: "6px 12px", fontSize: 12, fontWeight: 600, cursor: "pointer" };
const dangerBtn: CSSProperties = { border: "1px solid var(--border)", background: "var(--bg)", color: "var(--st-danger)", borderRadius: "var(--radius-md)", padding: "5px 10px", fontSize: 12, cursor: "pointer" };
