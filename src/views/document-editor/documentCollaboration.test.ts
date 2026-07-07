import { describe, expect, it } from "vitest";
import { createDocumentCollaboration } from "./documentCollaboration";

describe("documentCollaboration", () => {
  it("creates a stable room key from the document id", () => {
    const collaboration = createDocumentCollaboration({ documentId: "doc-1", providerMode: "local" });

    expect(collaboration.room).toBe("work-vault:document:doc-1");
    expect(collaboration.status).toBe("local");
    collaboration.destroy();
  });

  it("encodes and restores snapshots as base64 strings", () => {
    const collaboration = createDocumentCollaboration({ documentId: "doc-1", providerMode: "local" });
    const snapshot = collaboration.encodeState();
    const restored = createDocumentCollaboration({ documentId: "doc-1", initialState: snapshot, providerMode: "local" });

    expect(snapshot.length).toBeGreaterThan(0);
    expect(restored.room).toBe(collaboration.room);
    collaboration.destroy();
    restored.destroy();
  });

  it("creates a provider-backed collaboration object when enabled", () => {
    const collaboration = createDocumentCollaboration({ documentId: "doc-1", providerMode: "webrtc" });

    expect(collaboration.room).toBe("work-vault:document:doc-1");
    expect(collaboration.provider).not.toBeNull();
    collaboration.destroy();
  });

  it("falls back to a disconnected local document when the WebRTC provider is unavailable", () => {
    const collaboration = createDocumentCollaboration({
      documentId: "doc-1",
      providerMode: "webrtc",
      createProvider: () => {
        throw new Error("WebRTC unavailable");
      },
    });

    expect(collaboration.status).toBe("disconnected");
    expect(collaboration.provider).toBeNull();
    expect(collaboration.fragment).toBeTruthy();
    collaboration.destroy();
  });
});
