import { describe, it, expect, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { SftpList } from "./SftpList";
import type { SftpEntry } from "../domain/types";

const entries: SftpEntry[] = [
  { name: "src", isDir: true, size: 4096 },
  { name: "README.md", isDir: false, size: 123 },
];

function setup(over: Partial<Parameters<typeof SftpList>[0]> = {}) {
  const h = { onUp: vi.fn(), onOpen: vi.fn(), onClose: vi.fn() };
  render(<SftpList path="/home/u" entries={entries} error={null} {...h} {...over} />);
  return h;
}

describe("SftpList", () => {
  it("경로와 항목 렌더", () => {
    setup();
    expect(screen.getByText("/home/u")).toBeInTheDocument();
    expect(screen.getByTestId("sftp-src")).toBeInTheDocument();
    expect(screen.getByTestId("sftp-README.md")).toBeInTheDocument();
  });

  it("디렉터리 클릭 시 onOpen, 파일은 무시", async () => {
    const h = setup();
    await userEvent.click(screen.getByTestId("sftp-src"));
    expect(h.onOpen).toHaveBeenCalledWith(entries[0]);
    await userEvent.click(screen.getByTestId("sftp-README.md"));
    expect(h.onOpen).toHaveBeenCalledTimes(1);
  });

  it("상위/닫기 버튼", async () => {
    const h = setup();
    await userEvent.click(screen.getByLabelText("상위 폴더"));
    expect(h.onUp).toHaveBeenCalled();
    await userEvent.click(screen.getByLabelText("파일 브라우저 닫기"));
    expect(h.onClose).toHaveBeenCalled();
  });

  it("에러 표시", () => {
    setup({ error: "연결 실패" });
    expect(screen.getByText(/연결 실패/)).toBeInTheDocument();
  });
});
