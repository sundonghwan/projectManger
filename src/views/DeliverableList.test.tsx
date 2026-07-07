import { beforeEach, describe, it, expect, vi } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { DeliverableList } from "./DeliverableList";
import { DeliverablesView } from "./DeliverablesView";
import type { Deliverable } from "../domain/types";

const mocks = vi.hoisted(() => ({
  dragDropHandler: null as null | ((event: { payload: unknown }) => void),
  onDragDropEvent: vi.fn(),
  unlistenDragDrop: vi.fn(),
  openDialog: vi.fn(),
  upload: vi.fn(),
  setStatus: vi.fn(),
  rename: vi.fn(),
  move: vi.fn(),
  open: vi.fn(),
  archive: vi.fn(),
  reload: vi.fn(),
  uploading: false,
}));

vi.mock("@tauri-apps/api/webview", () => ({
  getCurrentWebview: () => ({
    onDragDropEvent: mocks.onDragDropEvent,
  }),
}));

vi.mock("@tauri-apps/plugin-dialog", () => ({
  open: mocks.openDialog,
}));

vi.mock("../hooks/useDeliverables", () => ({
  useDeliverables: () => ({
    deliverables: [],
    error: null,
    uploading: mocks.uploading,
    reload: mocks.reload,
    upload: mocks.upload,
    rename: mocks.rename,
    setStatus: mocks.setStatus,
    open: mocks.open,
    archive: mocks.archive,
    move: mocks.move,
  }),
}));

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
  beforeEach(() => {
    mocks.dragDropHandler = null;
    mocks.uploading = false;
    mocks.onDragDropEvent.mockReset();
    mocks.onDragDropEvent.mockImplementation(async (handler: (event: { payload: unknown }) => void) => {
      mocks.dragDropHandler = handler;
      return mocks.unlistenDragDrop;
    });
    mocks.unlistenDragDrop.mockReset();
    mocks.openDialog.mockReset();
    mocks.upload.mockReset();
    mocks.upload.mockResolvedValue(undefined);
    mocks.setStatus.mockReset();
    mocks.rename.mockReset();
    mocks.move.mockReset();
    mocks.open.mockReset();
    mocks.archive.mockReset();
    mocks.reload.mockReset();
  });

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

  it("dragActive일 때 드롭 안내 오버레이를 표시", () => {
    setup({ dragActive: true });
    expect(screen.getByText("여기에 파일을 놓아 업로드")).toBeInTheDocument();
  });
});

describe("DeliverablesView drag and drop", () => {
  beforeEach(() => {
    mocks.dragDropHandler = null;
    mocks.uploading = false;
    mocks.onDragDropEvent.mockReset();
    mocks.onDragDropEvent.mockImplementation(async (handler: (event: { payload: unknown }) => void) => {
      mocks.dragDropHandler = handler;
      return mocks.unlistenDragDrop;
    });
    mocks.unlistenDragDrop.mockReset();
    mocks.upload.mockReset();
    mocks.upload.mockResolvedValue(undefined);
  });

  it("Tauri drop paths를 현재 선택 폴더로 업로드", async () => {
    render(<DeliverablesView businessId="biz-1" projectId="project-1" selectedFolderId="folder-1" onChanged={vi.fn()} />);

    await waitFor(() => expect(mocks.onDragDropEvent).toHaveBeenCalled());
    mocks.dragDropHandler?.({
      payload: { type: "drop", paths: ["/tmp/a.pdf", "/tmp/b.pdf"], position: { x: 0, y: 0 } },
    });

    expect(mocks.upload).toHaveBeenCalledWith(["/tmp/a.pdf", "/tmp/b.pdf"], "folder-1");
  });

  it("업로드 중에는 Tauri 드래그 오버와 드롭을 무시", async () => {
    mocks.uploading = true;
    render(<DeliverablesView businessId="biz-1" projectId="project-1" selectedFolderId="folder-1" onChanged={vi.fn()} />);

    await waitFor(() => expect(mocks.onDragDropEvent).toHaveBeenCalled());
    mocks.dragDropHandler?.({
      payload: { type: "over", position: { x: 0, y: 0 } },
    });
    mocks.dragDropHandler?.({
      payload: { type: "drop", paths: ["/tmp/a.pdf"], position: { x: 0, y: 0 } },
    });

    expect(screen.queryByText("여기에 파일을 놓아 업로드")).not.toBeInTheDocument();
    expect(mocks.upload).not.toHaveBeenCalled();
  });
});
