import { describe, it, expect } from "vitest";
import {
  TYPE_COLOR,
  TYPE_LABEL,
  TASK_STATUS_COLOR,
  priorityColor,
  priorityLabel,
  businessColor,
} from "./colors";

describe("colors", () => {
  it("유형별 색상/라벨 매핑", () => {
    expect(TYPE_COLOR.si).toBe("#3b82f6");
    expect(TYPE_LABEL.internal).toBe("내부개발");
  });

  it("태스크 상태 색상 매핑", () => {
    expect(TASK_STATUS_COLOR.doing).toBe("#3b82f6");
    expect(TASK_STATUS_COLOR.done).toBe("#22c55e");
  });

  it("우선순위 색상/라벨", () => {
    expect(priorityColor(4)).toBe("#ef4444");
    expect(priorityLabel(3)).toBe("높음");
    expect(priorityLabel(0)).toBe("없음");
  });

  it("businessColor는 커스텀 색을 우선, 없으면 유형 컬러", () => {
    expect(businessColor("si", "#abcdef")).toBe("#abcdef");
    expect(businessColor("si", null)).toBe("#3b82f6");
    expect(businessColor("ops")).toBe("#f97316");
  });
});
