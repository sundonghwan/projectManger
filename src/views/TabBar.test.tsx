import { describe, it, expect, vi } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { TabBar } from "./TabBar";
import type { Tab } from "./tabs";

const tab = (key: string, title: string): Tab => ({
  key, title, kind: "ssh",
  server: { id: key, businessId: "b", projectId: null, name: title, host: "h", port: 22, username: "u", authType: "key", aiBridge: false },
});

describe("TabBar", () => {
  it("renders titles and calls onSelect on click", () => {
    const onSelect = vi.fn();
    render(<TabBar tabs={[tab("1", "A"), tab("2", "B")]} activeKey="1" onSelect={onSelect} onClose={vi.fn()} />);
    fireEvent.click(screen.getByText("B"));
    expect(onSelect).toHaveBeenCalledWith("2");
  });

  it("calls onClose when the × of a tab is clicked", () => {
    const onClose = vi.fn();
    render(<TabBar tabs={[tab("1", "A")]} activeKey="1" onSelect={vi.fn()} onClose={onClose} />);
    fireEvent.click(screen.getByRole("button", { name: /A 닫기/ }));
    expect(onClose).toHaveBeenCalledWith("1");
  });
});
