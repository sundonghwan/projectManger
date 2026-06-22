import { describe, it, expect, vi } from "vitest";
import { render, screen, within } from "@testing-library/react";
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
    onArchive: vi.fn(),
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

  it("사업 행의 보관 버튼은 onArchive 호출, 대시보드 행엔 없음", async () => {
    const { props } = setup();
    await userEvent.click(screen.getByRole("button", { name: "SI사업 A 보관" }));
    expect(props.onArchive).toHaveBeenCalledWith(bizRow);
    expect(screen.queryByRole("button", { name: "대시보드 보관" })).not.toBeInTheDocument();
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

  it("사업 추가 → 유형 선택 + 이름 입력 후 onAddBusiness(type, name)", async () => {
    const { props } = setup();
    await userEvent.click(screen.getByRole("button", { name: "사업 추가" }));
    await userEvent.click(screen.getByRole("button", { name: /내부개발/ }));
    await userEvent.type(screen.getByLabelText("이름"), "신규 사업");
    await userEvent.click(screen.getByRole("button", { name: "만들기" }));
    expect(props.onAddBusiness).toHaveBeenCalledWith("internal", "신규 사업");
  });

  it("사업 [+] → 프로젝트 생성, 문서·산출물 옵션은 없다", async () => {
    const { props } = setup();
    const addButtons = screen.getAllByRole("button", { name: "하위 추가" });
    // 사업에만 하위 추가 버튼(프로젝트). 대시보드·프로젝트엔 없음
    expect(addButtons).toHaveLength(1);
    await userEvent.click(addButtons[0]); // 사업 노드
    const dialog = screen.getByRole("dialog");
    // 문서·산출물은 각 목록에서 생성하므로 트리 추가 메뉴엔 없다
    expect(within(dialog).queryByRole("button", { name: /문서/ })).not.toBeInTheDocument();
    expect(within(dialog).queryByRole("button", { name: /산출물/ })).not.toBeInTheDocument();
    await userEvent.click(within(dialog).getByRole("button", { name: /프로젝트/ }));
    await userEvent.type(screen.getByLabelText("이름"), "신규 프로젝트");
    await userEvent.click(screen.getByRole("button", { name: "만들기" }));
    expect(props.onAddChild).toHaveBeenCalledWith(bizRow, "project", "신규 프로젝트");
  });

  it("프로젝트 행에는 하위 추가 버튼이 없다", () => {
    setup();
    const projItem = screen.getByText("프로젝트 1").closest('[role="treeitem"]') as HTMLElement;
    expect(within(projItem).queryByRole("button", { name: "하위 추가" })).toBeNull();
  });

  it("선택된 행은 aria-selected=true", () => {
    setup({ selectedId: "project:10" });
    const projItem = screen.getByText("프로젝트 1").closest('[role="treeitem"]')!;
    expect(projItem).toHaveAttribute("aria-selected", "true");
  });

  it("유형 필터 칩 클릭 시 onToggleType 호출", async () => {
    const onToggleType = vi.fn();
    setup({ onToggleType, typeFilter: new Set() });
    await userEvent.click(screen.getByRole("button", { name: /내부개발/ }));
    expect(onToggleType).toHaveBeenCalledWith("internal");
  });

  it("라벨 더블클릭 후 Enter로 이름변경 → onRename 호출", async () => {
    const onRename = vi.fn();
    setup({ onRename });
    await userEvent.dblClick(screen.getByText("프로젝트 1"));
    const input = screen.getByLabelText("이름 변경");
    await userEvent.clear(input);
    await userEvent.type(input, "새 프로젝트명{Enter}");
    expect(onRename).toHaveBeenCalledWith(projRow, "새 프로젝트명");
  });
});
