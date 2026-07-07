import { describe, it, expect, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { TaskList } from "./TaskList";
import type { Task } from "../domain/types";

const task = (o: Partial<Task> & Pick<Task, "id" | "status">): Task => ({
  businessId: "1",
  title: `태스크 ${o.id}`,
  priority: 2,
  sortOrder: 0,
  ...o,
});

describe("TaskList", () => {
  it("태스크 행과 상태를 렌더", () => {
    render(
      <TaskList
        tasks={[task({ id: "1", status: "doing", startDate: "2026-06-20", dueDate: "2026-07-01" })]}
        onToggleDone={vi.fn()}
      />,
    );
    expect(screen.getByText("태스크 1")).toBeInTheDocument();
    expect(screen.getByText("Doing")).toBeInTheDocument();
    expect(screen.getByText("2026-06-20")).toBeInTheDocument();
    expect(screen.getByText("2026-07-01")).toBeInTheDocument();
  });

  it("빈 목록 안내", () => {
    render(<TaskList tasks={[]} onToggleDone={vi.fn()} />);
    expect(screen.getByText("태스크가 없습니다.")).toBeInTheDocument();
  });

  it("완료된 태스크는 체크 상태", () => {
    render(<TaskList tasks={[task({ id: "2", status: "done" })]} onToggleDone={vi.fn()} />);
    expect(screen.getByRole("checkbox")).toHaveAttribute("aria-checked", "true");
  });

  it("체크박스 클릭 시 onToggleDone 호출", async () => {
    const onToggleDone = vi.fn();
    const t = task({ id: "3", status: "todo" });
    render(<TaskList tasks={[t]} onToggleDone={onToggleDone} />);
    await userEvent.click(screen.getByRole("checkbox"));
    expect(onToggleDone).toHaveBeenCalledWith(t);
  });
});
