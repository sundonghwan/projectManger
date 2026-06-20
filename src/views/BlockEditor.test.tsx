import { describe, it, expect, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { BlockEditor } from "./BlockEditor";
import { stringify } from "../domain/blockContent";
import type { Block, BlockType } from "../domain/types";

const block = (id: number, type: BlockType, text = "", checked = false): Block => ({
  id,
  documentId: 1,
  type,
  content: stringify({ text, checked }),
  sortOrder: id,
});

function setup(blocks: Block[]) {
  const handlers = {
    onChangeText: vi.fn(),
    onToggleCheck: vi.fn(),
    onAddBlock: vi.fn(),
    onDelete: vi.fn(),
  };
  render(<BlockEditor blocks={blocks} {...handlers} />);
  return handlers;
}

describe("BlockEditor", () => {
  it("빈 문서 안내", () => {
    setup([]);
    expect(screen.getByText(/빈 문서입니다/)).toBeInTheDocument();
  });

  it("블록 텍스트를 입력값으로 표시", () => {
    setup([block(1, "heading", "개요")]);
    expect(screen.getByDisplayValue("개요")).toBeInTheDocument();
  });

  it("텍스트 입력 시 onChangeText 호출", async () => {
    const h = setup([block(1, "paragraph", "")]);
    await userEvent.type(screen.getByLabelText("블록 텍스트"), "A");
    expect(h.onChangeText).toHaveBeenCalledWith(expect.objectContaining({ id: 1 }), "A");
  });

  it("체크리스트 체크박스 클릭 시 onToggleCheck", async () => {
    const h = setup([block(2, "checklist", "할일", false)]);
    await userEvent.click(screen.getByRole("checkbox"));
    expect(h.onToggleCheck).toHaveBeenCalledWith(expect.objectContaining({ id: 2 }));
  });

  it("블록 추가 버튼이 타입별로 onAddBlock 호출", async () => {
    const h = setup([]);
    await userEvent.click(screen.getByRole("button", { name: "+ 제목" }));
    expect(h.onAddBlock).toHaveBeenCalledWith("heading");
  });

  it("삭제 버튼이 onDelete 호출", async () => {
    const h = setup([block(3, "paragraph", "x")]);
    await userEvent.click(screen.getByRole("button", { name: "블록 삭제" }));
    expect(h.onDelete).toHaveBeenCalledWith(expect.objectContaining({ id: 3 }));
  });

  it("divider 블록은 텍스트 입력이 없다", () => {
    setup([block(4, "divider")]);
    expect(screen.queryByLabelText("블록 텍스트")).not.toBeInTheDocument();
  });
});
