import { describe, it, expect } from "vitest";
import { render } from "@testing-library/react";
import { Icon } from "./Icon";
import { ICON_PATHS, type IconName } from "./paths";

const NAMES = Object.keys(ICON_PATHS) as IconName[];

describe("Icon", () => {
  it("모든 아이콘이 24×24 단색 svg 로 렌더된다", () => {
    for (const name of NAMES) {
      const { container, unmount } = render(<Icon name={name} />);
      const svg = container.querySelector("svg");
      expect(svg, name).not.toBeNull();
      expect(svg?.getAttribute("viewBox")).toBe("0 0 24 24");
      expect(svg?.getAttribute("stroke")).toBe("currentColor");
      expect(svg?.getAttribute("fill")).toBe("none");
      // 내부 path/line 등 글리프 요소가 하나 이상 있어야 함
      expect(svg?.childElementCount ?? 0).toBeGreaterThan(0);
      unmount();
    }
  });

  it("size 가 width/height 에 반영된다", () => {
    const { container } = render(<Icon name="plus" size={24} />);
    const svg = container.querySelector("svg");
    expect(svg?.getAttribute("width")).toBe("24");
    expect(svg?.getAttribute("height")).toBe("24");
  });

  it("aria-label 이 없으면 aria-hidden, 있으면 라벨이 노출된다", () => {
    const hidden = render(<Icon name="trash" />);
    expect(hidden.container.querySelector("svg")?.getAttribute("aria-hidden")).toBe("true");
    hidden.unmount();

    const labeled = render(<Icon name="trash" aria-label="휴지통" />);
    const svg = labeled.container.querySelector("svg");
    expect(svg?.getAttribute("aria-hidden")).toBeNull();
    expect(svg?.getAttribute("aria-label")).toBe("휴지통");
  });
});
