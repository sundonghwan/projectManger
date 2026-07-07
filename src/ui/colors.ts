// 유형/상태/우선순위 → 색상·라벨 매핑. docs/06-디자인시스템 토큰과 일치.
import type { BusinessType, DeliverableStatus, MemoColor, Priority, TaskStatus } from "../domain/types";
import { businessTypeFallbackColor } from "../domain/businessTypes";

/** 메모 색상 선택 목록 (default = 기본 카드색) */
export const MEMO_COLORS: MemoColor[] = [
  "default",
  "red",
  "orange",
  "yellow",
  "green",
  "teal",
  "blue",
  "purple",
  "gray",
];

/** 메모 카드 배경 CSS 값. default/null 은 기본 카드색. */
export function memoBg(color?: MemoColor | null): string {
  return color && color !== "default" ? `var(--memo-${color})` : "var(--card)";
}

export const TYPE_COLOR: Record<string, string> = {
  si: "#3b82f6",
  internal: "#22c55e",
  ops: "#f97316",
  etc: "#94a3b8",
};

export const TYPE_LABEL: Record<string, string> = {
  si: "SI",
  internal: "내부개발",
  ops: "운영",
  etc: "기타",
};

export const TASK_STATUS_COLOR: Record<TaskStatus, string> = {
  todo: "#94a3b8",
  doing: "#3b82f6",
  review: "#f59e0b",
  done: "#22c55e",
};

export const TASK_STATUS_LABEL: Record<TaskStatus, string> = {
  todo: "Todo",
  doing: "Doing",
  review: "Review",
  done: "Done",
};

export const DELIVERABLE_STATUS_COLOR: Record<DeliverableStatus, string> = {
  draft: "#94a3b8",
  review: "#f59e0b",
  done: "#22c55e",
};

export const DELIVERABLE_STATUS_LABEL: Record<DeliverableStatus, string> = {
  draft: "작성중",
  review: "검토",
  done: "완료",
};

const PRIORITY_COLOR: Record<Priority, string> = {
  0: "transparent",
  1: "#cbd5e1",
  2: "#94a3b8",
  3: "#f97316",
  4: "#ef4444",
};
const PRIORITY_LABEL: Record<Priority, string> = {
  0: "없음",
  1: "낮음",
  2: "보통",
  3: "높음",
  4: "긴급",
};

export const priorityColor = (p: Priority): string => PRIORITY_COLOR[p];
export const priorityLabel = (p: Priority): string => PRIORITY_LABEL[p];

/** 사업의 표시 색상: 커스텀 color 우선, 없으면 유형 컬러 */
export function businessColor(type: BusinessType, color?: string | null): string {
  const customColor = color?.trim();
  if (customColor) return customColor;
  return TYPE_COLOR[type] ?? businessTypeFallbackColor(type);
}
