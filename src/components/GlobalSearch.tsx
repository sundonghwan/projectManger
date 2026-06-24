import { useRef, useState, type CSSProperties } from "react";
import type { SearchHit, SearchKind } from "../domain/types";
import { Icon, type IconName } from "../ui/icons/Icon";

export interface GlobalSearchProps {
  onSearch: (query: string) => Promise<SearchHit[]>;
  onPick: (hit: SearchHit) => void;
}

const KIND_ICON: Record<SearchKind, IconName> = {
  business: "business",
  project: "folder",
  task: "check-square",
  document: "document",
  deliverable: "deliverable",
  memo: "memo",
};
const KIND_LABEL: Record<SearchKind, string> = {
  business: "사업",
  project: "프로젝트",
  task: "태스크",
  document: "문서",
  deliverable: "산출물",
  memo: "메모",
};

export function GlobalSearch({ onSearch, onPick }: GlobalSearchProps) {
  const [query, setQuery] = useState("");
  const [results, setResults] = useState<SearchHit[]>([]);
  const [open, setOpen] = useState(false);
  const timer = useRef<ReturnType<typeof setTimeout> | null>(null);

  const runSearch = (q: string) => {
    setQuery(q);
    if (timer.current) clearTimeout(timer.current);
    if (!q.trim()) {
      setResults([]);
      setOpen(false);
      return;
    }
    timer.current = setTimeout(async () => {
      const hits = await onSearch(q);
      setResults(hits);
      setOpen(true);
    }, 250);
  };

  const pick = (hit: SearchHit) => {
    onPick(hit);
    setQuery("");
    setResults([]);
    setOpen(false);
  };

  return (
    <div style={{ position: "relative", padding: "10px 8px 4px" }}>
      <input
        aria-label="검색"
        value={query}
        onChange={(e) => runSearch(e.target.value)}
        placeholder="검색 (사업·프로젝트·태스크·문서)"
        style={inputStyle}
      />
      {open && (
        <div style={dropdown} role="listbox">
          {results.length === 0 ? (
            <div style={{ padding: "8px 10px", color: "var(--text3)", fontSize: 12 }}>
              결과 없음
            </div>
          ) : (
            results.map((hit) => (
              <button
                key={`${hit.kind}:${hit.id}`}
                role="option"
                aria-selected={false}
                onClick={() => pick(hit)}
                style={resultRow}
              >
                <span
                  style={{
                    width: 16,
                    display: "flex",
                    alignItems: "center",
                    justifyContent: "center",
                    color: "var(--text2)",
                  }}
                >
                  <Icon name={KIND_ICON[hit.kind]} size={14} />
                </span>
                <span style={{ flex: 1, overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
                  {hit.title}
                </span>
                <span style={{ fontSize: 11, color: "var(--text3)" }}>{KIND_LABEL[hit.kind]}</span>
              </button>
            ))
          )}
        </div>
      )}
    </div>
  );
}

const inputStyle: CSSProperties = {
  width: "100%",
  border: "1px solid var(--border)",
  borderRadius: "var(--radius-md)",
  background: "var(--input)",
  color: "var(--text)",
  padding: "6px 10px",
  fontSize: 13,
  fontFamily: "inherit",
};
const dropdown: CSSProperties = {
  position: "absolute",
  left: 8,
  right: 8,
  top: "100%",
  zIndex: 50,
  background: "var(--card)",
  border: "1px solid var(--border)",
  borderRadius: "var(--radius-md)",
  boxShadow: "var(--shadow-popover)",
  maxHeight: 320,
  overflowY: "auto",
};
const resultRow: CSSProperties = {
  display: "flex",
  alignItems: "center",
  gap: 8,
  width: "100%",
  border: "none",
  background: "transparent",
  color: "var(--text)",
  padding: "7px 10px",
  fontSize: 13,
  cursor: "pointer",
  textAlign: "left",
};
