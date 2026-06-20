// 드래그 재정렬용 sort_order 계산.
// 인접 두 항목 사이의 중간값을 부여하고, 간격이 소진되면 reindex로 재배치한다.

/** 기본 간격 단위 */
export const STEP = 1;

/** 이 값보다 간격이 좁아지면 정밀도 소진으로 보고 reindex가 필요 */
export const MIN_GAP = 1e-6;

/**
 * 드롭 위치의 앞(before)/뒤(after) 항목 sort_order로 새 값을 계산.
 * - 둘 다 null(빈 목록) → STEP
 * - 앞이 null(맨 앞 삽입) → after - STEP
 * - 뒤가 null(맨 뒤 삽입) → before + STEP
 * - 둘 다 있음 → 중간값
 */
export function computeSortOrder(before: number | null, after: number | null): number {
  if (before == null && after == null) return STEP;
  if (before == null) return (after as number) - STEP;
  if (after == null) return before + STEP;
  return (before + after) / 2;
}

/** 두 경계 사이 간격이 너무 좁은지(재배치 필요) 판단. 한쪽이 없으면 false. */
export function isTooClose(before: number | null, after: number | null): boolean {
  if (before == null || after == null) return false;
  return Math.abs(after - before) < MIN_GAP;
}

/** 표시 순서대로 받은 id 목록에 STEP 간격의 새 sort_order를 부여(정밀도 소진 시 호출). */
export function reindex(idsInOrder: number[]): { id: number; sortOrder: number }[] {
  return idsInOrder.map((id, i) => ({ id, sortOrder: (i + 1) * STEP }));
}
