import { describe, it, expect, vi } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { Kanban } from "./Kanban";
import { groupByStatus } from "../domain/kanban";
import type { Task } from "../domain/types";

const task = (o: Partial<Task> & Pick<Task, "id" | "status">): Task => ({
  businessId: "1",
  title: `태스크 ${o.id}`,
  priority: 2,
  sortOrder: o.sortOrder ?? 0,
  ...o,
});

function setup(tasks: Task[], over: Partial<Parameters<typeof Kanban>[0]> = {}) {
  const onMove = vi.fn();
  const onAddTask = vi.fn();
  render(
    <Kanban columns={groupByStatus(tasks)} onMove={onMove} onAddTask={onAddTask} {...over} />,
  );
  return { onMove, onAddTask };
}

describe("Kanban", () => {
  it("4개 컬럼과 카운트를 렌더한다", () => {
    setup([task({ id: "1", status: "todo" }), task({ id: "2", status: "doing" })]);
    expect(screen.getByText("Todo")).toBeInTheDocument();
    expect(screen.getByText("Doing")).toBeInTheDocument();
    expect(screen.getByText("Review")).toBeInTheDocument();
    expect(screen.getByText("Done")).toBeInTheDocument();
  });

  it("카드를 해당 컬럼에 표시", () => {
    setup([task({ id: "1", status: "todo" }), task({ id: "2", status: "done" })]);
    const todoCol = screen.getByTestId("col-todo");
    const doneCol = screen.getByTestId("col-done");
    expect(todoCol).toContainElement(screen.getByTestId("card-1"));
    expect(doneCol).toContainElement(screen.getByTestId("card-2"));
  });

  it("컬럼의 + 버튼은 해당 상태로 onAddTask 호출", async () => {
    const { onAddTask } = setup([]);
    await userEvent.click(screen.getByRole("button", { name: "Doing에 태스크 추가" }));
    expect(onAddTask).toHaveBeenCalledWith("doing");
  });

  it("카드를 다른 컬럼에 드롭하면 onMove를 새 상태로 호출", () => {
    const { onMove } = setup([task({ id: "1", status: "todo", sortOrder: 1 })]);
    const card = screen.getByTestId("card-1");
    const doneCol = screen.getByTestId("col-done");
    const dt = { getData: () => "1", setData: vi.fn() };
    fireEvent.dragStart(card, { dataTransfer: dt });
    fireEvent.drop(doneCol, { dataTransfer: dt });
    expect(onMove).toHaveBeenCalledWith("1", "done", expect.any(Number));
  });
});
