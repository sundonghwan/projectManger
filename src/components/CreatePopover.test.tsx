import { describe, it, expect, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { CreatePopover } from "./CreatePopover";

describe("CreatePopover - 사업", () => {
  it("유형 선택 + 이름 입력 후 만들기 → onCreateBusiness(type, name)", async () => {
    const onCreateBusiness = vi.fn();
    const onClose = vi.fn();
    render(
      <CreatePopover x={0} y={0} variant="business" onCreateBusiness={onCreateBusiness} onClose={onClose} />,
    );
    await userEvent.click(screen.getByRole("button", { name: /운영/ }));
    await userEvent.type(screen.getByLabelText("이름"), "분석 사업");
    await userEvent.click(screen.getByRole("button", { name: "만들기" }));
    expect(onCreateBusiness).toHaveBeenCalledWith("ops", "분석 사업");
    expect(onClose).toHaveBeenCalled();
  });

  it("이름이 비면 만들기 비활성", () => {
    render(<CreatePopover x={0} y={0} variant="business" onCreateBusiness={vi.fn()} onClose={vi.fn()} />);
    expect(screen.getByRole("button", { name: "만들기" })).toBeDisabled();
  });

  it("Enter로 제출", async () => {
    const onCreateBusiness = vi.fn();
    render(<CreatePopover x={0} y={0} variant="business" onCreateBusiness={onCreateBusiness} onClose={vi.fn()} />);
    await userEvent.type(screen.getByLabelText("이름"), "SI 사업{Enter}");
    expect(onCreateBusiness).toHaveBeenCalledWith("etc", "SI 사업");
  });
});

describe("CreatePopover - 하위", () => {
  it("종류 선택 + 이름 입력 → onCreateChild(kind, name)", async () => {
    const onCreateChild = vi.fn();
    render(
      <CreatePopover
        x={0}
        y={0}
        variant="child"
        allowedKinds={["document", "deliverable"]}
        onCreateChild={onCreateChild}
        onClose={vi.fn()}
      />,
    );
    // 프로젝트 옵션은 없음
    expect(screen.queryByRole("button", { name: /프로젝트/ })).not.toBeInTheDocument();
    await userEvent.click(screen.getByRole("button", { name: /산출물/ }));
    await userEvent.type(screen.getByLabelText("이름"), "최종 보고서");
    await userEvent.click(screen.getByRole("button", { name: "만들기" }));
    expect(onCreateChild).toHaveBeenCalledWith("deliverable", "최종 보고서");
  });

  it("기본 선택은 첫 허용 종류", async () => {
    const onCreateChild = vi.fn();
    render(
      <CreatePopover x={0} y={0} variant="child" allowedKinds={["project", "document"]} onCreateChild={onCreateChild} onClose={vi.fn()} />,
    );
    await userEvent.type(screen.getByLabelText("이름"), "기획{Enter}");
    expect(onCreateChild).toHaveBeenCalledWith("project", "기획");
  });
});
