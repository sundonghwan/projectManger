import { describe, it, expect, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { AddMenu } from "./AddMenu";

const options = [
  { key: "project", label: "프로젝트" },
  { key: "document", label: "문서" },
];

function setup() {
  const h = { onSelect: vi.fn(), onClose: vi.fn() };
  render(<AddMenu x={10} y={20} options={options} {...h} />);
  return h;
}

describe("AddMenu", () => {
  it("옵션을 렌더한다", () => {
    setup();
    expect(screen.getByRole("menuitem", { name: "프로젝트" })).toBeInTheDocument();
    expect(screen.getByRole("menuitem", { name: "문서" })).toBeInTheDocument();
  });

  it("옵션 선택 시 onSelect(key) 후 onClose", async () => {
    const h = setup();
    await userEvent.click(screen.getByRole("menuitem", { name: "문서" }));
    expect(h.onSelect).toHaveBeenCalledWith("document");
    expect(h.onClose).toHaveBeenCalled();
  });

  it("바깥(overlay) 클릭 시 onClose", async () => {
    const h = setup();
    await userEvent.click(screen.getByTestId("addmenu-overlay"));
    expect(h.onClose).toHaveBeenCalled();
  });
});
