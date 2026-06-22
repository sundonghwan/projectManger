import { useEffect, useState, type CSSProperties } from "react";
import { useDocuments } from "../hooks/useDocuments";
import { DocumentList } from "./DocumentList";
import { DocEditor } from "./DocEditor";
import { Icon } from "../ui/icons/Icon";

export interface DocumentsViewProps {
  businessId: number;
  projectId: number | null;
  onChanged: () => void;
  /** 진입 시 자동으로 열 문서 id (검색 등). 열린 뒤 onOpened 로 소비 통지. */
  initialOpenDocId?: number | null;
  onOpened?: () => void;
}

/** 문서 뷰 컨테이너 — 목록과 편집기를 전환한다(산출물과 동일한 단일 진입 패턴). */
export function DocumentsView({ businessId, projectId, onChanged, initialOpenDocId, onOpened }: DocumentsViewProps) {
  const d = useDocuments(businessId, projectId, onChanged);
  const [openId, setOpenId] = useState<number | null>(null);
  const openDoc = d.documents.find((x) => x.id === openId) ?? null;

  // 검색 등으로 특정 문서가 지정되면 해당 문서를 열고 소비 통지한다.
  useEffect(() => {
    if (initialOpenDocId != null) {
      setOpenId(initialOpenDocId);
      onOpened?.();
    }
  }, [initialOpenDocId, onOpened]);

  if (openDoc) {
    return (
      <div style={{ display: "flex", flexDirection: "column", height: "100%", minHeight: 0 }}>
        <div style={backBar}>
          <button onClick={() => setOpenId(null)} style={backBtn} aria-label="문서 목록으로">
            <Icon name="chevron-right" size={14} style={{ transform: "rotate(180deg)" }} /> 목록
          </button>
          <span style={{ fontSize: 12, color: "var(--text3)", overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
            {openDoc.title}
          </span>
        </div>
        <div style={{ flex: 1, minHeight: 0 }}>
          <DocEditor key={openDoc.id} document={openDoc} />
        </div>
      </div>
    );
  }

  return (
    <DocumentList
      documents={d.documents}
      error={d.error}
      onCreate={(title) => {
        void d.create(title).then((doc) => {
          if (doc) setOpenId(doc.id);
        });
      }}
      onOpen={(id) => setOpenId(id)}
      onRename={(id, title) => void d.rename(id, title)}
      onArchive={(id) => {
        if (!window.confirm("이 문서를 삭제할까요? (휴지통에서 복구 가능)")) return;
        if (openId === id) setOpenId(null);
        void d.archive(id);
      }}
    />
  );
}

const backBar: CSSProperties = {
  flexShrink: 0,
  display: "flex",
  alignItems: "center",
  gap: 10,
  height: 38,
  padding: "0 16px",
  borderBottom: "1px solid var(--border)",
};
const backBtn: CSSProperties = {
  display: "inline-flex",
  alignItems: "center",
  gap: 3,
  border: "1px solid var(--border)",
  background: "var(--bg)",
  color: "var(--text2)",
  borderRadius: "var(--radius-sm)",
  padding: "4px 10px",
  fontSize: 12,
  fontWeight: 600,
  cursor: "pointer",
};
