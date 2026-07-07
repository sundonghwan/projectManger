import { convertFileSrc as tauriConvertFileSrc } from "@tauri-apps/api/core";
import { api } from "../../api/client";
import type { DocumentAsset } from "../../domain/types";

export function assertSupportedImageName(name: string) {
  const lower = name.toLowerCase();
  if (!/\.(png|jpe?g|gif|webp)$/.test(lower)) {
    throw new Error("지원하지 않는 이미지 형식입니다. PNG, JPG, GIF, WEBP 파일만 사용할 수 있습니다.");
  }
}

export function createDocumentImageUploader(deps: {
  documentId: string;
  uploadAsset?: (documentId: string, fileName: string, bytes: number[]) => Promise<DocumentAsset>;
  convertFileSrc?: (path: string) => string;
}) {
  const uploadAsset = deps.uploadAsset ?? api.document.uploadAsset;
  const convertFileSrc = deps.convertFileSrc ?? tauriConvertFileSrc;

  return async (file: File) => {
    assertSupportedImageName(file.name);
    const bytes = await fileToBytes(file);
    const asset = await uploadAsset(deps.documentId, file.name, bytes);
    return convertFileSrc(asset.filePath || asset.url);
  };
}

function fileToBytes(file: File): Promise<number[]> {
  return new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.onerror = () => reject(new Error("이미지 파일을 읽을 수 없습니다."));
    reader.onload = () => {
      if (!(reader.result instanceof ArrayBuffer)) {
        reject(new Error("이미지 파일을 바이트로 변환할 수 없습니다."));
        return;
      }
      resolve(Array.from(new Uint8Array(reader.result)));
    };
    reader.readAsArrayBuffer(file);
  });
}
