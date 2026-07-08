import { describe, it, expect, vi, beforeEach } from "vitest";

// Tauri core invoke 모킹
const invoke = vi.fn();
vi.mock("@tauri-apps/api/core", () => ({
  invoke: (...args: unknown[]) => invoke(...args),
}));
vi.mock("@tauri-apps/plugin-opener", () => ({
  openPath: vi.fn(),
}));

import { api } from "./client";

beforeEach(() => {
  invoke.mockReset();
  invoke.mockResolvedValue(undefined);
});

describe("api.business", () => {
  it("list는 business_list를 인자 없이 호출", async () => {
    await api.business.list();
    expect(invoke).toHaveBeenCalledWith("business_list");
  });

  it("create는 input 래퍼로 호출", async () => {
    await api.business.create({ name: "A", type: "si", color: "#3b82f6" });
    expect(invoke).toHaveBeenCalledWith("business_create", {
      input: { name: "A", type: "si", color: "#3b82f6" },
    });
  });

  it("archive는 id를 전달", async () => {
    await api.business.archive("7");
    expect(invoke).toHaveBeenCalledWith("business_archive", { id: "7" });
  });
});

describe("api.project", () => {
  it("list는 businessId를 전달", async () => {
    await api.project.list("3");
    expect(invoke).toHaveBeenCalledWith("project_list", { businessId: "3" });
  });

  it("create는 input 래퍼로 호출", async () => {
    await api.project.create({ businessId: "3", name: "P1" });
    expect(invoke).toHaveBeenCalledWith("project_create", {
      input: { businessId: "3", name: "P1" },
    });
  });
});

describe("api.task", () => {
  it("list는 projectId 없으면 null로 정규화", async () => {
    await api.task.list("3");
    expect(invoke).toHaveBeenCalledWith("task_list", { businessId: "3", projectId: null });
  });

  it("move는 칸반 이동 인자를 input으로 전달", async () => {
    await api.task.move({ id: "1", status: "done", sortOrder: 5 });
    expect(invoke).toHaveBeenCalledWith("task_move", {
      input: { id: "1", status: "done", sortOrder: 5 },
    });
  });
});

describe("api.deliverable", () => {
  it("uploadFiles는 DOM drop 파일 payload를 백엔드에 전달", async () => {
    await api.deliverable.uploadFiles("biz-1", "project-1", [{ fileName: "a.pdf", bytes: [1, 2] }], "folder-1");
    expect(invoke).toHaveBeenCalledWith("deliverable_upload_files", {
      businessId: "biz-1",
      projectId: "project-1",
      folderId: "folder-1",
      files: [{ fileName: "a.pdf", bytes: [1, 2] }],
    });
  });

  it("open은 파일 경로 대신 산출물 id만 백엔드에 전달", async () => {
    await api.deliverable.open("deliv-1");
    expect(invoke).toHaveBeenCalledWith("deliverable_open", { id: "deliv-1" });
  });

  it("showInFolder는 산출물 id만 백엔드에 전달", async () => {
    await api.deliverable.showInFolder("deliv-1");
    expect(invoke).toHaveBeenCalledWith("deliverable_show_in_folder", { id: "deliv-1" });
  });
});

describe("api.document markdown body", () => {
  it("uploads a document asset", async () => {
    await api.document.uploadAsset("doc-1", "image.png", [1, 2, 3]);

    expect(invoke).toHaveBeenCalledWith("document_asset_upload", {
      documentId: "doc-1",
      fileName: "image.png",
      bytes: [1, 2, 3],
    });
  });

  it("opens the generated Markdown export folder by document id", async () => {
    await api.document.showExportFolder("doc-1");
    expect(invoke).toHaveBeenCalledWith("document_show_export_folder", { id: "doc-1" });
  });
});
