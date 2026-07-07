import * as Y from "yjs";
import { WebrtcProvider } from "y-webrtc";

export type CollaborationStatus = "local" | "connected" | "disconnected";
export type CollaborationProviderMode = "local" | "webrtc";

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
}) {
  const doc = new Y.Doc();
  const room = `work-vault:document:${input.documentId}`;
  if (input.initialState) {
    Y.applyUpdate(doc, base64ToBytes(input.initialState));
  }
  const provider = input.providerMode === "webrtc" ? new WebrtcProvider(room, doc) : null;

  return {
    doc,
    provider,
    room,
    status: provider ? ("connected" as CollaborationStatus) : ("local" as CollaborationStatus),
    fragment: doc.getXmlFragment("document-store"),
    encodeState: () => bytesToBase64(Y.encodeStateAsUpdate(doc)),
    destroy: () => {
      provider?.destroy();
      doc.destroy();
    },
  };
}
