import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import type { Business } from "../domain/types";
import { BusinessEditorPopover } from "./BusinessEditorPopover";

const business: Business = {
  id: "b1",
  name: "철도청",
  type: "철도",
  color: "#3b82f6",
  description: "설명",
  status: "active",
  sortOrder: 1,
  archivedAt: null,
};

describe("BusinessEditorPopover", () => {
  it("edits business fields and saves", async () => {
    const onSave = vi.fn();
    render(
      <BusinessEditorPopover
        business={business}
        businessTypeOptions={[{ type: "플랫폼", label: "플랫폼", color: "#22c55e", count: 1 }]}
        onSave={onSave}
        onClose={vi.fn()}
      />,
    );

    await userEvent.clear(screen.getByLabelText("사업 이름"));
    await userEvent.type(screen.getByLabelText("사업 이름"), "통합관제");
    await userEvent.clear(screen.getByLabelText("사업 유형"));
    await userEvent.type(screen.getByLabelText("사업 유형"), "플랫폼");
    await userEvent.clear(screen.getByLabelText("색상"));
    await userEvent.type(screen.getByLabelText("색상"), "#22c55e");
    await userEvent.selectOptions(screen.getByLabelText("상태"), "onhold");
    await userEvent.clear(screen.getByLabelText("설명"));
    await userEvent.type(screen.getByLabelText("설명"), "대기 중");
    await userEvent.click(screen.getByRole("button", { name: "저장" }));

    expect(onSave).toHaveBeenCalledWith({
      id: "b1",
      name: "통합관제",
      type: "플랫폼",
      color: "#22c55e",
      status: "onhold",
      description: "대기 중",
    });
  });

  it("empty name or type disables save", async () => {
    render(
      <BusinessEditorPopover
        business={business}
        businessTypeOptions={[]}
        onSave={vi.fn()}
        onClose={vi.fn()}
      />,
    );

    await userEvent.clear(screen.getByLabelText("사업 유형"));
    expect(screen.getByRole("button", { name: "저장" })).toBeDisabled();

    await userEvent.type(screen.getByLabelText("사업 유형"), "플랫폼");
    await userEvent.clear(screen.getByLabelText("사업 이름"));
    expect(screen.getByRole("button", { name: "저장" })).toBeDisabled();
  });

  it("cancel closes without saving", async () => {
    const onSave = vi.fn();
    const onClose = vi.fn();
    render(
      <BusinessEditorPopover
        business={business}
        businessTypeOptions={[]}
        onSave={onSave}
        onClose={onClose}
      />,
    );

    await userEvent.click(screen.getByRole("button", { name: "취소" }));

    expect(onSave).not.toHaveBeenCalled();
    expect(onClose).toHaveBeenCalledOnce();
  });
});
