import { describe, it, expect } from "vitest";
import { render, screen } from "@testing-library/react";
import { Dashboard } from "./Dashboard";
import { dashboardStats } from "../domain/dashboard";
import type { Business, Task } from "../domain/types";

const business: Business = {
  id: 1,
  name: "SI사업 A",
  type: "si",
  status: "active",
  sortOrder: 1,
};

const task = (o: Partial<Task> & Pick<Task, "id" | "status">): Task => ({
  businessId: 1,
  title: `t${o.id}`,
  priority: 2,
  sortOrder: 0,
  ...o,
});

describe("Dashboard", () => {
  it("사업명과 유형 라벨을 표시", () => {
    render(<Dashboard business={business} stats={dashboardStats([])} />);
    expect(screen.getByText("SI사업 A")).toBeInTheDocument();
    expect(screen.getByText("SI")).toBeInTheDocument();
  });

  it("상태별 카운트와 진행률을 표시", () => {
    const stats = dashboardStats([
      task({ id: 1, status: "done" }),
      task({ id: 2, status: "done" }),
      task({ id: 3, status: "todo" }),
      task({ id: 4, status: "doing" }),
    ]);
    render(<Dashboard business={business} stats={stats} />);
    expect(screen.getByTestId("count-done")).toHaveTextContent("2");
    expect(screen.getByTestId("count-todo")).toHaveTextContent("1");
    expect(screen.getByTestId("progress-pct")).toHaveTextContent("50%");
  });

  it("마감 임박 태스크를 보여준다", () => {
    const stats = dashboardStats([task({ id: 9, status: "todo", title: "마감있음", dueDate: "2026-07-01" })]);
    render(<Dashboard business={business} stats={stats} />);
    expect(screen.getByText("마감있음")).toBeInTheDocument();
    expect(screen.getByText("2026-07-01")).toBeInTheDocument();
  });
});
