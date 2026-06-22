import type { CSSProperties } from "react";
import type { SftpEntry } from "../domain/types";
import { Icon } from "../ui/icons/Icon";

export interface SftpListProps {
  path: string;
  entries: SftpEntry[];
  error: string | null;
  onUp: () => void;
  onOpen: (entry: SftpEntry) => void;
  onClose: () => void;
}

export function SftpList({ path, entries, error, onUp, onOpen, onClose }: SftpListProps) {
  return (
    <div style={{ display: "flex", flexDirection: "column", height: "100%", background: "var(--bg)" }}>
      <div style={header}>
        <button onClick={onUp} style={btn} aria-label="상위 폴더"><Icon name="arrow-up" size={14} /></button>
        <span style={{ flex: 1, fontFamily: "var(--font-mono)", fontSize: 12, color: "var(--text2)", overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
          {path || "/"}
        </span>
        <button onClick={onClose} style={btn} aria-label="파일 브라우저 닫기"><Icon name="close" size={14} /></button>
      </div>
      {error ? (
        <div style={{ display: "flex", alignItems: "center", gap: 6, padding: 16, color: "var(--st-danger)", fontSize: 13 }}>
          <Icon name="alert" size={15} />
          <span>{error}</span>
        </div>
      ) : entries.length === 0 ? (
        <div style={{ padding: 16, color: "var(--text3)", fontSize: 13 }}>(비어 있음)</div>
      ) : (
        <div style={{ flex: 1, overflow: "auto" }}>
          {entries.map((e) => (
            <div
              key={e.name}
              data-testid={`sftp-${e.name}`}
              onClick={() => e.isDir && onOpen(e)}
              style={{ ...row, cursor: e.isDir ? "pointer" : "default" }}
            >
              <span style={{ width: 18, display: "flex", alignItems: "center", color: "var(--text2)" }}>
                <Icon name={e.isDir ? "folder" : "document"} size={15} />
              </span>
              <span style={{ flex: 1, fontSize: 13, overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
                {e.name}
              </span>
              {!e.isDir && <span style={{ fontSize: 11, color: "var(--text3)" }}>{e.size}B</span>}
            </div>
          ))}
        </div>
      )}
    </div>
  );
}

const header: CSSProperties = { height: 36, flex: "none", display: "flex", alignItems: "center", gap: 8, padding: "0 12px", borderBottom: "1px solid var(--border)" };
const row: CSSProperties = { display: "flex", alignItems: "center", gap: 8, padding: "6px 14px", borderBottom: "1px solid var(--border)" };
const btn: CSSProperties = { display: "inline-flex", alignItems: "center", justifyContent: "center", border: "1px solid var(--border)", background: "var(--bg)", color: "var(--text2)", borderRadius: 5, padding: "5px 8px", fontSize: 12, cursor: "pointer" };
