import { describe, expect, it, vi } from "vitest";
import { assertSupportedImageName, createDocumentImageUploader } from "./documentAssets";

describe("documentAssets", () => {
  it("accepts supported image file names", () => {
    expect(() => assertSupportedImageName("a.png")).not.toThrow();
    expect(() => assertSupportedImageName("a.jpg")).not.toThrow();
    expect(() => assertSupportedImageName("a.jpeg")).not.toThrow();
    expect(() => assertSupportedImageName("a.gif")).not.toThrow();
    expect(() => assertSupportedImageName("a.webp")).not.toThrow();
  });

  it("rejects unsupported image file names", () => {
    expect(() => assertSupportedImageName("a.pdf")).toThrow("지원하지 않는 이미지 형식입니다");
  });

  it("uploads through the API and converts the returned file path to an asset URL", async () => {
    const uploadAsset = vi.fn().mockResolvedValue({
      id: "asset-1",
      documentId: "doc-1",
      fileName: "a.png",
      filePath: "/vault/files/documents/doc-1/assets/asset-1/a.png",
      url: "/vault/files/documents/doc-1/assets/asset-1/a.png",
    });
    const convertFileSrc = vi.fn((path: string) => `asset://${path}`);
    const upload = createDocumentImageUploader({ documentId: "doc-1", uploadAsset, convertFileSrc });

    const result = await upload(new File(["x"], "a.png", { type: "image/png" }));

    expect(uploadAsset).toHaveBeenCalledWith("doc-1", "a.png", [120]);
    expect(result).toBe("asset:///vault/files/documents/doc-1/assets/asset-1/a.png");
  });
});
