import { describe, it, expect, vi, beforeEach } from "vitest";
import { fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import type { Document } from "../domain/types";

const documentApi = vi.hoisted(() => ({
  get: vi.fn(),
  setBody: vi.fn(),
  showExportFolder: vi.fn(),
}));

vi.mock("../api/client", () => ({
  api: {
    document: documentApi,
  },
}));

import { DocEditor } from "./DocEditor";

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

describe("DocEditor (Markdown editor)", () => {
  beforeEach(() => {
    documentApi.get.mockReset();
    documentApi.get.mockResolvedValue({ ...doc, body: "# 안녕\n본문" });
    documentApi.setBody.mockReset();
    documentApi.setBody.mockResolvedValue(undefined);
    documentApi.showExportFolder.mockReset();
    documentApi.showExportFolder.mockResolvedValue(undefined);
  });

  it("마운트 시 최신 Markdown 본문을 textarea와 렌더링 영역에 표시", async () => {
    render(<DocEditor document={doc} />);

    const editor = (await screen.findByLabelText("Markdown 문서 본문")) as HTMLTextAreaElement;
    expect(editor.value).toBe("# 안녕\n본문");
    expect(screen.getByRole("heading", { name: "안녕", level: 1 })).toBeInTheDocument();
    expect(screen.getByText("본문")).toBeInTheDocument();
  });

  it("입력 즉시 Markdown preview를 갱신하고 blur 시 body만 저장", async () => {
    render(<DocEditor document={doc} />);
    const editor = (await screen.findByLabelText("Markdown 문서 본문")) as HTMLTextAreaElement;

    await userEvent.clear(editor);
    await userEvent.type(editor, "## 변경\n- 항목");
    expect(screen.getByRole("heading", { name: "변경", level: 2 })).toBeInTheDocument();
    expect(screen.getByText("항목")).toBeInTheDocument();

    fireEvent.blur(editor);

    await waitFor(() => expect(documentApi.setBody).toHaveBeenCalledWith("1", "## 변경\n- 항목"));
    expect(await screen.findByText("저장됨")).toBeInTheDocument();
  });

  it("Markdown HTML은 sanitize 후 렌더링한다", async () => {
    documentApi.get.mockResolvedValueOnce({
      ...doc,
      body: '<script>alert("x")</script><strong>굵게</strong>',
    });

    render(<DocEditor document={doc} />);

    await screen.findByLabelText("Markdown 문서 본문");
    const preview = screen.getByLabelText("Markdown 렌더링 결과");
    expect(within(preview).queryByText(/alert/)).not.toBeInTheDocument();
    expect(within(preview).getByText("굵게").tagName).toBe("STRONG");
  });

  it("폴더 열기는 문서 export 폴더 command를 호출한다", async () => {
    render(<DocEditor document={doc} />);
    await screen.findByLabelText("Markdown 문서 본문");

    await userEvent.click(screen.getByRole("button", { name: "기획안 폴더 열기" }));

    expect(documentApi.showExportFolder).toHaveBeenCalledWith("1");
  });
});
