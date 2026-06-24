import { useState } from "react";
import type { ServerConnection } from "../domain/types";
import { useServers } from "../hooks/useServers";
import { ServerPanel } from "./ServerPanel";
import { Terminal } from "./Terminal";
import { SftpBrowser } from "./SftpBrowser";

export interface ServerViewProps {
  businessId: number;
  projectId: number | null;
}

/** SSH 뷰 컨테이너 — 서버 프로파일 관리 + 선택 서버 터미널. */
export function ServerView({ businessId, projectId }: ServerViewProps) {
  const s = useServers(businessId, projectId);
  const [connecting, setConnecting] = useState<ServerConnection | null>(null);
  const [browsing, setBrowsing] = useState<ServerConnection | null>(null);
  const paneOpen = connecting || browsing;

  return (
    <div style={{ display: "flex", height: "100%", minHeight: 0 }}>
      <div
        style={{
          flex: paneOpen ? "0 0 300px" : 1,
          overflow: "auto",
          borderRight: paneOpen ? "1px solid var(--border)" : "none",
        }}
      >
        <ServerPanel
          servers={s.servers}
          onCreate={(d) => void s.create(d)}
          onUpdate={(d) => void s.update(d)}
          onArchive={(id) => void s.archive(id)}
          onSetSecret={(id, secret) => void s.setSecret(id, secret)}
          onConnect={(srv) => {
            setBrowsing(null);
            setConnecting(srv);
          }}
          onBrowse={(srv) => {
            setConnecting(null);
            setBrowsing(srv);
          }}
        />
      </div>
      {connecting && (
        <div style={{ flex: 1, minWidth: 0 }}>
          <Terminal server={connecting} onClose={() => setConnecting(null)} />
        </div>
      )}
      {browsing && !connecting && (
        <div style={{ flex: 1, minWidth: 0 }}>
          <SftpBrowser server={browsing} onClose={() => setBrowsing(null)} />
        </div>
      )}
    </div>
  );
}
