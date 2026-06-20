import { useState } from "react";
import type { ServerConnection } from "../domain/types";
import { useServers } from "../hooks/useServers";
import { ServerPanel } from "./ServerPanel";
import { Terminal } from "./Terminal";

export interface ServerViewProps {
  businessId: number;
  projectId: number | null;
}

/** SSH 뷰 컨테이너 — 서버 프로파일 관리 + 선택 서버 터미널. */
export function ServerView({ businessId, projectId }: ServerViewProps) {
  const s = useServers(businessId, projectId);
  const [connecting, setConnecting] = useState<ServerConnection | null>(null);

  return (
    <div style={{ display: "flex", height: "100%", minHeight: 0 }}>
      <div
        style={{
          flex: connecting ? "0 0 300px" : 1,
          overflow: "auto",
          borderRight: connecting ? "1px solid var(--border)" : "none",
        }}
      >
        <ServerPanel
          servers={s.servers}
          onCreate={(d) => void s.create(d)}
          onArchive={(id) => void s.archive(id)}
          onSetSecret={(id, secret) => void s.setSecret(id, secret)}
          onConnect={setConnecting}
        />
      </div>
      {connecting && (
        <div style={{ flex: 1, minWidth: 0 }}>
          <Terminal server={connecting} onClose={() => setConnecting(null)} />
        </div>
      )}
    </div>
  );
}
