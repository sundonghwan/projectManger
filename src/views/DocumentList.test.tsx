import { describe, it, expect, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { DocumentList } from "./DocumentList";
import type { Document } from "../domain/types";

const doc = (id: number, over: Partial<Document> = {}): Document => ({
  id,
  businessId: 1,
  projectId: null,
  title: `문서${id}`,
  sortOrder: id,
  createdAt: "2026-06-22T03:00:00Z",
  ...over,
});

function setup(over: Partial<Parameters<typeof DocumentList>[0]> = {}) {
  const h = {
    documents: [doc(1), doc(2)],
    error: null,
    onCreate: vi.fn(),
    onOpen: vi.fn(),
    onRename: vi.fn(),
    onArchive: vi.fn(),
    ...over,
  };
  render(<DocumentList {...h} />);
  return h;
}

describe("DocumentList", () => {
  it("문서를 행으로 렌더(제목·생성일)", () => {
    setup();
    const row = screen.getByTestId("doc-1");
    expect(row).toHaveTextContent("문서1");
    expect(row).toHaveTextContent("2026-06-22");
  });

  it("빈 목록 안내", () => {
    setup({ documents: [] });
    expect(screen.getByText(/문서가 없습니다/)).toBeInTheDocument();
  });

  it("새 문서 버튼 onCreate", async () => {
    const h = setup();
    await userEvent.click(screen.getByRole("button", { name: "새 문서" }));
    expect(h.onCreate).toHaveBeenCalled();
  });

  it("편집 버튼 onOpen", async () => {
    const h = setup();
    await userEvent.click(screen.getByRole("button", { name: "문서1 편집" }));
    expect(h.onOpen).toHaveBeenCalledWith(1);
  });

  it("제목 클릭 시 onOpen", async () => {
    const h = setup();
    await userEvent.click(screen.getByText("문서1"));
    expect(h.onOpen).toHaveBeenCalledWith(1);
  });

  it("삭제 버튼 onArchive", async () => {
    const h = setup();
    await userEvent.click(screen.getByRole("button", { name: "문서1 삭제" }));
    expect(h.onArchive).toHaveBeenCalledWith(1);
  });

  it("제목 더블클릭 후 Enter로 이름변경 → onRename", async () => {
    const h = setup();
    await userEvent.dblClick(screen.getByText("문서1"));
    const input = screen.getByLabelText("이름 변경");
    await userEvent.clear(input);
    await userEvent.type(input, "기획안{Enter}");
    expect(h.onRename).toHaveBeenCalledWith(1, "기획안");
  });
});
