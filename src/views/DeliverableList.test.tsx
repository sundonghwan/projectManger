import { describe, it, expect, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { DeliverableList } from "./DeliverableList";
import type { Deliverable, DeliverableVersion } from "../domain/types";

const deliv = (id: number, over: Partial<Deliverable> = {}): Deliverable => ({
  id,
  businessId: 1,
  title: `산출물 ${id}`,
  kind: "file",
  status: "draft",
  currentVersion: 1,
  sortOrder: id,
  ...over,
});

const versions: DeliverableVersion[] = [
  { id: 1, deliverableId: 1, version: 2, note: "수정", createdAt: "2026-06-20T00:00:00Z" },
  { id: 2, deliverableId: 1, version: 1, note: "최초 생성", createdAt: "2026-06-19T00:00:00Z" },
];

function setup(over: Partial<Parameters<typeof DeliverableList>[0]> = {}) {
  const h = {
    deliverables: [deliv(1, { currentVersion: 2 }), deliv(2, { status: "done" })],
    selectedId: 1,
    versions,
    onSelect: vi.fn(),
    onCreate: vi.fn(),
    onSetStatus: vi.fn(),
    onAddVersion: vi.fn(),
    onArchive: vi.fn(),
    ...over,
  };
  render(<DeliverableList {...h} />);
  return h;
}

describe("DeliverableList", () => {
  it("산출물과 버전 히스토리를 렌더", () => {
    setup();
    expect(screen.getByTestId("deliv-1")).toHaveTextContent("산출물 1");
    expect(screen.getByTestId("deliv-2")).toHaveTextContent("산출물 2");
    expect(screen.getByTestId("ver-2")).toBeInTheDocument();
    expect(screen.getByTestId("ver-1")).toBeInTheDocument();
  });

  it("빈 목록 안내", () => {
    setup({ deliverables: [], selectedId: null });
    expect(screen.getByText(/산출물이 없습니다/)).toBeInTheDocument();
  });

  it("+ 산출물 클릭 시 onCreate", async () => {
    const h = setup();
    await userEvent.click(screen.getByRole("button", { name: "+ 산출물" }));
    expect(h.onCreate).toHaveBeenCalled();
  });

  it("상태 변경 시 onSetStatus", async () => {
    const h = setup();
    await userEvent.selectOptions(screen.getByLabelText("산출물 1 상태"), "review");
    expect(h.onSetStatus).toHaveBeenCalledWith(1, "review");
  });

  it("새 버전 버튼 onAddVersion", async () => {
    const h = setup();
    await userEvent.click(screen.getByRole("button", { name: "새 버전" }));
    expect(h.onAddVersion).toHaveBeenCalledWith(1);
  });

  it("행 클릭 시 onSelect", async () => {
    const h = setup();
    await userEvent.click(screen.getByTestId("deliv-2"));
    expect(h.onSelect).toHaveBeenCalledWith(2);
  });
});
