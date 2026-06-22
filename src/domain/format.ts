/** 바이트 수를 사람이 읽기 쉬운 단위로 포맷. null/음수는 "—". */
export function formatBytes(bytes?: number | null): string {
  if (bytes == null || bytes < 0) return "—";
  if (bytes < 1024) return `${bytes} B`;
  const units = ["KB", "MB", "GB", "TB"];
  let v = bytes / 1024;
  let i = 0;
  while (v >= 1024 && i < units.length - 1) {
    v /= 1024;
    i += 1;
  }
  // 10 미만은 소수1자리, 그 이상은 정수
  const s = v < 10 ? v.toFixed(1) : Math.round(v).toString();
  return `${s} ${units[i]}`;
}
