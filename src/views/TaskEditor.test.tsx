import { describe, it, expect, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { TaskEditor } from "./TaskEditor";
import type { Label, Task } from "../domain/types";

const task: Task = {
  id: 1,
  businessId: 1,
  title: "로그인 API",
  status: "todo",
  priority: 3,
  dueDate: "2026-07-01",
  sortOrder: 1,
};
const labels: Label[] = [{ id: 5, name: "백엔드", color: "#3b82f6" }];

function setup() {
  const h = {
    onSave: vi.fn(),
    onAddLabel: vi.fn(),
    onRemoveLabel: vi.fn(),
    onArchive: vi.fn(),
    onClose: vi.fn(),
  };
  render(<TaskEditor task={task} labels={labels} {...h} />);
  return h;
}

describe("TaskEditor", () => {
  it("기존 값을 채워 보여준다", () => {
    setup();
    expect(screen.getByLabelText("제목")).toHaveValue("로그인 API");
    expect(screen.getByLabelText("우선순위")).toHaveValue("3");
    expect(screen.getByLabelText("마감")).toHaveValue("2026-07-01");
    expect(screen.getByText("백엔드")).toBeInTheDocument();
  });

  it("제목 수정 후 저장하면 patch와 함께 onSave", async () => {
    const h = setup();
    const title = screen.getByLabelText("제목");
    await userEvent.clear(title);
    await userEvent.type(title, "수정됨");
    await userEvent.click(screen.getByRole("button", { name: "저장" }));
    expect(h.onSave).toHaveBeenCalledWith(
      expect.objectContaining({ title: "수정됨", priority: 3, dueDate: "2026-07-01" }),
    );
  });

  it("새 라벨 추가 시 onAddLabel", async () => {
    const h = setup();
    await userEvent.type(screen.getByLabelText("라벨 이름"), "긴급");
    await userEvent.click(screen.getByRole("button", { name: "추가" }));
    expect(h.onAddLabel).toHaveBeenCalledWith("긴급", expect.any(String));
  });

  it("라벨 × 클릭 시 onRemoveLabel", async () => {
    const h = setup();
    await userEvent.click(screen.getByRole("button", { name: "백엔드 라벨 제거" }));
    expect(h.onRemoveLabel).toHaveBeenCalledWith(labels[0]);
  });

  it("보관 버튼은 onArchive", async () => {
    const h = setup();
    await userEvent.click(screen.getByRole("button", { name: "보관" }));
    expect(h.onArchive).toHaveBeenCalled();
  });

  it("오버레이/닫기 클릭 시 onClose", async () => {
    const h = setup();
    await userEvent.click(screen.getByLabelText("닫기"));
    expect(h.onClose).toHaveBeenCalled();
  });
});
