import { describe, it, expect, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { TrashPanel } from "./TrashPanel";
import type { TrashItem } from "../domain/types";

const items: TrashItem[] = [
  { kind: "task", id: "1", title: "보관된 태스크", archivedAt: "2026-06-20T00:00:00Z" },
  { kind: "document", id: "2", title: "보관된 문서", archivedAt: "2026-06-19T00:00:00Z" },
];

function setup(list = items) {
  const h = { onRestore: vi.fn(), onPurge: vi.fn(), onClose: vi.fn() };
  render(<TrashPanel items={list} {...h} />);
  return h;
}

describe("TrashPanel", () => {
  it("보관 항목을 렌더", () => {
    setup();
    expect(screen.getByText("보관된 태스크")).toBeInTheDocument();
    expect(screen.getByText("보관된 문서")).toBeInTheDocument();
  });

  it("빈 휴지통 안내", () => {
    setup([]);
    expect(screen.getByText("보관된 항목이 없습니다.")).toBeInTheDocument();
  });

  it("복구 버튼 클릭 시 onRestore", async () => {
    const h = setup();
    await userEvent.click(screen.getAllByRole("button", { name: "복구" })[0]);
    expect(h.onRestore).toHaveBeenCalledWith(items[0]);
  });

  it("삭제 버튼 클릭 시 onPurge", async () => {
    const h = setup();
    await userEvent.click(screen.getByRole("button", { name: "보관된 문서 영구삭제" }));
    expect(h.onPurge).toHaveBeenCalledWith(items[1]);
  });
});
