import { describe, it, expect, vi } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import type { Document } from "../domain/types";

vi.mock("../api/client", () => ({
  api: {
    document: {
      get: vi.fn().mockResolvedValue({ body: "# 안녕\n본문" }),
      setEditorBody: vi.fn().mockResolvedValue(undefined),
    },
  },
}));

vi.mock("./document-editor/BlockDocumentEditor", () => ({
  BlockDocumentEditor: ({
    initialMarkdown,
    onChange,
    onBlur,
  }: {
    initialMarkdown: string;
    onChange: (payload: { markdown: string; blocks: unknown[]; collaborationState?: string | null }) => void;
    onBlur: () => void;
  }) => {
    if (initialMarkdown === "# crash") {
      throw new Error("editor boot failed");
    }
    return (
      <textarea
        aria-label="라이브 문서 본문"
        defaultValue={initialMarkdown}
        onChange={(event) =>
          onChange({
            markdown: event.currentTarget.value,
            blocks: [{ type: "paragraph", content: event.currentTarget.value }],
            collaborationState: null,
          })
        }
        onBlur={onBlur}
      />
    );
  },
}));

import { DocEditor } from "./DocEditor";
import { api } from "../api/client";

const doc: Document = {
  id: "1",
  businessId: "1",
  projectId: null,
  title: "기획안",
  icon: null,
  body: "# 안녕",
  sortOrder: 1,
  archivedAt: null,
  createdAt: "2026-06-22T00:00:00Z",
};

describe("DocEditor (live block editor)", () => {
  it("마운트 시 최신 본문을 라이브 에디터에 전달", async () => {
    render(<DocEditor document={doc} />);
    const editor = (await screen.findByLabelText("라이브 문서 본문")) as HTMLTextAreaElement;
    expect(editor.value).toBe("# 안녕\n본문");
  });

  it("입력 후 blur 시 Markdown과 BlockNote JSON을 함께 저장", async () => {
    render(<DocEditor document={doc} />);
    const editor = (await screen.findByLabelText("라이브 문서 본문")) as HTMLTextAreaElement;
    expect(editor.value).toBe("# 안녕\n본문");

    await userEvent.type(editor, " 끝");
    editor.blur();

    await waitFor(() =>
      expect(api.document.setEditorBody).toHaveBeenCalledWith("1", {
        body: "# 안녕\n본문 끝",
        editorBody: JSON.stringify([{ type: "paragraph", content: "# 안녕\n본문 끝" }]),
        editorBodyFormat: "blocknote-json",
        collaborationState: null,
      }),
    );
    expect(await screen.findByText("저장됨")).toBeInTheDocument();
  });

  it("깨진 editorBody는 Markdown 본문으로 열고 경고를 표시", async () => {
    vi.mocked(api.document.get).mockResolvedValueOnce({
      body: "# 복구",
      editorBody: "{broken",
      editorBodyFormat: "blocknote-json",
    } as never);

    render(<DocEditor document={{ ...doc, body: "" }} />);

    const editor = (await screen.findByLabelText("라이브 문서 본문")) as HTMLTextAreaElement;
    expect(editor.value).toBe("# 복구");
    expect(screen.getByText(/블록 문서를 읽지 못해/)).toBeInTheDocument();
  });

  it("에디터 렌더 오류가 앱 흰 화면으로 번지지 않도록 오류 메시지를 표시", async () => {
    const consoleError = vi.spyOn(console, "error").mockImplementation(() => {});
    vi.mocked(api.document.get).mockResolvedValueOnce({ body: "# crash" } as never);

    render(<DocEditor document={{ ...doc, body: "" }} />);

    expect(await screen.findByText(/문서 편집기를 열지 못했습니다/)).toBeInTheDocument();
    expect(screen.getByText(/editor boot failed/)).toBeInTheDocument();
    consoleError.mockRestore();
  });
});
