import { describe, it, expect } from "vitest";
import { render, screen } from "@testing-library/react";
import { TimelineView } from "./TimelineView";
import type { Task } from "../domain/types";

const task = (id: number, dueDate?: string): Task => ({
  id,
  businessId: 1,
  title: `태스크 ${id}`,
  status: "todo",
  priority: 2,
  sortOrder: 0,
  dueDate,
});

describe("TimelineView", () => {
  it("마감 있는 태스크를 날짜와 함께 표시", () => {
    render(<TimelineView tasks={[task(1, "2026-07-01"), task(2, "2026-07-10")]} />);
    expect(screen.getByTestId("tl-1")).toHaveTextContent("태스크 1");
    expect(screen.getByTestId("tl-1")).toHaveTextContent("2026-07-01");
    expect(screen.getByTestId("tl-2")).toHaveTextContent("태스크 2");
  });

  it("마감 없으면 안내", () => {
    render(<TimelineView tasks={[task(1)]} />);
    expect(screen.getByText(/마감일이 있는 태스크가 없습니다/)).toBeInTheDocument();
  });
});
