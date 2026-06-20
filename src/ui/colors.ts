// 유형/상태/우선순위 → 색상·라벨 매핑. docs/06-디자인시스템 토큰과 일치.
import type { BusinessType, DeliverableStatus, Priority, TaskStatus } from "../domain/types";

export const TYPE_COLOR: Record<BusinessType, string> = {
  si: "#3b82f6",
  internal: "#22c55e",
  ops: "#f97316",
  etc: "#94a3b8",
};

export const TYPE_LABEL: Record<BusinessType, string> = {
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
  return color ?? TYPE_COLOR[type];
}
