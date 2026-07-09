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
  aiBridge: boolean;
}

export interface ServerPanelProps {
  servers: ServerConnection[];
  onCreate: (data: ServerFormData) => void;
  /** 기존 서버 접속정보 수정 */
  onUpdate?: (data: ServerFormData & { id: string }) => void;
  onArchive: (id: string) => void;
  onSetSecret: (id: string, secret: string) => void;
  onClearSecret?: (id: string) => void;
  onConnect: (server: ServerConnection) => void;
  onBrowse?: (server: ServerConnection) => void;
}

const AUTH_LABEL: Record<AuthType, string> = { key: "키 기반", password: "비밀번호", agent: "에이전트" };

const EMPTY: ServerFormData = { name: "", host: "", port: 22, username: "", authType: "key", keyPath: null, aiBridge: false };

export function ServerPanel({ servers, onCreate, onUpdate, onArchive, onSetSecret, onClearSecret, onConnect, onBrowse }: ServerPanelProps) {
  const [form, setForm] = useState<ServerFormData>(EMPTY);
  const [secretInputs, setSecretInputs] = useState<Record<string, string>>({});
  const [editingSecretIds, setEditingSecretIds] = useState<Record<string, boolean>>({});
  const [editingId, setEditingId] = useState<string | null>(null);
  const [editForm, setEditForm] = useState<ServerFormData>(EMPTY);

  const startEdit = (s: ServerConnection) => {
    setEditingId(s.id);
    setEditForm({
      name: s.name,
      host: s.host,
      port: s.port,
      username: s.username,
      authType: s.authType as AuthType,
      keyPath: s.keyPath ?? null,
      aiBridge: s.aiBridge ?? false,
    });
  };
  const submitEdit = () => {
    if (editingId == null) return;
    if (!editForm.name.trim() || !editForm.host.trim() || !editForm.username.trim()) return;
    onUpdate?.({ id: editingId, ...editForm });
    setEditingId(null);
  };

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
        <label style={checkboxLabel}>
          <input
            type="checkbox"
            checked={form.aiBridge}
            onChange={(e) => setForm({ ...form, aiBridge: e.target.checked })}
          />
          AI 자격증명 브리지 (claude/codex 원격 사용)
        </label>
      </div>

      {/* 목록 */}
      {servers.length === 0 ? (
        <div style={{ color: "var(--text3)", fontSize: 13, marginTop: 16 }}>등록된 서버가 없습니다.</div>
      ) : (
        servers.map((s) => (
          <div key={s.id} style={card} data-testid={`server-${s.id}`}>
            {editingId === s.id ? (
              <div style={{ display: "flex", flexDirection: "column", gap: 8 }}>
                <div style={{ display: "flex", gap: 8, flexWrap: "wrap" }}>
                  <input aria-label="이름 수정" placeholder="이름" value={editForm.name} onChange={(e) => setEditForm({ ...editForm, name: e.target.value })} style={{ ...input, flex: 2, minWidth: 0 }} />
                  <input aria-label="호스트 수정" placeholder="host" value={editForm.host} onChange={(e) => setEditForm({ ...editForm, host: e.target.value })} style={{ ...input, flex: 2, minWidth: 0 }} />
                  <input aria-label="포트 수정" type="number" value={editForm.port} onChange={(e) => setEditForm({ ...editForm, port: Number(e.target.value) })} style={{ ...input, width: 70 }} />
                </div>
                <div style={{ display: "flex", gap: 8, flexWrap: "wrap", alignItems: "center" }}>
                  <input aria-label="사용자 수정" placeholder="user" value={editForm.username} onChange={(e) => setEditForm({ ...editForm, username: e.target.value })} style={{ ...input, flex: 1, minWidth: 0 }} />
                  <select aria-label="인증 방식 수정" value={editForm.authType} onChange={(e) => setEditForm({ ...editForm, authType: e.target.value as AuthType })} style={{ ...input, width: 110 }}>
                    <option value="key">키 기반</option>
                    <option value="password">비밀번호</option>
                    <option value="agent">에이전트</option>
                  </select>
                  <span style={{ flex: 1 }} />
                  <button onClick={submitEdit} style={primaryBtn}>저장</button>
                  <button onClick={() => setEditingId(null)} style={secondaryBtn}>취소</button>
                </div>
                <label style={checkboxLabel}>
                  <input
                    type="checkbox"
                    checked={editForm.aiBridge}
                    onChange={(e) => setEditForm({ ...editForm, aiBridge: e.target.checked })}
                  />
                  AI 자격증명 브리지 (claude/codex 원격 사용)
                </label>
              </div>
            ) : (
              <>
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
            <div style={{ display: "flex", gap: 6, alignItems: "center", flexWrap: "wrap" }}>
              <button onClick={() => onConnect(s)} style={primaryBtn}>접속</button>
              {onBrowse && (
                <button onClick={() => onBrowse(s)} style={secondaryBtn} aria-label={`${s.name} 파일`}>파일</button>
              )}
              {onUpdate && (
                <button onClick={() => startEdit(s)} style={secondaryBtn} aria-label={`${s.name} 수정`}>수정</button>
              )}
              <span style={{ flex: 1 }} />
              <button onClick={() => onArchive(s.id)} style={dangerBtn} aria-label={`${s.name} 보관`}>보관</button>
            </div>
            {s.authType !== "agent" && (
              <SecretEditor
                server={s}
                value={secretInputs[s.id] ?? ""}
                editing={Boolean(editingSecretIds[s.id]) || !s.secretRef}
                onEdit={() => setEditingSecretIds({ ...editingSecretIds, [s.id]: true })}
                onCancel={() => {
                  setEditingSecretIds({ ...editingSecretIds, [s.id]: false });
                  setSecretInputs({ ...secretInputs, [s.id]: "" });
                }}
                onChange={(value) => setSecretInputs({ ...secretInputs, [s.id]: value })}
                onSave={() => {
                  const v = secretInputs[s.id];
                  if (v) {
                    onSetSecret(s.id, v);
                    setSecretInputs({ ...secretInputs, [s.id]: "" });
                    setEditingSecretIds({ ...editingSecretIds, [s.id]: false });
                  }
                }}
                onClear={
                  onClearSecret
                    ? () => {
                        onClearSecret(s.id);
                        setSecretInputs({ ...secretInputs, [s.id]: "" });
                        setEditingSecretIds({ ...editingSecretIds, [s.id]: false });
                      }
                    : undefined
                }
              />
            )}
              </>
            )}
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

interface SecretEditorProps {
  server: ServerConnection;
  value: string;
  editing: boolean;
  onEdit: () => void;
  onCancel: () => void;
  onChange: (value: string) => void;
  onSave: () => void;
  onClear?: () => void;
}

function secretLabel(authType: AuthType): string {
  return authType === "password" ? "비밀번호" : "패스프레이즈";
}

function SecretEditor({ server, value, editing, onEdit, onCancel, onChange, onSave, onClear }: SecretEditorProps) {
  const label = secretLabel(server.authType as AuthType);
  if (!editing) {
    return (
      <div style={secretRow}>
        <span style={secretHint}>{label}가 OS 키체인에 저장되어 있습니다.</span>
        <button onClick={onEdit} style={secondaryBtn} aria-label={`${server.name} ${label} 변경`}>
          {label} 변경
        </button>
        {onClear && (
          <button onClick={onClear} style={dangerBtn} aria-label={`${server.name} ${label} 삭제`}>
            삭제
          </button>
        )}
      </div>
    );
  }
  return (
    <div style={secretRow}>
      <input
        aria-label={`${server.name} 비밀값`}
        type="password"
        placeholder={label}
        value={value}
        onChange={(e) => onChange(e.target.value)}
        style={{ ...input, flex: 1, minWidth: 0 }}
      />
      <button onClick={onSave} style={secondaryBtn}>
        {server.secretRef ? `${label} 변경 저장` : "시크릿 저장"}
      </button>
      {server.secretRef && (
        <button onClick={onCancel} style={secondaryBtn}>
          취소
        </button>
      )}
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
const checkboxLabel: CSSProperties = { display: "flex", alignItems: "center", gap: 6, fontSize: 12, color: "var(--text2)", marginTop: 8, cursor: "pointer" };
const secretRow: CSSProperties = { display: "flex", gap: 6, alignItems: "center", marginTop: 8, flexWrap: "wrap" };
const secretHint: CSSProperties = { flex: 1, minWidth: 160, fontSize: 12, color: "var(--text2)" };
const primaryBtn: CSSProperties = { border: "none", background: "var(--accent)", color: "#fff", borderRadius: "var(--radius-md)", padding: "6px 14px", fontSize: 12, fontWeight: 600, cursor: "pointer", whiteSpace: "nowrap", flexShrink: 0 };
const secondaryBtn: CSSProperties = { border: "1px solid var(--border)", background: "var(--bg)", color: "var(--text)", borderRadius: "var(--radius-md)", padding: "6px 10px", fontSize: 12, cursor: "pointer", whiteSpace: "nowrap", flexShrink: 0 };
const dangerBtn: CSSProperties = { border: "1px solid var(--border)", background: "var(--bg)", color: "var(--st-danger)", borderRadius: "var(--radius-md)", padding: "6px 12px", fontSize: 12, fontWeight: 600, cursor: "pointer", whiteSpace: "nowrap", flexShrink: 0 };
