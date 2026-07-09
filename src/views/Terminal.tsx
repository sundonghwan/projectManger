import { useCallback, useEffect, useRef, useState } from "react";
import { Terminal as XTerm } from "@xterm/xterm";
import { FitAddon } from "@xterm/addon-fit";
import "@xterm/xterm/css/xterm.css";
import { listen } from "@tauri-apps/api/event";
import { api } from "../api/client";
import type { CommandSnippet, ServerConnection } from "../domain/types";
import { SnippetBar } from "./SnippetBar";

const TERMINAL_FONT_FAMILY = 'Menlo, Monaco, "Courier New", monospace';

export interface TerminalProps {
  server: ServerConnection;
  onClose: () => void;
  /** true 면 원격 SSH 대신 로컬 로그인 셸(PTY)에 연결한다(`claude login`/`cswap` 등). */
  local?: boolean;
}

/** SSH PTY 입출력을 xterm.js에 직접 연결한다. */
export function Terminal({ server, onClose, local }: TerminalProps) {
  const terminalRef = useRef<HTMLDivElement>(null);
  const xtermRef = useRef<XTerm | null>(null);
  const [snippets, setSnippets] = useState<CommandSnippet[]>([]);

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

    void (local ? api.ssh.connectLocal(id) : api.ssh.connect(id))
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
      void api.ssh.disconnect(id);
      term.dispose();
      xtermRef.current = null;
    };
  }, [server.id, write, local]);

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
        {server.aiBridge && (
          <span style={{ fontSize: 10, color: "#c9b458", border: "1px solid #4a441f", borderRadius: 4, padding: "1px 5px" }}>AI 브리지</span>
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
