import { describe, it, expect, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { LabelChips } from "./LabelChips";
import type { Label } from "../domain/types";

const labels: Label[] = [
  { id: "1", name: "백엔드", color: "#3b82f6" },
  { id: "2", name: "긴급", color: "#ef4444" },
];

describe("LabelChips", () => {
  it("라벨이 없으면 아무것도 렌더하지 않는다", () => {
    const { container } = render(<LabelChips labels={[]} />);
    expect(container).toBeEmptyDOMElement();
  });

  it("라벨 칩을 렌더한다", () => {
    render(<LabelChips labels={labels} />);
    expect(screen.getByText("백엔드")).toBeInTheDocument();
    expect(screen.getByText("긴급")).toBeInTheDocument();
  });

  it("onRemove 없으면 제거 버튼이 없다", () => {
    render(<LabelChips labels={labels} />);
    expect(screen.queryByRole("button")).not.toBeInTheDocument();
  });

  it("onRemove 있으면 × 클릭 시 호출", async () => {
    const onRemove = vi.fn();
    render(<LabelChips labels={labels} onRemove={onRemove} />);
    await userEvent.click(screen.getByRole("button", { name: "백엔드 라벨 제거" }));
    expect(onRemove).toHaveBeenCalledWith(labels[0]);
  });
});
