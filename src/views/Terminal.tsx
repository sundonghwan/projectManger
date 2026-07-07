import { useCallback, useEffect, useRef, useState, type CSSProperties, type KeyboardEvent } from "react";
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
}

/** xterm.js는 출력 렌더러로만 쓰고, 입력은 일반 textarea로 받아 macOS IME를 보존한다. */
export function Terminal({ server, onClose }: TerminalProps) {
  const terminalRef = useRef<HTMLDivElement>(null);
  const inputRef = useRef<HTMLTextAreaElement>(null);
  const composingRef = useRef(false);
  const [snippets, setSnippets] = useState<CommandSnippet[]>([]);

  const write = useCallback(
    (data: string) => {
      if (!data) return;
      void api.ssh.write(server.id, data.normalize("NFC"));
    },
    [server.id],
  );

  const clearInput = useCallback(() => {
    if (inputRef.current) {
      inputRef.current.value = "";
    }
  }, []);

  const flushInput = useCallback(() => {
    const value = inputRef.current?.value ?? "";
    if (!value) return;
    write(value);
    clearInput();
  }, [clearInput, write]);

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
      disableStdin: true,
      fontFamily: TERMINAL_FONT_FAMILY,
      fontSize: 13,
      cursorBlink: true,
      letterSpacing: 0,
      theme: { background: "#0d0d0c", foreground: "#d8d8cf" },
    });
    const fit = new FitAddon();
    term.loadAddon(fit);
    if (terminalRef.current) term.open(terminalRef.current);
    window.setTimeout(() => inputRef.current?.focus(), 0);
    try {
      fit.fit();
    } catch {
      /* 레이아웃 준비 전 */
    }

    const dataPromise = listen<string>(`terminal://data/${id}`, (e) => term.write(e.payload));
    const exitPromise = listen(`terminal://exit/${id}`, () =>
      term.write("\r\n\x1b[2m[연결이 종료되었습니다]\x1b[0m\r\n"),
    );

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

  const handleKeyDown = (event: KeyboardEvent<HTMLTextAreaElement>) => {
    if (composingRef.current) return;

    if (event.ctrlKey && !event.metaKey && !event.altKey && event.key.length === 1) {
      const code = event.key.toUpperCase().charCodeAt(0) - 64;
      if (code >= 1 && code <= 26) {
        event.preventDefault();
        clearInput();
        write(String.fromCharCode(code));
        return;
      }
    }

    const keyMap: Record<string, string> = {
      Enter: "\r",
      Backspace: "\x7f",
      Tab: "\t",
      Escape: "\x1b",
      ArrowUp: "\x1b[A",
      ArrowDown: "\x1b[B",
      ArrowRight: "\x1b[C",
      ArrowLeft: "\x1b[D",
      Delete: "\x1b[3~",
      Home: "\x1b[H",
      End: "\x1b[F",
      PageUp: "\x1b[5~",
      PageDown: "\x1b[6~",
    };
    const sequence = keyMap[event.key];
    if (sequence) {
      event.preventDefault();
      flushInput();
      write(sequence);
    }
  };

  const focusInput = () => {
    window.setTimeout(() => inputRef.current?.focus(), 0);
  };

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
        onRun={(cmd) => write(cmd + "\r")}
        onCreate={(name, command) => void api.snippet.create(server.id, name, command).then(reloadSnippets)}
        onDelete={(id) => void api.snippet.delete(id).then(reloadSnippets)}
      />
      <textarea
        ref={inputRef}
        aria-label="터미널 입력"
        rows={1}
        spellCheck={false}
        placeholder="터미널 입력"
        onCompositionStart={() => {
          composingRef.current = true;
        }}
        onCompositionEnd={() => {
          composingRef.current = false;
          window.setTimeout(flushInput, 0);
        }}
        onInput={() => {
          if (!composingRef.current) flushInput();
        }}
        onKeyDown={handleKeyDown}
        style={inputCapture}
      />
      <div
        ref={terminalRef}
        className="ssh-terminal-host"
        onMouseDown={focusInput}
        style={{ flex: 1, minHeight: 0, padding: 8, overflow: "hidden" }}
      />
    </div>
  );
}

const inputCapture: CSSProperties = {
  flex: "none",
  height: 30,
  resize: "none",
  outline: "none",
  border: "none",
  borderBottom: "1px solid #26261f",
  background: "#10100e",
  color: "#d8d8cf",
  caretColor: "#d8d8cf",
  padding: "6px 14px",
  fontFamily: TERMINAL_FONT_FAMILY,
  fontSize: 13,
  lineHeight: "18px",
};
