// 평면 <select> 용 폴더 옵션 빌더 — 루트 폴더 뒤에 그 하위 폴더를 배치(최대 2단계).
// 하위 폴더는 depth=1 로 표시하여 라벨을 들여쓴다.
import type { Folder } from "./types";

export interface FolderOption {
  id: number;
  label: string;
  /** 0=루트, 1=하위 */
  depth: number;
}

const bySort = (a: Folder, b: Folder): number => a.sortOrder - b.sortOrder;

export function folderOptions(folders: Folder[]): FolderOption[] {
  const active = folders.filter((f) => !f.archivedAt);
  const roots = active.filter((f) => f.parentId == null).sort(bySort);
  const out: FolderOption[] = [];
  for (const r of roots) {
    out.push({ id: r.id, label: r.name, depth: 0 });
    const kids = active.filter((f) => f.parentId === r.id).sort(bySort);
    for (const k of kids) out.push({ id: k.id, label: k.name, depth: 1 });
  }
  return out;
}
