import { useEffect, useRef, useState, type CSSProperties } from "react";
import type { ServerConnection } from "../domain/types";
import { api } from "../api/client";
import { useServers } from "../hooks/useServers";
import { ServerPanel } from "./ServerPanel";
import { Terminal } from "./Terminal";
import { SftpBrowser } from "./SftpBrowser";
import { Icon } from "../ui/icons/Icon";
import { EMPTY_TABS, openSsh, openSftp, openLocal, closeTab, type TabsState } from "./tabs";
import { TabBar } from "./TabBar";

export interface ServerViewProps {
  businessId: string;
  projectId: string | null;
}

interface TrustPrompt {
  server: ServerConnection;
  fingerprint: string;
  keyLines: string;
  /** 신뢰 등록 후 이어서 실행할 동작(접속/파일 열기) */
  proceed: () => void;
}

/** SSH 뷰 컨테이너 — 서버 프로파일 관리 + 선택 서버 터미널/SFTP. */
export function ServerView({ businessId, projectId }: ServerViewProps) {
  const s = useServers(businessId, projectId);
  const [tabState, setTabState] = useState<TabsState>(EMPTY_TABS);
  const paneOpen = tabState.tabs.length > 0;
  const [trust, setTrust] = useState<TrustPrompt | null>(null);
  const [trustErr, setTrustErr] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  // 뷰를 벗어나면(언마운트) 열려 있던 모든 터미널 세션을 정리한다. 탭을 명시적으로
  // 닫지 않고 다른 뷰로 이동하면 백엔드 PTY 세션이 고아로 남기 때문. (SFTP 는 세션 없음)
  const tabsRef = useRef(tabState.tabs);
  tabsRef.current = tabState.tabs;
  useEffect(() => {
    return () => {
      for (const t of tabsRef.current) {
        if (t.kind !== "sftp") void api.ssh.disconnect(t.key);
      }
    };
  }, []);

  // 로컬 로그인 셸 탭 — 원격 SSH 없이 `claude login`/`cswap` 등을 로컬에서 실행하기 위한
  // 진입점. Terminal 이 요구하는 필드만 채운 합성 세션을 만들고 local=true 로 연다.
  const openLocalTerminal = () => {
    const server: ServerConnection = {
      id: `local:${crypto.randomUUID()}`,
      businessId,
      projectId,
      name: "로컬",
      host: "localhost",
      port: 0,
      username: "local",
      authType: "key",
      aiBridge: false,
    };
    setTabState((s) => openLocal(s, server));
  };

  // 접속/파일 열기 전에 호스트 키 신뢰 여부를 확인한다(MITM 방어).
  // 미신뢰 호스트는 지문을 보여주고 사용자가 수락해야 진행.
  const gate = async (server: ServerConnection, proceed: () => void) => {
    setTrustErr(null);
    try {
      const known = await api.ssh.hostStatus(server.id);
      if (known) {
        proceed();
        return;
      }
      const scan = await api.ssh.scanHost(server.id);
      setTrust({ server, fingerprint: scan.fingerprint, keyLines: scan.keyLines, proceed });
    } catch (e) {
      setTrustErr(String(e));
    }
  };

  const acceptTrust = async () => {
    if (!trust) return;
    setBusy(true);
    try {
      await api.ssh.trustHost(trust.keyLines);
      const go = trust.proceed;
      setTrust(null);
      go();
    } catch (e) {
      setTrustErr(String(e));
    } finally {
      setBusy(false);
    }
  };

  return (
    <div style={{ display: "flex", height: "100%", minHeight: 0 }}>
      <div
        style={{
          flex: paneOpen ? "0 0 300px" : 1,
          overflow: "auto",
          borderRight: paneOpen ? "1px solid var(--border)" : "none",
        }}
      >
        {trustErr && (
          <div style={errBar}>
            <Icon name="alert" size={14} />
            <span>{trustErr}</span>
          </div>
        )}
        <div style={{ padding: "12px 16px 0" }}>
          <button onClick={openLocalTerminal} style={localTerminalBtn}>
            <Icon name="server" size={14} />
            로컬 터미널
          </button>
        </div>
        <ServerPanel
          servers={s.servers}
          onCreate={(d) => void s.create(d)}
          onUpdate={(d) => void s.update(d)}
          onArchive={(id) => void s.archive(id)}
          onSetSecret={(id, secret) => void s.setSecret(id, secret)}
          onClearSecret={(id) => void s.clearSecret(id)}
          onConnect={(srv) => void gate(srv, () => setTabState((s) => openSsh(s, srv)))}
          onBrowse={(srv) => void gate(srv, () => setTabState((s) => openSftp(s, srv)))}
        />
      </div>
      {paneOpen && (
        <div style={{ flex: 1, minWidth: 0, display: "flex", flexDirection: "column" }}>
          <TabBar
            tabs={tabState.tabs}
            activeKey={tabState.activeKey}
            onSelect={(key) => setTabState((s) => ({ ...s, activeKey: key }))}
            onClose={(key) => setTabState((s) => closeTab(s, key))}
          />
          <div style={{ flex: 1, minHeight: 0, position: "relative" }}>
            {tabState.tabs.map((t) => {
              const active = t.key === tabState.activeKey;
              return (
                <div
                  key={t.key}
                  style={{ position: "absolute", inset: 0, display: active ? "block" : "none" }}
                >
                  {t.kind === "sftp" ? (
                    <SftpBrowser server={t.server} onClose={() => setTabState((s) => closeTab(s, t.key))} />
                  ) : (
                    <Terminal
                      server={t.server}
                      local={t.kind === "local"}
                      active={active}
                      onClose={() => setTabState((s) => closeTab(s, t.key))}
                    />
                  )}
                </div>
              );
            })}
          </div>
        </div>
      )}

      {trust && (
        <div style={overlay} onClick={() => setTrust(null)}>
          <div role="dialog" aria-label="호스트 키 확인" style={dialog} onClick={(e) => e.stopPropagation()}>
            <div style={{ display: "flex", alignItems: "center", gap: 8, marginBottom: 6 }}>
              <Icon name="lock" size={16} style={{ color: "var(--accent)" }} />
              <span style={{ fontSize: 14, fontWeight: 700 }}>처음 접속하는 호스트입니다</span>
            </div>
            <div style={{ fontSize: 12.5, color: "var(--text2)", lineHeight: 1.6 }}>
              <strong>{trust.server.username}@{trust.server.host}:{trust.server.port}</strong> 의 호스트 키 지문을
              확인하세요. 신뢰할 수 있는 경로(서버 관리자 고지 등)와 일치할 때만 수락하세요.
            </div>
            <pre style={fpBox}>{trust.fingerprint || "(지문을 가져오지 못했습니다)"}</pre>
            {trustErr && <div style={{ color: "#ef4444", fontSize: 12, marginBottom: 6 }}>{trustErr}</div>}
            <div style={{ display: "flex", justifyContent: "flex-end", gap: 8 }}>
              <button onClick={() => setTrust(null)} style={cancelBtn} disabled={busy}>
                취소
              </button>
              <button onClick={() => void acceptTrust()} style={acceptBtn} disabled={busy}>
                {busy ? "등록 중…" : "신뢰하고 계속"}
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}

const localTerminalBtn: CSSProperties = {
  display: "inline-flex",
  alignItems: "center",
  gap: 6,
  border: "1px solid var(--border)",
  background: "var(--bg)",
  color: "var(--text2)",
  borderRadius: "var(--radius-md)",
  padding: "7px 12px",
  fontSize: 12.5,
  fontWeight: 600,
  cursor: "pointer",
};
const overlay: CSSProperties = {
  position: "fixed",
  inset: 0,
  background: "rgba(0,0,0,.35)",
  display: "flex",
  alignItems: "center",
  justifyContent: "center",
  zIndex: 300,
};
const dialog: CSSProperties = {
  width: 460,
  maxWidth: "90%",
  background: "var(--card)",
  border: "1px solid var(--border)",
  borderRadius: "var(--radius-md)",
  boxShadow: "var(--shadow-popover)",
  padding: 18,
};
const fpBox: CSSProperties = {
  margin: "12px 0",
  padding: "10px 12px",
  background: "var(--input)",
  border: "1px solid var(--border)",
  borderRadius: "var(--radius-sm)",
  fontFamily: "var(--font-mono)",
  fontSize: 12,
  color: "var(--text)",
  whiteSpace: "pre-wrap",
  wordBreak: "break-all",
};
const errBar: CSSProperties = {
  display: "flex",
  alignItems: "center",
  gap: 6,
  margin: "12px 16px 0",
  padding: "8px 12px",
  borderRadius: "var(--radius-md)",
  background: "rgba(239,68,68,.1)",
  color: "#ef4444",
  fontSize: 12,
};
const cancelBtn: CSSProperties = {
  border: "1px solid var(--border)",
  background: "var(--bg)",
  color: "var(--text2)",
  borderRadius: "var(--radius-md)",
  padding: "7px 14px",
  fontSize: 12.5,
  fontWeight: 600,
  cursor: "pointer",
};
const acceptBtn: CSSProperties = {
  border: "none",
  background: "var(--accent)",
  color: "#fff",
  borderRadius: "var(--radius-md)",
  padding: "7px 14px",
  fontSize: 12.5,
  fontWeight: 700,
  cursor: "pointer",
};
