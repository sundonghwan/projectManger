import type { DocumentEditorBodyInput } from "../../api/client";

export type EditorSource =
  | { kind: "blocks"; blocks: unknown[]; markdown: string; warning: null }
  | { kind: "markdown"; blocks: null; markdown: string; warning: string | null };

export function parseEditorBody(editorBody?: string | null): unknown[] | null {
  if (!editorBody) return null;
  try {
    const parsed = JSON.parse(editorBody);
    return Array.isArray(parsed) ? parsed : null;
  } catch {
    return null;
  }
}

export function buildInitialEditorSource(document: {
  body?: string | null;
  editorBody?: string | null;
  editorBodyFormat?: string | null;
}): EditorSource {
  if (document.editorBodyFormat === "blocknote-json" && document.editorBody) {
    const blocks = parseEditorBody(document.editorBody);
    if (blocks) {
      return { kind: "blocks", blocks, markdown: document.body ?? "", warning: null };
    }
    return {
      kind: "markdown",
      blocks: null,
      markdown: document.body ?? "",
      warning: "블록 문서를 읽지 못해 Markdown 본문으로 열었습니다.",
    };
  }

  return { kind: "markdown", blocks: null, markdown: document.body ?? "", warning: null };
}

export function prepareEditorSavePayload(input: {
  markdown: string;
  blocks: unknown[];
  collaborationState?: string | null;
}): DocumentEditorBodyInput {
  return {
    body: input.markdown,
    editorBody: JSON.stringify(input.blocks),
    editorBodyFormat: "blocknote-json",
    collaborationState: input.collaborationState ?? null,
  };
}
