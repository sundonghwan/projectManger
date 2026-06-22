import { useState, type CSSProperties } from "react";
import type { AuthType, ServerConnection } from "../domain/types";
import { Icon } from "../ui/icons/Icon";

export interface ServerFormData {
  name: string;
  host: string;
  port: number;
  username: string;
  authType: AuthType;
  keyPath: string | null;
}

export interface ServerPanelProps {
  servers: ServerConnection[];
  onCreate: (data: ServerFormData) => void;
  onArchive: (id: number) => void;
  onSetSecret: (id: number, secret: string) => void;
  onConnect: (server: ServerConnection) => void;
  onBrowse?: (server: ServerConnection) => void;
}

const AUTH_LABEL: Record<AuthType, string> = { key: "키 기반", password: "비밀번호", agent: "에이전트" };

const EMPTY: ServerFormData = { name: "", host: "", port: 22, username: "", authType: "key", keyPath: null };

export function ServerPanel({ servers, onCreate, onArchive, onSetSecret, onConnect, onBrowse }: ServerPanelProps) {
  const [form, setForm] = useState<ServerFormData>(EMPTY);
  const [secretInputs, setSecretInputs] = useState<Record<number, string>>({});

  const submit = () => {
    if (!form.name.trim() || !form.host.trim() || !form.username.trim()) return;
    onCreate(form);
    setForm(EMPTY);
  };

  return (
    <div style={{ padding: "16px 20px", maxWidth: 760 }}>
      <div style={{ fontSize: 15, fontWeight: 600, marginBottom: 12 }}>서버 연결</div>

      {/* 추가 폼 */}
      <div style={formBox}>
        <div style={{ display: "flex", gap: 8, flexWrap: "wrap" }}>
          <input aria-label="이름" placeholder="이름" value={form.name} onChange={(e) => setForm({ ...form, name: e.target.value })} style={{ ...input, flex: 2 }} />
          <input aria-label="호스트" placeholder="host" value={form.host} onChange={(e) => setForm({ ...form, host: e.target.value })} style={{ ...input, flex: 2 }} />
          <input aria-label="포트" type="number" value={form.port} onChange={(e) => setForm({ ...form, port: Number(e.target.value) })} style={{ ...input, width: 70 }} />
        </div>
        <div style={{ display: "flex", gap: 8, marginTop: 8, flexWrap: "wrap" }}>
          <input aria-label="사용자" placeholder="user" value={form.username} onChange={(e) => setForm({ ...form, username: e.target.value })} style={{ ...input, flex: 1 }} />
          <select aria-label="인증 방식" value={form.authType} onChange={(e) => setForm({ ...form, authType: e.target.value as AuthType })} style={{ ...input, width: 110 }}>
            <option value="key">키 기반</option>
            <option value="password">비밀번호</option>
            <option value="agent">에이전트</option>
          </select>
          <button onClick={submit} style={primaryBtn}>추가</button>
        </div>
      </div>

      {/* 목록 */}
      {servers.length === 0 ? (
        <div style={{ color: "var(--text3)", fontSize: 13, marginTop: 16 }}>등록된 서버가 없습니다.</div>
      ) : (
        servers.map((s) => (
          <div key={s.id} style={card} data-testid={`server-${s.id}`}>
            <div style={{ display: "flex", alignItems: "center", gap: 8 }}>
              <span style={{ display: "flex", alignItems: "center", gap: 6, fontSize: 14, fontWeight: 600, flex: 1 }}>
                <Icon name="server" size={15} style={{ color: "var(--text2)" }} />
                {s.name}
              </span>
              <span style={authBadge}>{AUTH_LABEL[s.authType as AuthType]}</span>
              {s.secretRef && (
                <span style={{ display: "inline-flex", alignItems: "center", gap: 4, fontSize: 11, color: "var(--st-done)" }}>
                  <Icon name="lock" size={12} /> 저장됨
                </span>
              )}
            </div>
            <div style={{ fontSize: 12, color: "var(--text2)", fontFamily: "var(--font-mono)", margin: "4px 0 8px" }}>
              {s.username}@{s.host}:{s.port}
            </div>
            <div style={{ display: "flex", gap: 6, alignItems: "center" }}>
              <button onClick={() => onConnect(s)} style={primaryBtn}>접속</button>
              {onBrowse && (
                <button onClick={() => onBrowse(s)} style={secondaryBtn} aria-label={`${s.name} 파일`}>파일</button>
              )}
              {s.authType !== "agent" && (
                <>
                  <input
                    aria-label={`${s.name} 비밀값`}
                    type="password"
                    placeholder={s.authType === "password" ? "비밀번호" : "패스프레이즈"}
                    value={secretInputs[s.id] ?? ""}
                    onChange={(e) => setSecretInputs({ ...secretInputs, [s.id]: e.target.value })}
                    style={{ ...input, width: 140 }}
                  />
                  <button
                    onClick={() => {
                      const v = secretInputs[s.id];
                      if (v) {
                        onSetSecret(s.id, v);
                        setSecretInputs({ ...secretInputs, [s.id]: "" });
                      }
                    }}
                    style={secondaryBtn}
                  >
                    시크릿 저장
                  </button>
                </>
              )}
              <span style={{ flex: 1 }} />
              <button onClick={() => onArchive(s.id)} style={dangerBtn} aria-label={`${s.name} 보관`}>보관</button>
            </div>
          </div>
        ))
      )}
      <div style={{ display: "flex", alignItems: "center", gap: 6, fontSize: 11, color: "var(--text3)", marginTop: 12 }}>
        <Icon name="alert" size={13} />
        <span>비밀번호·패스프레이즈는 OS 키체인에 저장되며 DB에는 참조만 기록됩니다.</span>
      </div>
    </div>
  );
}

const formBox: CSSProperties = { border: "1px solid var(--border)", borderRadius: "var(--radius-md)", padding: 12, background: "var(--card)" };
const card: CSSProperties = { border: "1px solid var(--border)", borderRadius: "var(--radius-md)", padding: 12, marginTop: 10, background: "var(--card)" };
const input: CSSProperties = {
  border: "1px solid var(--border)",
  borderRadius: "var(--radius-md)",
  background: "var(--input)",
  color: "var(--text)",
  padding: "6px 9px",
  fontSize: 13,
  fontFamily: "inherit",
};
const authBadge: CSSProperties = { fontSize: 11, fontWeight: 600, color: "var(--text2)", background: "var(--hover)", borderRadius: 4, padding: "2px 7px" };
const primaryBtn: CSSProperties = { border: "none", background: "var(--accent)", color: "#fff", borderRadius: "var(--radius-md)", padding: "6px 14px", fontSize: 12, fontWeight: 600, cursor: "pointer" };
const secondaryBtn: CSSProperties = { border: "1px solid var(--border)", background: "var(--bg)", color: "var(--text)", borderRadius: "var(--radius-md)", padding: "6px 10px", fontSize: 12, cursor: "pointer" };
const dangerBtn: CSSProperties = { border: "1px solid var(--border)", background: "var(--bg)", color: "var(--st-danger)", borderRadius: "var(--radius-md)", padding: "6px 12px", fontSize: 12, fontWeight: 600, cursor: "pointer" };
