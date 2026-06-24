import { describe, it, expect, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { RecurringPanel } from "./RecurringPanel";
import type { RecurringTask } from "../domain/types";

const item: RecurringTask = {
  id: "1",
  businessId: "1",
  title: "주간보고",
  priority: 2,
  intervalDays: 7,
  nextRun: "2026-07-01",
  active: 1,
};

function setup(items: RecurringTask[] = [item]) {
  const h = { onCreate: vi.fn(), onToggle: vi.fn(), onDelete: vi.fn(), onGenerate: vi.fn() };
  render(<RecurringPanel items={items} {...h} />);
  return h;
}

describe("RecurringPanel", () => {
  it("반복 항목 렌더", () => {
    setup();
    expect(screen.getByTestId("recurring-1")).toHaveTextContent("주간보고");
  });

  it("폼 작성 후 추가 → onCreate", async () => {
    const h = setup([]);
    await userEvent.type(screen.getByLabelText("반복 제목"), "정산");
    await userEvent.type(screen.getByLabelText("다음 실행"), "2026-07-15");
    await userEvent.click(screen.getByRole("button", { name: "추가" }));
    expect(h.onCreate).toHaveBeenCalledWith(
      expect.objectContaining({ title: "정산", nextRun: "2026-07-15", intervalDays: 7 }),
    );
  });

  it("일시중지/삭제/지금 생성", async () => {
    const h = setup();
    await userEvent.click(screen.getByRole("button", { name: "일시중지" }));
    expect(h.onToggle).toHaveBeenCalledWith(item);
    await userEvent.click(screen.getByRole("button", { name: "주간보고 삭제" }));
    expect(h.onDelete).toHaveBeenCalledWith("1");
    await userEvent.click(screen.getByRole("button", { name: "지금 생성" }));
    expect(h.onGenerate).toHaveBeenCalled();
  });
});
