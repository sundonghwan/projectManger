import { describe, it, expect, vi } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { GlobalSearch } from "./GlobalSearch";
import type { SearchHit } from "../domain/types";

const hits: SearchHit[] = [
  { kind: "business", id: 1, title: "알파 사업", businessId: 1, projectId: null },
  { kind: "task", id: 9, title: "알파 태스크", businessId: 1, projectId: 2 },
];

describe("GlobalSearch", () => {
  it("입력 시 onSearch 호출 후 결과를 보여준다", async () => {
    const onSearch = vi.fn(async () => hits);
    render(<GlobalSearch onSearch={onSearch} onPick={vi.fn()} />);
    await userEvent.type(screen.getByLabelText("검색"), "알파");
    await waitFor(() => expect(onSearch).toHaveBeenCalledWith("알파"));
    expect(await screen.findByText("알파 사업")).toBeInTheDocument();
    expect(screen.getByText("알파 태스크")).toBeInTheDocument();
  });

  it("결과 클릭 시 onPick 호출", async () => {
    const onPick = vi.fn();
    render(<GlobalSearch onSearch={async () => hits} onPick={onPick} />);
    await userEvent.type(screen.getByLabelText("검색"), "알파");
    const opt = await screen.findByText("알파 태스크");
    await userEvent.click(opt);
    expect(onPick).toHaveBeenCalledWith(hits[1]);
  });

  it("결과 없으면 안내", async () => {
    render(<GlobalSearch onSearch={async () => []} onPick={vi.fn()} />);
    await userEvent.type(screen.getByLabelText("검색"), "없는것");
    expect(await screen.findByText("결과 없음")).toBeInTheDocument();
  });
});
