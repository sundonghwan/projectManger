import { describe, expect, it } from "vitest";
import type { Business } from "./types";
import { businessTypeFallbackColor, businessTypeOptions, normalizeBusinessType } from "./businessTypes";

function business(overrides: Partial<Business>): Business {
  return {
    id: overrides.id ?? "b1",
    name: overrides.name ?? "사업",
    type: overrides.type ?? "철도",
    color: overrides.color,
    description: null,
    status: overrides.status ?? "active",
    sortOrder: overrides.sortOrder ?? 0,
    archivedAt: overrides.archivedAt,
  };
}

describe("businessTypes", () => {
  it("normalizeBusinessType trims custom type labels", () => {
    expect(normalizeBusinessType("  철도  ")).toBe("철도");
    expect(normalizeBusinessType("\t플랫폼\n")).toBe("플랫폼");
  });

  it("businessTypeFallbackColor returns a deterministic color for a type", () => {
    expect(businessTypeFallbackColor("철도")).toBe(businessTypeFallbackColor("철도"));
    expect(businessTypeFallbackColor("")).toBe("#94a3b8");
  });

  it("businessTypeOptions derives active non-archived unique types with counts and first non-empty color", () => {
    const options = businessTypeOptions([
      business({ id: "1", type: " 플랫폼 ", color: "" }),
      business({ id: "2", type: "철도", color: "#123456" }),
      business({ id: "3", type: "플랫폼", color: "#abcdef" }),
      business({ id: "4", type: "철도", color: "#999999", archivedAt: "2026-07-01T00:00:00.000Z" }),
      business({ id: "5", type: "  ", color: "#222222" }),
      business({ id: "6", type: "연구", status: "done" }),
    ]);

    expect(options).toEqual([
      { type: "철도", label: "철도", color: "#123456", count: 1 },
      { type: "플랫폼", label: "플랫폼", color: "#abcdef", count: 2 },
    ]);
  });

  it("businessTypeOptions uses fallback color when a type has no business color", () => {
    const options = businessTypeOptions([
      business({ id: "1", type: "플랫폼", color: null }),
      business({ id: "2", type: "플랫폼" }),
    ]);

    expect(options).toEqual([
      {
        type: "플랫폼",
        label: "플랫폼",
        color: businessTypeFallbackColor("플랫폼"),
        count: 2,
      },
    ]);
  });
});
