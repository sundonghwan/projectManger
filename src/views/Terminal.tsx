import { useCallback, useEffect, useRef, useState, type CSSProperties } from "react";
import { Terminal as XTerm } from "@xterm/xterm";
import { FitAddon } from "@xterm/addon-fit";
import "@xterm/xterm/css/xterm.css";
import { listen } from "@tauri-apps/api/event";
import { api } from "../api/client";
import type { CommandSnippet, ServerConnection } from "../domain/types";
import { SnippetBar } from "./SnippetBar";

const TERMINAL_FONT_FAMILY = 'Menlo, Monaco, "Courier New", monospace';

/** SnippetBar의 runBtn 스타일과 시각적으로 맞춘 런처 버튼(claude/codex). */
const launcherBtn: CSSProperties = {
  display: "inline-flex",
  alignItems: "center",
  background: "transparent",
  border: "1px solid #33332a",
  borderRadius: 5,
  color: "#a9c7a0",
  fontSize: 11,
  padding: "3px 10px",
  cursor: "pointer",
  fontFamily: "var(--font-mono)",
};

export interface TerminalProps {
  server: ServerConnection;
  onClose: () => void;
  /** true 면 원격 SSH 대신 로컬 로그인 셸(PTY)에 연결한다(`claude login`/`cswap` 등). */
  local?: boolean;
  /** 탭에서 현재 보이는지. 미지정 시 항상 활성으로 간주. */
  active?: boolean;
}

type BridgeAlertKind = "auth" | "quota";

interface BridgeAlertPayload {
  provider: "anthropic" | "openai";
  kind: BridgeAlertKind;
}

const BRIDGE_ALERT_MESSAGES: Record<BridgeAlertKind, string> = {
  auth: "인증 만료 — 로컬 터미널에서 재로그인/계정 전환 필요",
  quota: "쿼터 도달 — 계정 전환 고려",
};

/** SSH PTY 입출력을 xterm.js에 직접 연결한다. */
export function Terminal({ server, onClose, local, active }: TerminalProps) {
  const terminalRef = useRef<HTMLDivElement>(null);
  const xtermRef = useRef<XTerm | null>(null);
  const fitRef = useRef<FitAddon | null>(null);
  const [snippets, setSnippets] = useState<CommandSnippet[]>([]);
  const [bridgeAlert, setBridgeAlert] = useState<BridgeAlertPayload | null>(null);
  const [bridgeOn, setBridgeOn] = useState(!!server.aiBridge);
  const [bridgeBusy, setBridgeBusy] = useState(false);
  const [reconnectNonce, setReconnectNonce] = useState(0);

  useEffect(() => {
    setBridgeOn(!!server.aiBridge);
  }, [server.aiBridge]);

  const toggleBridge = useCallback(async () => {
    if (local || bridgeBusy) return;
    const next = !bridgeOn;
    setBridgeBusy(true);
    try {
      await api.server.update({
        id: server.id,
        name: server.name,
        host: server.host,
        port: server.port,
        username: server.username,
        authType: server.authType,
        keyPath: server.keyPath ?? null,
        aiBridge: next,
      });
      setBridgeOn(next);
      // 재연결은 connect effect 가 disconnect→connect 순차로 처리한다(레이스 방지).
      setReconnectNonce((n) => n + 1);
    } catch (e) {
      xtermRef.current?.write(`\r\n\x1b[31m브리지 전환 실패: ${String(e)}\x1b[0m\r\n`);
    } finally {
      setBridgeBusy(false);
    }
  }, [local, bridgeBusy, bridgeOn, server]);

  const write = useCallback(
    (data: string) => {
      if (!data) return;
      void api.ssh.write(server.id, data.normalize("NFC"));
    },
    [server.id],
  );

  const reloadSnippets = useCallback(async () => {
    try {
      setSnippets(await api.snippet.list(server.id));
    } catch {
      /* noop */
    }
  }, [server.id]);

  useEffect(() => {
    void reloadSnippets();
  }, [reloadSnippets]);

  useEffect(() => {
    const id = server.id;
    const term = new XTerm({
      disableStdin: false,
      fontFamily: TERMINAL_FONT_FAMILY,
      fontSize: 13,
      cursorBlink: true,
      letterSpacing: 0,
      theme: { background: "#0d0d0c", foreground: "#d8d8cf" },
    });
    const fit = new FitAddon();
    xtermRef.current = term;
    fitRef.current = fit;
    term.loadAddon(fit);
    if (terminalRef.current) term.open(terminalRef.current);
    const dataDisposable = term.onData(write);

    const fitAndResize = () => {
      try {
        fit.fit();
        void api.ssh.resize(id, term.rows, term.cols);
      } catch {
        /* 레이아웃 준비 전 */
      }
    };
    const scheduleFit = () => {
      window.requestAnimationFrame(() => {
        window.requestAnimationFrame(fitAndResize);
      });
    };
    scheduleFit();
    window.setTimeout(() => term.focus(), 0);

    const dataPromise = listen<string>(`terminal://data/${id}`, (e) => term.write(e.payload));
    const exitPromise = listen(`terminal://exit/${id}`, () =>
      term.write("\r\n\x1b[2m[연결이 종료되었습니다]\x1b[0m\r\n"),
    );
    const alertPromise = listen<BridgeAlertPayload>("aibridge://alert", (e) => setBridgeAlert(e.payload));

    // 먼저 disconnect 를 await 한 뒤에만 connect 를 보낸다 → 재연결 시 이전 세션을
    // 확실히 정리하고 새 세션을 띄워, connect 후 뒤늦은 disconnect 가 세션을 지우는
    // 레이스를 없앤다. (언마운트용 disconnect 는 아래 별도 effect 가 담당)
    void api.ssh
      .disconnect(id)
      .catch(() => {})
      .then(() => (local ? api.ssh.connectLocal(id) : api.ssh.connect(id)))
      .then(() => {
        fitAndResize();
      })
      .catch((err) => term.write(`\x1b[31m연결 실패: ${String(err)}\x1b[0m\r\n`));

    window.addEventListener("resize", scheduleFit);
    const terminalElement = terminalRef.current;
    const resizeObserver =
      typeof ResizeObserver === "undefined" || !terminalElement
        ? null
        : new ResizeObserver(scheduleFit);
    if (terminalElement) resizeObserver?.observe(terminalElement);

    return () => {
      window.removeEventListener("resize", scheduleFit);
      resizeObserver?.disconnect();
      dataDisposable.dispose();
      void dataPromise.then((f) => f());
      void exitPromise.then((f) => f());
      void alertPromise.then((f) => f());
      // disconnect 는 여기서 하지 않는다(재연결 시 setup 의 connect 와 레이스). 언마운트
      // 정리는 아래 [server.id] effect 가 담당한다.
      term.dispose();
      xtermRef.current = null;
      fitRef.current = null;
    };
  }, [server.id, write, local, reconnectNonce]);

  // 세션 정리는 실제 언마운트(또는 server.id 변경) 시에만 — reconnectNonce 변경으로는
  // 재실행되지 않으므로 재연결 시 connect 와 disconnect 가 겹치지 않는다.
  useEffect(() => {
    return () => {
      void api.ssh.disconnect(server.id);
    };
  }, [server.id]);

  useEffect(() => {
    if (active === false) return; // 숨겨진 탭이면 아무것도 안 함
    const term = xtermRef.current;
    const fit = fitRef.current;
    if (!term || !fit) return;
    const raf = window.requestAnimationFrame(() => {
      try {
        fit.fit();
        void api.ssh.resize(server.id, term.rows, term.cols);
        term.focus();
      } catch {
        /* 레이아웃 준비 전 */
      }
    });
    return () => window.cancelAnimationFrame(raf);
  }, [active, server.id]);

  return (
    <div style={{ display: "flex", flexDirection: "column", height: "100%", background: "#0d0d0c" }}>
      <div
        style={{
          height: 36,
          flex: "none",
          display: "flex",
          alignItems: "center",
          gap: 9,
          padding: "0 14px",
          borderBottom: "1px solid #26261f",
        }}
      >
        <span style={{ width: 8, height: 8, borderRadius: "50%", background: "#22c55e" }} />
        <span style={{ fontSize: 12.5, color: "#d8d8cf", fontFamily: TERMINAL_FONT_FAMILY }}>
          {local ? "localhost" : `${server.username}@${server.host}:${server.port}`}
        </span>
        {!local && (
          <button
            onClick={() => void toggleBridge()}
            disabled={bridgeBusy}
            title={bridgeOn ? "클릭하면 AI 브리지를 끄고 재연결합니다" : "클릭하면 AI 브리지를 켜고 재연결합니다"}
            style={{
              fontSize: 10,
              color: bridgeOn ? "#c9b458" : "#6b6b62",
              background: "transparent",
              border: bridgeOn ? "1px solid #4a441f" : "1px solid #2c2c26",
              borderRadius: 4,
              padding: "1px 5px",
              cursor: bridgeBusy ? "default" : "pointer",
              opacity: bridgeBusy ? 0.6 : 1,
            }}
          >
            {bridgeOn ? "AI 브리지" : "AI 브리지 꺼짐"}
          </button>
        )}
        {bridgeBusy && (
          <span style={{ fontSize: 10.5, color: "#8a8a80", fontFamily: TERMINAL_FONT_FAMILY }}>재연결 중…</span>
        )}
        <span style={{ flex: 1 }} />
        <button
          onClick={onClose}
          style={{
            fontSize: 11,
            color: "#f2a9a0",
            background: "transparent",
            border: "1px solid #4a2a26",
            borderRadius: 5,
            padding: "3px 9px",
            cursor: "pointer",
          }}
        >
          종료
        </button>
      </div>
      {bridgeOn && !local && bridgeAlert && (
        <div
          style={{
            flex: "none",
            display: "flex",
            alignItems: "center",
            gap: 9,
            padding: "6px 14px",
            borderBottom: "1px solid #4a441f",
            background: "#241f0f",
          }}
        >
          <span style={{ fontSize: 11.5, color: "#e8c96a", fontFamily: TERMINAL_FONT_FAMILY }}>
            [{bridgeAlert.provider}] {BRIDGE_ALERT_MESSAGES[bridgeAlert.kind]}
          </span>
          <span style={{ flex: 1 }} />
          <button
            onClick={() => setBridgeAlert(null)}
            style={{
              fontSize: 11,
              color: "#e8c96a",
              background: "transparent",
              border: "1px solid #4a441f",
              borderRadius: 5,
              padding: "1px 8px",
              cursor: "pointer",
            }}
          >
            닫기
          </button>
        </div>
      )}
      {bridgeOn && !local && (
        <div style={{ display: "flex", alignItems: "center", gap: 6, padding: "6px 14px", borderBottom: "1px solid #26261f", background: "#141412" }}>
          <button onClick={() => write("claude\r")} style={launcherBtn} title="claude 실행">
            claude
          </button>
          <button onClick={() => write("codex\r")} style={launcherBtn} title="codex 실행">
            codex
          </button>
        </div>
      )}
      <SnippetBar
        snippets={snippets}
        onRun={(cmd) => write(cmd + "\r")}
        onCreate={(name, command) => void api.snippet.create(server.id, name, command).then(reloadSnippets)}
        onDelete={(id) => void api.snippet.delete(id).then(reloadSnippets)}
      />
      <div
        ref={terminalRef}
        className="ssh-terminal-host"
        onMouseDown={() => xtermRef.current?.focus()}
        style={{ flex: 1, minHeight: 0, overflow: "hidden" }}
      />
    </div>
  );
}
