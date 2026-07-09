import { describe, it, expect, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { ServerPanel } from "./ServerPanel";
import type { ServerConnection } from "../domain/types";

const server = (over: Partial<ServerConnection> = {}): ServerConnection => ({
  id: "1",
  businessId: "1",
  name: "스테이징",
  host: "10.0.0.5",
  port: 22,
  username: "deploy",
  authType: "password",
  ...over,
});

function setup(servers: ServerConnection[] = [server()]) {
  const h = {
    onCreate: vi.fn(),
    onUpdate: vi.fn(),
    onArchive: vi.fn(),
    onSetSecret: vi.fn(),
    onClearSecret: vi.fn(),
    onConnect: vi.fn(),
  };
  render(<ServerPanel servers={servers} {...h} />);
  return h;
}

describe("ServerPanel", () => {
  it("서버를 카드로 렌더", () => {
    setup();
    expect(screen.getByText("스테이징")).toBeInTheDocument();
    expect(screen.getByText("deploy@10.0.0.5:22")).toBeInTheDocument();
  });

  it("빈 목록 안내", () => {
    setup([]);
    expect(screen.getByText("등록된 서버가 없습니다.")).toBeInTheDocument();
  });

  it("폼 작성 후 추가하면 onCreate", async () => {
    const h = setup([]);
    await userEvent.type(screen.getByLabelText("이름"), "운영");
    await userEvent.type(screen.getByLabelText("호스트"), "prod.x");
    await userEvent.type(screen.getByLabelText("사용자"), "admin");
    await userEvent.click(screen.getByRole("button", { name: "추가" }));
    expect(h.onCreate).toHaveBeenCalledWith(
      expect.objectContaining({ name: "운영", host: "prod.x", username: "admin", authType: "key" }),
    );
  });

  it("접속 버튼은 onConnect", async () => {
    const h = setup();
    await userEvent.click(screen.getByRole("button", { name: "접속" }));
    expect(h.onConnect).toHaveBeenCalledWith(expect.objectContaining({ id: "1" }));
  });

  it("시크릿 저장은 onSetSecret", async () => {
    const h = setup();
    await userEvent.type(screen.getByLabelText("스테이징 비밀값"), "pw123");
    await userEvent.click(screen.getByRole("button", { name: "시크릿 저장" }));
    expect(h.onSetSecret).toHaveBeenCalledWith("1", "pw123");
  });

  it("저장된 비밀번호는 변경 버튼을 누른 뒤 새 값 저장", async () => {
    const h = setup([server({ secretRef: "ssh/conn-1" })]);

    expect(screen.queryByLabelText("스테이징 비밀값")).not.toBeInTheDocument();
    await userEvent.click(screen.getByRole("button", { name: "스테이징 비밀번호 변경" }));
    await userEvent.type(screen.getByLabelText("스테이징 비밀값"), "new-pw");
    await userEvent.click(screen.getByRole("button", { name: "비밀번호 변경 저장" }));

    expect(h.onSetSecret).toHaveBeenCalledWith("1", "new-pw");
  });

  it("저장된 비밀번호 삭제는 onClearSecret", async () => {
    const h = setup([server({ secretRef: "ssh/conn-1" })]);

    await userEvent.click(screen.getByRole("button", { name: "스테이징 비밀번호 삭제" }));

    expect(h.onClearSecret).toHaveBeenCalledWith("1");
  });

  it("보관 버튼은 onArchive", async () => {
    const h = setup();
    await userEvent.click(screen.getByRole("button", { name: "스테이징 보관" }));
    expect(h.onArchive).toHaveBeenCalledWith("1");
  });

  it("AI 브리지 체크박스를 켜고 추가하면 onCreate에 aiBridge: true", async () => {
    const h = setup([]);
    await userEvent.type(screen.getByLabelText("이름"), "운영");
    await userEvent.type(screen.getByLabelText("호스트"), "prod.x");
    await userEvent.type(screen.getByLabelText("사용자"), "admin");
    await userEvent.click(screen.getByText("AI 자격증명 브리지 (claude/codex 원격 사용)"));
    await userEvent.click(screen.getByRole("button", { name: "추가" }));
    expect(h.onCreate).toHaveBeenCalledWith(expect.objectContaining({ aiBridge: true }));
  });

  it("AI 브리지를 켜지 않으면 기본값 false로 onCreate", async () => {
    const h = setup([]);
    await userEvent.type(screen.getByLabelText("이름"), "운영");
    await userEvent.type(screen.getByLabelText("호스트"), "prod.x");
    await userEvent.type(screen.getByLabelText("사용자"), "admin");
    await userEvent.click(screen.getByRole("button", { name: "추가" }));
    expect(h.onCreate).toHaveBeenCalledWith(expect.objectContaining({ aiBridge: false }));
  });
});
