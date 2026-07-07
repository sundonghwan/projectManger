import { describe, it, expect, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { CreatePopover } from "./CreatePopover";

describe("CreatePopover - 사업", () => {
  it("자유 입력 사업 유형 + 이름 입력 후 만들기 → onCreateBusiness(type, name)", async () => {
    const onCreateBusiness = vi.fn();
    const onClose = vi.fn();
    render(
      <CreatePopover x={0} y={0} variant="business" onCreateBusiness={onCreateBusiness} onClose={onClose} />,
    );
    await userEvent.type(screen.getByLabelText("사업 유형"), "  철도  ");
    await userEvent.type(screen.getByLabelText("이름"), "분석 사업");
    await userEvent.click(screen.getByRole("button", { name: "만들기" }));
    expect(onCreateBusiness).toHaveBeenCalledWith("철도", "분석 사업");
    expect(onClose).toHaveBeenCalled();
  });

  it("이름 또는 사업 유형이 비면 만들기 비활성", async () => {
    render(<CreatePopover x={0} y={0} variant="business" onCreateBusiness={vi.fn()} onClose={vi.fn()} />);
    const createButton = screen.getByRole("button", { name: "만들기" });
    expect(createButton).toBeDisabled();
    await userEvent.type(screen.getByLabelText("사업 유형"), "플랫폼");
    expect(createButton).toBeDisabled();
    await userEvent.type(screen.getByLabelText("이름"), "포털");
    expect(createButton).toBeEnabled();
  });

  it("동적 사업 유형 칩 선택 후 Enter로 제출", async () => {
    const onCreateBusiness = vi.fn();
    render(
      <CreatePopover
        x={0}
        y={0}
        variant="business"
        businessTypeOptions={[
          { type: "철도", label: "철도", color: "#2563eb", count: 2 },
          { type: "플랫폼", label: "플랫폼", color: "#16a34a", count: 1 },
        ]}
        onCreateBusiness={onCreateBusiness}
        onClose={vi.fn()}
      />,
    );
    await userEvent.click(screen.getByRole("button", { name: "플랫폼" }));
    await userEvent.type(screen.getByLabelText("이름"), "포털 사업{Enter}");
    expect(onCreateBusiness).toHaveBeenCalledWith("플랫폼", "포털 사업");
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
