import { describe, it, expect } from "vitest";
import { EMPTY_TABS, openSsh, openSftp, openLocal, closeTab } from "./tabs";
import type { ServerConnection } from "../domain/types";

const srv = (id: string): ServerConnection => ({
  id, businessId: "b", projectId: null, name: `srv-${id}`, host: "h", port: 22,
  username: "u", authType: "key", aiBridge: false,
});

describe("tabs", () => {
  it("openSsh adds a tab and activates it", () => {
    const s = openSsh(EMPTY_TABS, srv("1"));
    expect(s.tabs).toHaveLength(1);
    expect(s.tabs[0].key).toBe("1");
    expect(s.tabs[0].kind).toBe("ssh");
    expect(s.activeKey).toBe("1");
  });

  it("openSsh on an already-open server focuses (no duplicate)", () => {
    let s = openSsh(EMPTY_TABS, srv("1"));
    s = openSsh(s, srv("2"));
    s = openSsh(s, srv("1")); // reopen 1
    expect(s.tabs).toHaveLength(2);
    expect(s.activeKey).toBe("1");
  });

  it("openLocal always adds (unique ids)", () => {
    let s = openLocal(EMPTY_TABS, srv("local:a"));
    s = openLocal(s, srv("local:b"));
    expect(s.tabs).toHaveLength(2);
    expect(s.activeKey).toBe("local:b");
  });

  it("openSftp keys by sftp:<id> and focuses on reopen", () => {
    let s = openSftp(EMPTY_TABS, srv("1"));
    expect(s.tabs[0].key).toBe("sftp:1");
    expect(s.tabs[0].kind).toBe("sftp");
    const before = s.tabs.length;
    s = openSftp(s, srv("1"));
    expect(s.tabs).toHaveLength(before);
  });

  it("closeTab removes and activates the previous neighbor", () => {
    let s = openSsh(EMPTY_TABS, srv("1"));
    s = openSsh(s, srv("2"));
    s = openSsh(s, srv("3")); // active=3, order [1,2,3]
    s = closeTab(s, "3");
    expect(s.tabs.map((t) => t.key)).toEqual(["1", "2"]);
    expect(s.activeKey).toBe("2"); // previous neighbor
  });

  it("closing the last tab yields null active", () => {
    let s = openSsh(EMPTY_TABS, srv("1"));
    s = closeTab(s, "1");
    expect(s.tabs).toHaveLength(0);
    expect(s.activeKey).toBeNull();
  });

  it("closing a non-active tab keeps active unchanged", () => {
    let s = openSsh(EMPTY_TABS, srv("1"));
    s = openSsh(s, srv("2")); // active=2
    s = closeTab(s, "1");
    expect(s.activeKey).toBe("2");
  });
});
