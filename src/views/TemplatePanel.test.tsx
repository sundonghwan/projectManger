import { describe, it, expect, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { TemplatePanel } from "./TemplatePanel";
import type { Template } from "../domain/types";

const tpl: Template = { id: "1", name: "표준 프로젝트", kind: "project", payload: "{}" };

function setup(templates: Template[] = [tpl]) {
  const h = { onApply: vi.fn(), onCreate: vi.fn(), onDelete: vi.fn() };
  render(<TemplatePanel templates={templates} {...h} />);
  return h;
}

describe("TemplatePanel", () => {
  it("템플릿 렌더 + 적용", async () => {
    const h = setup();
    expect(screen.getByTestId("template-1")).toHaveTextContent("표준 프로젝트");
    await userEvent.click(screen.getByRole("button", { name: "적용" }));
    expect(h.onApply).toHaveBeenCalledWith(tpl);
  });

  it("이름 입력 후 추가 → onCreate", async () => {
    const h = setup([]);
    await userEvent.type(screen.getByLabelText("템플릿 이름"), "회의록");
    await userEvent.click(screen.getByRole("button", { name: "추가" }));
    expect(h.onCreate).toHaveBeenCalledWith(
      expect.objectContaining({ name: "회의록", kind: "project" }),
    );
  });

  it("삭제 → onDelete", async () => {
    const h = setup();
    await userEvent.click(screen.getByRole("button", { name: "표준 프로젝트 삭제" }));
    expect(h.onDelete).toHaveBeenCalledWith("1");
  });
});
