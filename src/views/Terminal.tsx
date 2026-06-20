import { useCallback, useEffect, useRef, useState } from "react";
import { Terminal as XTerm } from "@xterm/xterm";
import { FitAddon } from "@xterm/addon-fit";
import "@xterm/xterm/css/xterm.css";
import { listen } from "@tauri-apps/api/event";
import { api } from "../api/client";
import type { CommandSnippet, ServerConnection } from "../domain/types";
import { SnippetBar } from "./SnippetBar";

export interface TerminalProps {
  server: ServerConnection;
  onClose: () => void;
}

/** xterm.js 터미널 — Rust PTY(ssh) 세션과 양방향 스트리밍. */
export function Terminal({ server, onClose }: TerminalProps) {
  const ref = useRef<HTMLDivElement>(null);
  const [snippets, setSnippets] = useState<CommandSnippet[]>([]);

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
      fontFamily: "var(--font-mono), monospace",
      fontSize: 13,
      cursorBlink: true,
      theme: { background: "#0d0d0c", foreground: "#d8d8cf" },
    });
    const fit = new FitAddon();
    term.loadAddon(fit);
    if (ref.current) term.open(ref.current);
    try {
      fit.fit();
    } catch {
      /* 레이아웃 준비 전 */
    }

    const dataPromise = listen<string>(`terminal://data/${id}`, (e) => term.write(e.payload));
    const exitPromise = listen(`terminal://exit/${id}`, () =>
      term.write("\r\n\x1b[2m[연결이 종료되었습니다]\x1b[0m\r\n"),
    );

    term.onData((d) => void api.ssh.write(id, d));

    void api.ssh
      .connect(id)
      .then(() => api.ssh.resize(id, term.rows, term.cols))
      .catch((err) => term.write(`\x1b[31m연결 실패: ${String(err)}\x1b[0m\r\n`));

    const onResize = () => {
      try {
        fit.fit();
        void api.ssh.resize(id, term.rows, term.cols);
      } catch {
        /* noop */
      }
    };
    window.addEventListener("resize", onResize);

    return () => {
      window.removeEventListener("resize", onResize);
      void dataPromise.then((f) => f());
      void exitPromise.then((f) => f());
      void api.ssh.disconnect(id);
      term.dispose();
    };
  }, [server.id]);

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
        <span style={{ fontSize: 12.5, color: "#d8d8cf", fontFamily: "var(--font-mono)" }}>
          {server.username}@{server.host}:{server.port}
        </span>
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
        onRun={(cmd) => void api.ssh.write(server.id, cmd + "\r")}
        onCreate={(name, command) => void api.snippet.create(server.id, name, command).then(reloadSnippets)}
        onDelete={(id) => void api.snippet.delete(id).then(reloadSnippets)}
      />
      <div ref={ref} style={{ flex: 1, minHeight: 0, padding: 8 }} />
    </div>
  );
}
