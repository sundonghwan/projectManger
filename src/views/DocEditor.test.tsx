import { describe, it, expect, vi } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import type { Document } from "../domain/types";

vi.mock("../api/client", () => ({
  api: {
    document: {
      get: vi.fn().mockResolvedValue({ body: "# 안녕\n본문" }),
      setBody: vi.fn().mockResolvedValue(undefined),
    },
  },
}));

import { DocEditor } from "./DocEditor";
import { api } from "../api/client";

const doc: Document = {
  id: "1",
  businessId: "1",
  projectId: null,
  title: "기획안",
  icon: null,
  body: "# 안녕",
  sortOrder: 1,
  archivedAt: null,
  createdAt: "2026-06-22T00:00:00Z",
};

describe("DocEditor (markdown)", () => {
  it("마운트 시 최신 본문을 불러와 textarea에 표시", async () => {
    render(<DocEditor document={doc} />);
    const ta = screen.getByLabelText("문서 본문") as HTMLTextAreaElement;
    await waitFor(() => expect(ta.value).toBe("# 안녕\n본문"));
  });

  it("미리보기 토글 시 마크다운을 렌더", async () => {
    render(<DocEditor document={doc} />);
    const ta = screen.getByLabelText("문서 본문") as HTMLTextAreaElement;
    await waitFor(() => expect(ta.value).toBe("# 안녕\n본문"));
    await userEvent.click(screen.getByRole("button", { name: "미리보기" }));
    expect(screen.getByRole("heading", { level: 1, name: "안녕" })).toBeInTheDocument();
  });

  it("미리보기에서 위험한 HTML을 제거(sanitize)한다", async () => {
    vi.mocked(api.document.get).mockResolvedValueOnce({
      body: '텍스트 <img src=x onerror="alert(1)"> 끝\n\n[클릭](javascript:alert(2))',
    } as never);
    render(<DocEditor document={{ ...doc, body: "" }} />);
    const ta = screen.getByLabelText("문서 본문") as HTMLTextAreaElement;
    await waitFor(() => expect(ta.value).toContain("onerror"));
    await userEvent.click(screen.getByRole("button", { name: "미리보기" }));
    const preview = document.querySelector(".md-preview") as HTMLElement;
    // onerror 핸들러 제거 + 링크에서 javascript: 스킴 제거
    expect(preview.innerHTML).not.toContain("onerror");
    const link = preview.querySelector("a");
    expect(link?.getAttribute("href") ?? "").not.toContain("javascript:");
  });

  it("입력 후 blur 시 setBody로 저장", async () => {
    render(<DocEditor document={doc} />);
    const ta = screen.getByLabelText("문서 본문") as HTMLTextAreaElement;
    await waitFor(() => expect(ta.value).toBe("# 안녕\n본문"));
    await userEvent.type(ta, " 끝");
    ta.blur();
    expect(api.document.setBody).toHaveBeenCalledWith("1", "# 안녕\n본문 끝");
  });
});
