import * as Y from "yjs";
import { WebrtcProvider } from "y-webrtc";

export type CollaborationStatus = "local" | "connected" | "disconnected";
export type CollaborationProviderMode = "local" | "webrtc";
type CollaborationProvider = InstanceType<typeof WebrtcProvider>;

function bytesToBase64(bytes: Uint8Array) {
  let binary = "";
  bytes.forEach((byte) => {
    binary += String.fromCharCode(byte);
  });
  return btoa(binary);
}

function base64ToBytes(value: string) {
  const binary = atob(value);
  return Uint8Array.from(binary, (char) => char.charCodeAt(0));
}

export function createDocumentCollaboration(input: {
  documentId: string;
  initialState?: string | null;
  providerMode?: CollaborationProviderMode;
  createProvider?: (room: string, doc: Y.Doc) => CollaborationProvider;
}) {
  const doc = new Y.Doc();
  const room = `work-vault:document:${input.documentId}`;
  if (input.initialState) {
    Y.applyUpdate(doc, base64ToBytes(input.initialState));
  }
  let provider: CollaborationProvider | null = null;
  let status: CollaborationStatus = "local";
  if (input.providerMode === "webrtc") {
    try {
      provider = input.createProvider ? input.createProvider(room, doc) : new WebrtcProvider(room, doc);
      status = "connected";
    } catch {
      status = "disconnected";
    }
  }

  return {
    doc,
    provider,
    room,
    status,
    fragment: doc.getXmlFragment("document-store"),
    encodeState: () => bytesToBase64(Y.encodeStateAsUpdate(doc)),
    destroy: () => {
      provider?.destroy();
      doc.destroy();
    },
  };
}
