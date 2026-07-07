import type { Business, BusinessType } from "./types";

export interface BusinessTypeOption {
  type: BusinessType;
  label: string;
  color: string;
  count: number;
}

const FALLBACK_COLORS = [
  "#3b82f6",
  "#22c55e",
  "#f97316",
  "#8b5cf6",
  "#06b6d4",
  "#e11d48",
  "#84cc16",
  "#f59e0b",
  "#64748b",
];

const LEGACY_TYPE_COLORS: Record<string, string> = {
  si: "#3b82f6",
  internal: "#22c55e",
  ops: "#f97316",
  etc: "#94a3b8",
};

export function normalizeBusinessType(type: string): BusinessType {
  return type.trim();
}

export function businessTypeFallbackColor(type: BusinessType): string {
  const normalized = normalizeBusinessType(type);
  if (!normalized) return "#94a3b8";
  const legacyColor = LEGACY_TYPE_COLORS[normalized];
  if (legacyColor) return legacyColor;

  let hash = 0;
  for (let i = 0; i < normalized.length; i += 1) {
    hash = (hash * 31 + normalized.charCodeAt(i)) >>> 0;
  }
  return FALLBACK_COLORS[hash % FALLBACK_COLORS.length];
}

export function businessTypeOptions(businesses: Business[]): BusinessTypeOption[] {
  const byType = new Map<BusinessType, BusinessTypeOption>();

  for (const business of businesses) {
    if (business.archivedAt || business.status !== "active") continue;

    const type = normalizeBusinessType(business.type);
    if (!type) continue;

    const existing = byType.get(type);
    const businessColor = business.color?.trim();
    if (existing) {
      existing.count += 1;
      if (!existing.color && businessColor) existing.color = businessColor;
      continue;
    }

    byType.set(type, {
      type,
      label: type,
      color: businessColor ?? "",
      count: 1,
    });
  }

  return Array.from(byType.values())
    .map((option) => ({
      ...option,
      color: option.color || businessTypeFallbackColor(option.type),
    }))
    .sort((a, b) => a.label.localeCompare(b.label, "ko-KR"));
}
