import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import { Terminal } from "./Terminal";
import { api } from "../api/client";
import type { ServerConnection } from "../domain/types";

const mocks = vi.hoisted(() => {
  class MockTerminal {
    static latest: MockTerminal | null = null;
    options: unknown;
    rows = 24;
    cols = 80;
    onDataHandler: ((data: string) => void) | null = null;
    disposed = false;

    constructor(options: unknown) {
      this.options = options;
      MockTerminal.latest = this;
    }

    loadAddon = vi.fn();
    open = vi.fn();
    write = vi.fn();
    focus = vi.fn();
    dispose = vi.fn(() => {
      this.disposed = true;
    });
    onData = vi.fn((handler: (data: string) => void) => {
      this.onDataHandler = handler;
      return { dispose: vi.fn() };
    });
  }

  return {
    MockTerminal,
    fitMock: vi.fn(),
  };
});

vi.mock("@xterm/xterm", () => ({
  Terminal: mocks.MockTerminal,
}));

vi.mock("@xterm/addon-fit", () => ({
  FitAddon: class {
    fit = mocks.fitMock;
  },
}));

vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn().mockResolvedValue(vi.fn()),
}));

vi.mock("../api/client", () => ({
  api: {
    ssh: {
      connect: vi.fn().mockResolvedValue(undefined),
      connectLocal: vi.fn().mockResolvedValue(undefined),
      disconnect: vi.fn().mockResolvedValue(undefined),
      resize: vi.fn().mockResolvedValue(undefined),
      write: vi.fn().mockResolvedValue(undefined),
    },
    snippet: {
      list: vi.fn().mockReturnValue(new Promise(() => {})),
      create: vi.fn().mockResolvedValue(undefined),
      delete: vi.fn().mockResolvedValue(undefined),
    },
    server: {
      update: vi.fn().mockResolvedValue(undefined),
    },
  },
}));

const server: ServerConnection = {
  id: "srv-1",
  businessId: "biz-1",
  projectId: null,
  name: "GPU",
  host: "192.168.11.136",
  port: 22,
  username: "dhsun",
  authType: "password",
  keyPath: null,
  secretRef: "saved",
};

describe("Terminal", () => {
  it("uses xterm as the input surface without rendering the extra textarea", () => {
    render(<Terminal server={server} onClose={vi.fn()} />);

    expect(screen.queryByLabelText("터미널 입력")).not.toBeInTheDocument();
    expect(screen.queryByPlaceholderText("터미널 입력")).not.toBeInTheDocument();
    expect(mocks.MockTerminal.latest?.options).toMatchObject({ disableStdin: false });
    expect(mocks.MockTerminal.latest?.onData).toHaveBeenCalled();
  });

  it("forwards xterm data to the SSH PTY as NFC text", () => {
    render(<Terminal server={server} onClose={vi.fn()} />);

    mocks.MockTerminal.latest?.onDataHandler?.("테스트");

    expect(api.ssh.write).toHaveBeenCalledWith("srv-1", "테스트");
  });

  it("aiBridge가 false면 배지를 렌더하지 않음", () => {
    render(<Terminal server={server} onClose={vi.fn()} />);

    expect(screen.queryByText("AI 브리지")).not.toBeInTheDocument();
  });

  it("aiBridge가 true면 헤더에 AI 브리지 배지 표시", () => {
    render(<Terminal server={{ ...server, aiBridge: true }} onClose={vi.fn()} />);

    expect(screen.getByText("AI 브리지")).toBeInTheDocument();
  });

  it("aiBridge가 false면 꺼짐 상태 칩을 표시", () => {
    render(<Terminal server={server} onClose={vi.fn()} />);

    expect(screen.getByText("AI 브리지 꺼짐")).toBeInTheDocument();
  });

  it("local 세션이면 브리지 칩을 렌더하지 않음", () => {
    render(<Terminal server={server} onClose={vi.fn()} local />);

    expect(screen.queryByText("AI 브리지")).not.toBeInTheDocument();
    expect(screen.queryByText("AI 브리지 꺼짐")).not.toBeInTheDocument();
  });

  it("aiBridge가 true면 claude/codex 런처 버튼을 표시하고 클릭 시 write 호출", async () => {
    const user = userEvent.setup();
    render(<Terminal server={{ ...server, aiBridge: true }} onClose={vi.fn()} />);

    const claudeBtn = screen.getByRole("button", { name: "claude" });
    const codexBtn = screen.getByRole("button", { name: "codex" });
    expect(claudeBtn).toBeInTheDocument();
    expect(codexBtn).toBeInTheDocument();

    await user.click(claudeBtn);
    expect(api.ssh.write).toHaveBeenCalledWith("srv-1", "claude\r");

    await user.click(codexBtn);
    expect(api.ssh.write).toHaveBeenCalledWith("srv-1", "codex\r");
  });

  it("aiBridge가 false면 런처 버튼을 렌더하지 않음", () => {
    render(<Terminal server={server} onClose={vi.fn()} />);

    expect(screen.queryByRole("button", { name: "claude" })).not.toBeInTheDocument();
    expect(screen.queryByRole("button", { name: "codex" })).not.toBeInTheDocument();
  });

  it("브리지 칩 클릭 시 server.update로 반전값을 저장하고 재연결한다", async () => {
    const user = userEvent.setup();
    render(<Terminal server={server} onClose={vi.fn()} />);

    const chip = screen.getByText("AI 브리지 꺼짐");
    await user.click(chip);

    await waitFor(() => {
      expect(api.server.update).toHaveBeenCalledWith(
        expect.objectContaining({ id: "srv-1", aiBridge: true }),
      );
    });
    expect(api.ssh.disconnect).toHaveBeenCalledWith("srv-1");

    // 로컬 상태가 즉시 반영되어 칩 라벨이 켜짐 상태로 바뀐다.
    await waitFor(() => {
      expect(screen.getByText("AI 브리지")).toBeInTheDocument();
    });

    // disconnect 후 재연결을 위해 connect가 다시 호출된다(초기 1회 + 재연결 1회).
    await waitFor(() => {
      expect((api.ssh.connect as ReturnType<typeof vi.fn>).mock.calls.length).toBeGreaterThanOrEqual(2);
    });
  });
});
