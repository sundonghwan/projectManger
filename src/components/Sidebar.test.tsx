import { describe, it, expect, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { Sidebar, type SidebarProps } from "./Sidebar";
import type { TreeRow } from "../domain/tree";

const bizRow: TreeRow = {
  id: "business:1",
  type: "business",
  entityId: 1,
  depth: 0,
  label: "SI사업 A",
  hasChildren: true,
  expanded: true,
};
const dashRow: TreeRow = {
  id: "dashboard:1",
  type: "dashboard",
  entityId: 1,
  depth: 1,
  label: "대시보드",
  hasChildren: false,
  expanded: false,
};
const projRow: TreeRow = {
  id: "project:10",
  type: "project",
  entityId: 10,
  depth: 1,
  label: "프로젝트 1",
  hasChildren: false,
  expanded: false,
};

function setup(overrides: Partial<SidebarProps> = {}) {
  const props: SidebarProps = {
    rows: [bizRow, dashRow, projRow],
    selectedId: "business:1",
    colorFor: () => "#3b82f6",
    onSelect: vi.fn(),
    onToggle: vi.fn(),
    onAddBusiness: vi.fn(),
    onAddChild: vi.fn(),
    ...overrides,
  };
  return { props, ...render(<Sidebar {...props} />) };
}

describe("Sidebar", () => {
  it("행 라벨을 모두 렌더한다", () => {
    setup();
    expect(screen.getByText("SI사업 A")).toBeInTheDocument();
    expect(screen.getByText("대시보드")).toBeInTheDocument();
    expect(screen.getByText("프로젝트 1")).toBeInTheDocument();
  });

  it("빈 목록이면 안내 문구를 보여준다", () => {
    setup({ rows: [] });
    expect(screen.getByText(/아직 사업이 없습니다/)).toBeInTheDocument();
  });

  it("사업 행에는 유형 컬러 점을 colorFor 값으로 표시", () => {
    const colorFor = vi.fn(() => "#22c55e");
    setup({ colorFor });
    expect(colorFor).toHaveBeenCalledWith(1);
    expect(screen.getByTestId("biz-dot")).toHaveStyle({ background: "#22c55e" });
  });

  it("행 클릭 시 onSelect 호출", async () => {
    const { props } = setup();
    await userEvent.click(screen.getByText("프로젝트 1"));
    expect(props.onSelect).toHaveBeenCalledWith(projRow);
  });

  it("캐럿 클릭은 onToggle만 호출하고 onSelect는 호출하지 않는다", async () => {
    const { props } = setup();
    const bizItem = screen.getByText("SI사업 A").closest('[role="treeitem"]')!;
    const caret = bizItem.querySelector('button[aria-label="접기"]')!;
    await userEvent.click(caret);
    expect(props.onToggle).toHaveBeenCalledWith(bizRow);
    expect(props.onSelect).not.toHaveBeenCalled();
  });

  it("사업 추가 버튼은 onAddBusiness 호출", async () => {
    const { props } = setup();
    await userEvent.click(screen.getByRole("button", { name: "사업 추가" }));
    expect(props.onAddBusiness).toHaveBeenCalled();
  });

  it("하위 추가 버튼은 onAddChild 호출 (사업/프로젝트에만 노출)", async () => {
    const { props } = setup();
    const addButtons = screen.getAllByRole("button", { name: "하위 추가" });
    // 사업 1 + 프로젝트 1 = 2개 (대시보드에는 없음)
    expect(addButtons).toHaveLength(2);
    await userEvent.click(addButtons[0]);
    expect(props.onAddChild).toHaveBeenCalledWith(bizRow);
  });

  it("선택된 행은 aria-selected=true", () => {
    setup({ selectedId: "project:10" });
    const projItem = screen.getByText("프로젝트 1").closest('[role="treeitem"]')!;
    expect(projItem).toHaveAttribute("aria-selected", "true");
  });
});
