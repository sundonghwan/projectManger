// 메모를 '고정됨'/'기타' 섹션으로 분리. 백엔드가 이미 고정 우선·sort_order 로 정렬해 주므로
// 입력 순서를 보존하며 pinned 플래그로만 나눈다.
import type { Memo } from "./types";

export interface MemoSections {
  pinned: Memo[];
  others: Memo[];
}

export function partitionMemos(memos: Memo[]): MemoSections {
  const pinned: Memo[] = [];
  const others: Memo[] = [];
  for (const m of memos) {
    if (m.pinned) pinned.push(m);
    else others.push(m);
  }
  return { pinned, others };
}
