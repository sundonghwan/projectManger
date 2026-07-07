import { describe, it, expect, vi } from "vitest";
import { fireEvent, render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { DeliverableList } from "./DeliverableList";
import type { Deliverable } from "../domain/types";

const deliv = (id: string, over: Partial<Deliverable> = {}): Deliverable => ({
  id,
  businessId: "1",
  title: `보고서${id}.pdf`,
  kind: "file",
  status: "draft",
  currentVersion: 1,
  sortOrder: Number(id),
  filePath: `/data/deliverables/${id}/보고서${id}.pdf`,
  fileSize: 1536,
  originalName: `보고서${id}.pdf`,
  createdAt: "2026-06-22T03:00:00Z",
  ...over,
});

function setup(over: Partial<Parameters<typeof DeliverableList>[0]> = {}) {
  const h = {
    deliverables: [deliv("1"), deliv("2", { status: "done" })],
    error: null,
    onUpload: vi.fn(),
    onSetStatus: vi.fn(),
    onRename: vi.fn(),
    onOpen: vi.fn(),
    onArchive: vi.fn(),
    ...over,
  };
  render(<DeliverableList {...h} />);
  return h;
}

describe("DeliverableList", () => {
  it("업로드된 파일을 행으로 렌더(파일명·크기·업로드일)", () => {
    setup();
    const row = screen.getByTestId("deliv-1");
    expect(row).toHaveTextContent("보고서1.pdf");
    expect(row).toHaveTextContent("1.5 KB");
    expect(row).toHaveTextContent("2026-06-22");
    expect(screen.getByTestId("deliv-2")).toHaveTextContent("보고서2.pdf");
  });

  it("빈 목록 안내", () => {
    setup({ deliverables: [] });
    expect(screen.getByText(/업로드된 산출물이 없습니다/)).toBeInTheDocument();
  });

  it("파일 업로드 버튼 클릭 시 onUpload", async () => {
    const h = setup();
    await userEvent.click(screen.getByRole("button", { name: "파일 업로드" }));
    expect(h.onUpload).toHaveBeenCalled();
  });

  it("상태 변경 시 onSetStatus", async () => {
    const h = setup();
    await userEvent.selectOptions(screen.getByLabelText("보고서1.pdf 상태"), "review");
    expect(h.onSetStatus).toHaveBeenCalledWith("1", "review");
  });

  it("열기 버튼 onOpen", async () => {
    const h = setup();
    await userEvent.click(screen.getByRole("button", { name: "보고서1.pdf 열기" }));
    expect(h.onOpen).toHaveBeenCalledWith(expect.objectContaining({ id: "1" }));
  });

  it("삭제 버튼 onArchive", async () => {
    const h = setup();
    await userEvent.click(screen.getByRole("button", { name: "보고서1.pdf 삭제" }));
    expect(h.onArchive).toHaveBeenCalledWith("1");
  });

  it("파일명 더블클릭 후 Enter로 이름변경 → onRename", async () => {
    const h = setup();
    await userEvent.dblClick(screen.getByText("보고서1.pdf"));
    const input = screen.getByLabelText("이름 변경");
    await userEvent.clear(input);
    await userEvent.type(input, "최종본{Enter}");
    expect(h.onRename).toHaveBeenCalledWith("1", "최종본");
  });

  it("파일 경로 없으면 열기 비활성화", () => {
    setup({ deliverables: [deliv("1", { filePath: null })] });
    expect(screen.getByRole("button", { name: "보고서1.pdf 열기" })).toBeDisabled();
  });

  it("드롭한 파일 경로를 onDropFiles에 전달", () => {
    const onDropFiles = vi.fn();
    setup({ onDropFiles });
    const first = new File(["a"], "a.pdf", { type: "application/pdf" }) as File & { path?: string };
    first.path = "/tmp/a.pdf";
    const missingPath = new File(["b"], "b.pdf", { type: "application/pdf" });
    const second = new File(["c"], "c.pdf", { type: "application/pdf" }) as File & { path?: string };
    second.path = "/tmp/c.pdf";

    fireEvent.drop(screen.getByTestId("deliverable-drop-zone"), {
      dataTransfer: { files: [first, missingPath, second] },
    });

    expect(onDropFiles).toHaveBeenCalledWith(["/tmp/a.pdf", "/tmp/c.pdf"]);
  });

  it("드래그 오버 중 드롭 안내 오버레이를 표시", () => {
    setup({ onDropFiles: vi.fn() });

    fireEvent.dragOver(screen.getByTestId("deliverable-drop-zone"));

    expect(screen.getByText("여기에 파일을 놓아 업로드")).toBeInTheDocument();
  });
});
