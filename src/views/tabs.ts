import type { ServerConnection } from "../domain/types";

export type TabKind = "ssh" | "local" | "sftp";
export interface Tab {
  key: string;
  kind: TabKind;
  server: ServerConnection;
  title: string;
}
export interface TabsState {
  tabs: Tab[];
  activeKey: string | null;
}
export const EMPTY_TABS: TabsState = { tabs: [], activeKey: null };

function focusOrAdd(state: TabsState, tab: Tab): TabsState {
  if (state.tabs.some((t) => t.key === tab.key)) {
    return { ...state, activeKey: tab.key };
  }
  return { tabs: [...state.tabs, tab], activeKey: tab.key };
}

export function openSsh(state: TabsState, server: ServerConnection): TabsState {
  return focusOrAdd(state, { key: server.id, kind: "ssh", server, title: server.name });
}

export function openSftp(state: TabsState, server: ServerConnection): TabsState {
  return focusOrAdd(state, {
    key: `sftp:${server.id}`,
    kind: "sftp",
    server,
    title: `SFTP: ${server.name}`,
  });
}

export function openLocal(state: TabsState, server: ServerConnection): TabsState {
  // server.id 는 호출자가 이미 `local:<uuid>` 로 채운다 → 항상 새 탭.
  return {
    tabs: [...state.tabs, { key: server.id, kind: "local", server, title: server.name }],
    activeKey: server.id,
  };
}

export function closeTab(state: TabsState, key: string): TabsState {
  const idx = state.tabs.findIndex((t) => t.key === key);
  if (idx < 0) return state;
  const tabs = state.tabs.filter((t) => t.key !== key);
  let activeKey = state.activeKey;
  if (state.activeKey === key) {
    if (tabs.length === 0) activeKey = null;
    else activeKey = (tabs[idx - 1] ?? tabs[idx] ?? tabs[0]).key; // 이전 이웃 우선
  }
  return { tabs, activeKey };
}
