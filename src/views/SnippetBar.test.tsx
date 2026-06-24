import { describe, it, expect, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { SnippetBar } from "./SnippetBar";
import type { CommandSnippet } from "../domain/types";

const snippets: CommandSnippet[] = [
  { id: "1", serverId: "1", name: "배포", command: "./deploy.sh", sortOrder: 1 },
];

function setup(list = snippets) {
  const h = { onRun: vi.fn(), onCreate: vi.fn(), onDelete: vi.fn() };
  render(<SnippetBar snippets={list} {...h} />);
  return h;
}

describe("SnippetBar", () => {
  it("스니펫 칩을 렌더하고 클릭 시 onRun(명령)", async () => {
    const h = setup();
    await userEvent.click(screen.getByRole("button", { name: "배포" }));
    expect(h.onRun).toHaveBeenCalledWith("./deploy.sh");
  });

  it("삭제 버튼은 onDelete", async () => {
    const h = setup();
    await userEvent.click(screen.getByRole("button", { name: "배포 삭제" }));
    expect(h.onDelete).toHaveBeenCalledWith("1");
  });

  it("스니펫 추가 폼으로 onCreate", async () => {
    const h = setup([]);
    await userEvent.click(screen.getByRole("button", { name: "스니펫 추가" }));
    await userEvent.type(screen.getByLabelText("스니펫 이름"), "상태");
    await userEvent.type(screen.getByLabelText("스니펫 명령"), "uptime");
    await userEvent.click(screen.getByRole("button", { name: "저장" }));
    expect(h.onCreate).toHaveBeenCalledWith("상태", "uptime");
  });
});
