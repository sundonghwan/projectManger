import { describe, expect, it } from "vitest";
import { buildInitialEditorSource, parseEditorBody, prepareEditorSavePayload } from "./documentBody";

describe("documentBody", () => {
  it("prefers valid BlockNote JSON over Markdown", () => {
    const parsed = buildInitialEditorSource({
      body: "# 공유용",
      editorBody: '[{"type":"paragraph","content":"원본"}]',
      editorBodyFormat: "blocknote-json",
    });

    expect(parsed.kind).toBe("blocks");
    expect(parsed.warning).toBeNull();
  });

  it("falls back to Markdown when BlockNote JSON is broken", () => {
    const parsed = buildInitialEditorSource({
      body: "# 공유용",
      editorBody: "{broken",
      editorBodyFormat: "blocknote-json",
    });

    expect(parsed.kind).toBe("markdown");
    expect(parsed.markdown).toBe("# 공유용");
    expect(parsed.warning).toContain("블록 문서");
  });

  it("prepares Markdown as the external sharing body", () => {
    const payload = prepareEditorSavePayload({
      markdown: "# 공유용\n\n![이미지](asset://image.png)",
      blocks: [{ type: "paragraph", content: "공유용" }],
      collaborationState: null,
    });

    expect(payload.body).toContain("# 공유용");
    expect(payload.editorBodyFormat).toBe("blocknote-json");
    expect(JSON.parse(payload.editorBody)).toEqual([{ type: "paragraph", content: "공유용" }]);
  });

  it("rejects non-array editor JSON", () => {
    expect(parseEditorBody("{}")).toBeNull();
  });
});
