# Drag Upload and Custom Business Types Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add drag-and-drop deliverable upload, dynamic business type filters, and full sidebar business editing.

**Architecture:** Keep the backend storage model unchanged because Rust already stores business type as a string. Move business type behavior into a small frontend domain helper, then wire Sidebar/CreatePopover/App to string-based type options. Reuse the existing deliverable upload API for dropped file paths.

**Tech Stack:** React, TypeScript, Vitest, Testing Library, Tauri invoke API, existing file-store backend.

---

## File Structure

- Create `src/domain/businessTypes.ts`
  - Normalizes business type strings.
  - Derives filter/create suggestions from `Business[]`.
  - Provides deterministic fallback colors for arbitrary strings.
- Create `src/domain/businessTypes.test.ts`
  - Locks type normalization, de-duplication, archived filtering, and deterministic colors.
- Modify `src/domain/types.ts`
  - Change `BusinessType` from a fixed union to `string`.
- Modify `src/ui/colors.ts`
  - Remove hard dependency on fixed business type maps.
  - Keep `businessColor(type, color)` with string fallback color.
- Modify `src/components/CreatePopover.tsx`
  - Replace fixed business type chips with a free text input and datalist.
- Create `src/components/BusinessEditorPopover.tsx`
  - New compact business edit form for name/type/color/status/description.
- Create `src/components/BusinessEditorPopover.test.tsx`
  - Test save/cancel and validation behavior.
- Modify `src/components/Sidebar.tsx`
  - Render dynamic type filter options.
  - Add business edit button and editor popover.
- Modify `src/components/Sidebar.test.tsx`
  - Update fixed-type tests to dynamic type tests.
  - Add business edit test.
- Modify `src/App.tsx`
  - Use `Set<string>` for type filter.
  - Pass `businessTypeOptions` and full `businesses` to Sidebar.
  - Add `onUpdateBusiness`.
- Modify `src/views/DeliverableList.tsx`
  - Add drag-over/drop state and `onDropFiles(paths)`.
- Modify `src/views/DeliverableList.test.tsx`
  - Test drop path extraction and upload callback.
- Modify `src/views/DeliverablesView.tsx`
  - Wire dropped file paths to `d.upload(paths, selectedFolderId)`.

---

### Task 1: Business Type Domain Helper

**Files:**
- Create: `src/domain/businessTypes.ts`
- Create: `src/domain/businessTypes.test.ts`
- Modify: `src/domain/types.ts`
- Modify: `src/ui/colors.ts`

- [ ] **Step 1: Write the failing helper tests**

Create `src/domain/businessTypes.test.ts`:

```ts
import { describe, expect, it } from "vitest";
import type { Business } from "./types";
import {
  businessTypeFallbackColor,
  businessTypeOptions,
  normalizeBusinessType,
} from "./businessTypes";

const business = (over: Partial<Business> & Pick<Business, "id" | "name" | "type">): Business => ({
  color: null,
  description: null,
  status: "active",
  sortOrder: 1,
  archivedAt: null,
  ...over,
});

describe("businessTypes", () => {
  it("normalizes free-form type labels", () => {
    expect(normalizeBusinessType("  철도  ")).toBe("철도");
    expect(normalizeBusinessType("")).toBe("");
    expect(normalizeBusinessType("   ")).toBe("");
  });

  it("derives unique active business type options with colors", () => {
    const options = businessTypeOptions([
      business({ id: "1", name: "A", type: "철도", color: "#123456" }),
      business({ id: "2", name: "B", type: "철도", color: "#abcdef" }),
      business({ id: "3", name: "C", type: "플랫폼", color: null }),
      business({ id: "6", name: "F", type: "플랫폼", color: "#22c55e" }),
      business({ id: "4", name: "D", type: "보관", archivedAt: "2026-01-01T00:00:00.000Z" }),
      business({ id: "5", name: "E", type: "   " }),
    ]);

    expect(options.map((o) => o.type)).toEqual(["철도", "플랫폼"]);
    expect(options[0]).toMatchObject({ type: "철도", label: "철도", color: "#123456", count: 2 });
    expect(options[1]).toMatchObject({ type: "플랫폼", label: "플랫폼", color: "#22c55e", count: 2 });
  });

  it("fallback color is deterministic for a type string", () => {
    expect(businessTypeFallbackColor("플랫폼")).toBe(businessTypeFallbackColor("플랫폼"));
    expect(businessTypeFallbackColor("플랫폼")).toMatch(/^#[0-9a-f]{6}$/);
  });
});
```

- [ ] **Step 2: Run the failing test**

Run:

```bash
npm test -- src/domain/businessTypes.test.ts
```

Expected: FAIL because `src/domain/businessTypes.ts` does not exist.

- [ ] **Step 3: Implement the helper**

Create `src/domain/businessTypes.ts`:

```ts
import type { Business } from "./types";

export interface BusinessTypeOption {
  type: string;
  label: string;
  color: string;
  count: number;
}

const FALLBACK_COLORS = [
  "#3b82f6",
  "#22c55e",
  "#f97316",
  "#8b5cf6",
  "#14b8a6",
  "#ef4444",
  "#64748b",
  "#f59e0b",
];

export function normalizeBusinessType(type: string): string {
  return type.trim();
}

export function businessTypeFallbackColor(type: string): string {
  const normalized = normalizeBusinessType(type);
  let hash = 0;
  for (let i = 0; i < normalized.length; i += 1) {
    hash = (hash * 31 + normalized.charCodeAt(i)) >>> 0;
  }
  return FALLBACK_COLORS[hash % FALLBACK_COLORS.length];
}

export function businessTypeOptions(businesses: Business[]): BusinessTypeOption[] {
  const map = new Map<string, { type: string; label: string; color: string | null; count: number }>();
  for (const business of businesses) {
    if (business.archivedAt) continue;
    const type = normalizeBusinessType(business.type);
    if (!type) continue;
    const existing = map.get(type);
    if (existing) {
      existing.count += 1;
      if (existing.color == null && business.color) existing.color = business.color;
      continue;
    }
    map.set(type, {
      type,
      label: type,
      color: business.color ?? null,
      count: 1,
    });
  }
  return [...map.values()]
    .map((option) => ({ ...option, color: option.color ?? businessTypeFallbackColor(option.type) }))
    .sort((a, b) => a.label.localeCompare(b.label, "ko"));
}
```

- [ ] **Step 4: Loosen the TypeScript business type**

In `src/domain/types.ts`, replace:

```ts
export type BusinessType = "si" | "internal" | "ops" | "etc";
```

with:

```ts
export type BusinessType = string;
```

In `src/ui/colors.ts`, replace the business type maps and `businessColor` implementation with:

```ts
import type { DeliverableStatus, MemoColor, Priority, TaskStatus } from "../domain/types";
import { businessTypeFallbackColor } from "../domain/businessTypes";

export function businessColor(type: string, color?: string | null): string {
  return color ?? businessTypeFallbackColor(type);
}
```

Keep the existing memo, task, deliverable, and priority exports unchanged.

- [ ] **Step 5: Run the helper tests**

Run:

```bash
npm test -- src/domain/businessTypes.test.ts
```

Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add src/domain/types.ts src/domain/businessTypes.ts src/domain/businessTypes.test.ts src/ui/colors.ts
git commit -m "feat: derive business type options"
```

---

### Task 2: Custom Type Input in CreatePopover

**Files:**
- Modify: `src/components/CreatePopover.tsx`
- Modify: `src/components/Sidebar.tsx`
- Modify: `src/components/Sidebar.test.tsx`

- [ ] **Step 1: Update the failing Sidebar create test**

In `src/components/Sidebar.test.tsx`, replace the existing `"사업 추가 → 유형 선택 + 이름 입력 후 onAddBusiness(type, name)"` test with:

```ts
it("사업 추가 → 사용자 유형 + 이름 입력 후 onAddBusiness(type, name)", async () => {
  const { props } = setup({
    businessTypeOptions: [
      { type: "철도", label: "철도", color: "#3b82f6", count: 1 },
      { type: "플랫폼", label: "플랫폼", color: "#22c55e", count: 1 },
    ],
  });

  await userEvent.click(screen.getByRole("button", { name: "사업 추가" }));
  await userEvent.clear(screen.getByLabelText("사업 유형"));
  await userEvent.type(screen.getByLabelText("사업 유형"), "철도");
  await userEvent.type(screen.getByLabelText("이름"), "신규 사업");
  await userEvent.click(screen.getByRole("button", { name: "만들기" }));

  expect(props.onAddBusiness).toHaveBeenCalledWith("철도", "신규 사업");
});
```

Also update `setup` defaults to include:

```ts
businessTypeOptions: [{ type: "철도", label: "철도", color: "#3b82f6", count: 1 }],
```

- [ ] **Step 2: Run the failing Sidebar test**

Run:

```bash
npm test -- src/components/Sidebar.test.tsx
```

Expected: FAIL because `businessTypeOptions` is not a Sidebar prop and CreatePopover still uses fixed chips.

- [ ] **Step 3: Update CreatePopover props and UI**

In `src/components/CreatePopover.tsx`:

Replace fixed business type imports with:

```ts
import type { BusinessTypeOption } from "../domain/businessTypes";
import { normalizeBusinessType } from "../domain/businessTypes";
```

Update props:

```ts
export interface CreatePopoverProps {
  x: number;
  y: number;
  variant: "business" | "child";
  allowedKinds?: AddKind[];
  businessTypeOptions?: BusinessTypeOption[];
  onCreateBusiness?: (type: string, name: string) => void;
  onCreateChild?: (kind: AddKind, name: string) => void;
  onClose: () => void;
}
```

Initialize type from options:

```ts
const { x, y, variant, allowedKinds = [], businessTypeOptions = [], onCreateBusiness, onCreateChild, onClose } = props;
const [type, setType] = useState(businessTypeOptions[0]?.type ?? "");
```

In `submit`, normalize and reject empty type:

```ts
const submit = () => {
  const n = name.trim();
  const t = normalizeBusinessType(type);
  if (!n) return;
  if (variant === "business") {
    if (!t) return;
    onCreateBusiness?.(t, n);
  } else {
    onCreateChild?.(kind, n);
  }
  onClose();
};
```

Replace the business chips block with:

```tsx
<div style={{ display: "flex", flexDirection: "column", gap: 8 }}>
  <input
    aria-label="사업 유형"
    list="business-type-options"
    placeholder="예: 철도, 플랫폼, 운영"
    value={type}
    onChange={(e) => setType(e.target.value)}
    style={input}
  />
  <datalist id="business-type-options">
    {businessTypeOptions.map((option) => (
      <option key={option.type} value={option.type} />
    ))}
  </datalist>
  {businessTypeOptions.length > 0 && (
    <div style={chips}>
      {businessTypeOptions.map((option) => (
        <button
          key={option.type}
          type="button"
          aria-pressed={option.type === normalizeBusinessType(type)}
          onClick={() => setType(option.type)}
          style={chip(option.type === normalizeBusinessType(type), option.color)}
        >
          <span style={{ width: 7, height: 7, borderRadius: "50%", background: option.color }} />
          {option.label}
        </button>
      ))}
    </div>
  )}
</div>
```

Disable create when business type is empty:

```tsx
const disabled = !name.trim() || (variant === "business" && !normalizeBusinessType(type));
```

Use `disabled` for the create button.

- [ ] **Step 4: Pass type options through Sidebar**

In `src/components/Sidebar.tsx`, import:

```ts
import type { BusinessTypeOption } from "../domain/businessTypes";
```

Add prop:

```ts
businessTypeOptions?: BusinessTypeOption[];
```

Default it in destructuring:

```ts
businessTypeOptions = [],
```

Pass it to `CreatePopover`:

```tsx
businessTypeOptions={businessTypeOptions}
```

- [ ] **Step 5: Run Sidebar tests**

Run:

```bash
npm test -- src/components/Sidebar.test.tsx
```

Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add src/components/CreatePopover.tsx src/components/Sidebar.tsx src/components/Sidebar.test.tsx
git commit -m "feat: create businesses with custom types"
```

---

### Task 3: Dynamic Sidebar Filter

**Files:**
- Modify: `src/components/Sidebar.tsx`
- Modify: `src/components/Sidebar.test.tsx`
- Modify: `src/App.tsx`

- [ ] **Step 1: Update the filter test**

In `src/components/Sidebar.test.tsx`, replace the fixed filter test with:

```ts
it("유형 필터 아이콘 → 동적 유형 칩 클릭 시 onToggleType 호출", async () => {
  const onToggleType = vi.fn();
  setup({
    onToggleType,
    typeFilter: new Set(),
    businessTypeOptions: [
      { type: "철도", label: "철도", color: "#3b82f6", count: 2 },
      { type: "플랫폼", label: "플랫폼", color: "#22c55e", count: 1 },
    ],
  });

  expect(screen.queryByRole("button", { name: /철도/ })).toBeNull();
  await userEvent.click(screen.getByRole("button", { name: "유형 필터" }));
  await userEvent.click(screen.getByRole("button", { name: /철도/ }));

  expect(onToggleType).toHaveBeenCalledWith("철도");
});
```

- [ ] **Step 2: Run the failing filter test**

Run:

```bash
npm test -- src/components/Sidebar.test.tsx
```

Expected: FAIL while Sidebar still reads `TYPE_LABEL`.

- [ ] **Step 3: Update Sidebar filter rendering**

In `src/components/Sidebar.tsx`:

Remove imports of `BusinessType`, `TYPE_COLOR`, and `TYPE_LABEL`.

Change props:

```ts
typeFilter?: Set<string>;
onToggleType?: (type: string) => void;
```

In the reset button, replace the fixed `Object.keys(TYPE_LABEL)` loop with:

```ts
businessTypeOptions.forEach((option) => {
  if (typeFilter?.has(option.type)) onToggleType(option.type);
});
```

Replace fixed filter chip rendering with:

```tsx
{businessTypeOptions.map((option) => {
  const active = typeFilter?.has(option.type) ?? false;
  return (
    <button
      key={option.type}
      onClick={() => onToggleType(option.type)}
      aria-pressed={active}
      style={{
        display: "flex",
        alignItems: "center",
        gap: 5,
        fontSize: 12,
        padding: "3px 9px",
        borderRadius: 20,
        border: `1px solid ${active ? option.color : "var(--border)"}`,
        background: active ? option.color + "22" : "transparent",
        color: active ? option.color : "var(--text2)",
        cursor: "pointer",
      }}
    >
      <span style={{ width: 7, height: 7, borderRadius: "50%", background: option.color }} />
      {option.label}
    </button>
  );
})}
```

If `businessTypeOptions.length === 0`, render:

```tsx
<div style={{ fontSize: 12, color: "var(--text3)" }}>필터링할 유형이 없습니다.</div>
```

- [ ] **Step 4: Wire dynamic options in App**

In `src/App.tsx`, import:

```ts
import { businessTypeOptions } from "./domain/businessTypes";
```

Change:

```ts
const [typeFilter, setTypeFilter] = useState<Set<BusinessType>>(new Set());
```

to:

```ts
const [typeFilter, setTypeFilter] = useState<Set<string>>(new Set());
```

Change `onToggleType` to accept `string`.

Add:

```ts
const typeOptions = useMemo(() => businessTypeOptions(businesses), [businesses]);
```

Pass to Sidebar:

```tsx
businessTypeOptions={typeOptions}
```

- [ ] **Step 5: Run tests**

Run:

```bash
npm test -- src/components/Sidebar.test.tsx src/domain/businessTypes.test.ts
npx tsc --noEmit
```

Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add src/App.tsx src/components/Sidebar.tsx src/components/Sidebar.test.tsx
git commit -m "feat: filter businesses by dynamic types"
```

---

### Task 4: Sidebar Business Editor

**Files:**
- Create: `src/components/BusinessEditorPopover.tsx`
- Create: `src/components/BusinessEditorPopover.test.tsx`
- Modify: `src/components/Sidebar.tsx`
- Modify: `src/components/Sidebar.test.tsx`
- Modify: `src/App.tsx`

- [ ] **Step 1: Write BusinessEditorPopover tests**

Create `src/components/BusinessEditorPopover.test.tsx`:

```tsx
import { describe, expect, it, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { BusinessEditorPopover } from "./BusinessEditorPopover";
import type { Business } from "../domain/types";

const business: Business = {
  id: "b1",
  name: "철도청",
  type: "철도",
  color: "#3b82f6",
  description: "설명",
  status: "active",
  sortOrder: 1,
  archivedAt: null,
};

describe("BusinessEditorPopover", () => {
  it("edits business fields and saves", async () => {
    const onSave = vi.fn();
    render(
      <BusinessEditorPopover
        business={business}
        businessTypeOptions={[{ type: "플랫폼", label: "플랫폼", color: "#22c55e", count: 1 }]}
        onSave={onSave}
        onClose={vi.fn()}
      />,
    );

    await userEvent.clear(screen.getByLabelText("사업 이름"));
    await userEvent.type(screen.getByLabelText("사업 이름"), "통합관제");
    await userEvent.clear(screen.getByLabelText("사업 유형"));
    await userEvent.type(screen.getByLabelText("사업 유형"), "플랫폼");
    await userEvent.clear(screen.getByLabelText("색상"));
    await userEvent.type(screen.getByLabelText("색상"), "#22c55e");
    await userEvent.selectOptions(screen.getByLabelText("상태"), "onhold");
    await userEvent.clear(screen.getByLabelText("설명"));
    await userEvent.type(screen.getByLabelText("설명"), "대기 중");
    await userEvent.click(screen.getByRole("button", { name: "저장" }));

    expect(onSave).toHaveBeenCalledWith({
      id: "b1",
      name: "통합관제",
      type: "플랫폼",
      color: "#22c55e",
      status: "onhold",
      description: "대기 중",
    });
  });

  it("empty name or type disables save", async () => {
    render(
      <BusinessEditorPopover
        business={business}
        businessTypeOptions={[]}
        onSave={vi.fn()}
        onClose={vi.fn()}
      />,
    );

    await userEvent.clear(screen.getByLabelText("사업 유형"));
    expect(screen.getByRole("button", { name: "저장" })).toBeDisabled();
  });
});
```

- [ ] **Step 2: Run the failing component test**

Run:

```bash
npm test -- src/components/BusinessEditorPopover.test.tsx
```

Expected: FAIL because the component does not exist.

- [ ] **Step 3: Implement BusinessEditorPopover**

Create `src/components/BusinessEditorPopover.tsx`:

```tsx
import { useState, type CSSProperties } from "react";
import type { Business, EntityStatus } from "../domain/types";
import type { BusinessTypeOption } from "../domain/businessTypes";
import { normalizeBusinessType } from "../domain/businessTypes";

export interface BusinessEditorInput {
  id: string;
  name: string;
  type: string;
  status: EntityStatus;
  color?: string | null;
  description?: string | null;
}

export interface BusinessEditorPopoverProps {
  business: Business;
  businessTypeOptions: BusinessTypeOption[];
  onSave: (input: BusinessEditorInput) => void;
  onClose: () => void;
}

export function BusinessEditorPopover({ business, businessTypeOptions, onSave, onClose }: BusinessEditorPopoverProps) {
  const [name, setName] = useState(business.name);
  const [type, setType] = useState(business.type);
  const [color, setColor] = useState(business.color ?? "");
  const [status, setStatus] = useState<EntityStatus>(business.status);
  const [description, setDescription] = useState(business.description ?? "");
  const valid = name.trim().length > 0 && normalizeBusinessType(type).length > 0;

  const save = () => {
    if (!valid) return;
    onSave({
      id: business.id,
      name: name.trim(),
      type: normalizeBusinessType(type),
      status,
      color: color.trim() || null,
      description: description.trim() || null,
    });
  };

  return (
    <div style={backdrop} onClick={onClose}>
      <div role="dialog" aria-label="사업 수정" style={box} onClick={(e) => e.stopPropagation()}>
        <div style={title}>사업 수정</div>
        <label style={field}>
          <span style={label}>이름</span>
          <input aria-label="사업 이름" value={name} onChange={(e) => setName(e.target.value)} style={input} />
        </label>
        <label style={field}>
          <span style={label}>유형</span>
          <input
            aria-label="사업 유형"
            list="business-edit-type-options"
            value={type}
            onChange={(e) => setType(e.target.value)}
            style={input}
          />
          <datalist id="business-edit-type-options">
            {businessTypeOptions.map((option) => (
              <option key={option.type} value={option.type} />
            ))}
          </datalist>
        </label>
        <label style={field}>
          <span style={label}>색상</span>
          <input aria-label="색상" value={color} onChange={(e) => setColor(e.target.value)} style={input} />
          <input aria-label="색상 선택" type="color" value={color || "#64748b"} onChange={(e) => setColor(e.target.value)} style={colorInput} />
        </label>
        <label style={field}>
          <span style={label}>상태</span>
          <select aria-label="상태" value={status} onChange={(e) => setStatus(e.target.value as EntityStatus)} style={input}>
            <option value="active">진행중</option>
            <option value="onhold">보류</option>
            <option value="done">완료</option>
          </select>
        </label>
        <label style={field}>
          <span style={label}>설명</span>
          <textarea aria-label="설명" value={description} onChange={(e) => setDescription(e.target.value)} style={{ ...input, minHeight: 64, resize: "vertical" }} />
        </label>
        <div style={actions}>
          <button onClick={onClose} style={secondary}>취소</button>
          <button onClick={save} disabled={!valid} style={{ ...primary, opacity: valid ? 1 : 0.5 }}>저장</button>
        </div>
      </div>
    </div>
  );
}

const backdrop: CSSProperties = { position: "fixed", inset: 0, zIndex: 220, background: "rgba(15,23,42,.18)" };
const box: CSSProperties = { position: "fixed", left: 280, top: 88, width: 300, background: "var(--card)", border: "1px solid var(--border)", borderRadius: "var(--radius-md)", boxShadow: "var(--shadow-popover)", padding: 12 };
const title: CSSProperties = { fontSize: 13, fontWeight: 700, marginBottom: 10 };
const field: CSSProperties = { display: "flex", flexDirection: "column", gap: 4, marginBottom: 8 };
const label: CSSProperties = { fontSize: 11, fontWeight: 600, color: "var(--text2)" };
const input: CSSProperties = { border: "1px solid var(--border)", borderRadius: "var(--radius-md)", background: "var(--input)", color: "var(--text)", padding: "6px 8px", fontSize: 13, fontFamily: "inherit" };
const colorInput: CSSProperties = { width: 44, height: 28, border: "1px solid var(--border)", borderRadius: "var(--radius-sm)", background: "var(--bg)", padding: 2 };
const actions: CSSProperties = { display: "flex", justifyContent: "flex-end", gap: 6, marginTop: 10 };
const secondary: CSSProperties = { border: "1px solid var(--border)", background: "var(--bg)", color: "var(--text2)", borderRadius: "var(--radius-md)", padding: "6px 10px", cursor: "pointer" };
const primary: CSSProperties = { border: "none", background: "var(--accent)", color: "#fff", borderRadius: "var(--radius-md)", padding: "6px 12px", fontWeight: 600, cursor: "pointer" };
```

- [ ] **Step 4: Wire editor through Sidebar**

In `src/components/Sidebar.tsx`, import:

```ts
import type { Business } from "../domain/types";
import { BusinessEditorPopover, type BusinessEditorInput } from "./BusinessEditorPopover";
```

Add props:

```ts
businesses?: Business[];
onUpdateBusiness?: (input: BusinessEditorInput) => void;
```

Add state:

```ts
const [editingBusinessId, setEditingBusinessId] = useState<string | null>(null);
const editingBusiness = businesses.find((b) => b.id === editingBusinessId) ?? null;
```

On business rows, add an edit button before archive:

```tsx
{row.type === "business" && onUpdateBusiness && (
  <button
    aria-label={`${row.label} 수정`}
    title="사업 수정"
    onClick={(e) => {
      e.stopPropagation();
      setEditingBusinessId(row.entityId);
    }}
    style={archiveStyle}
  >
    <Icon name="settings" size={13} />
  </button>
)}
```

Render the popover near the end of Sidebar:

```tsx
{editingBusiness && onUpdateBusiness && (
  <BusinessEditorPopover
    business={editingBusiness}
    businessTypeOptions={businessTypeOptions}
    onSave={(input) => {
      onUpdateBusiness(input);
      setEditingBusinessId(null);
    }}
    onClose={() => setEditingBusinessId(null)}
  />
)}
```

- [ ] **Step 5: Add Sidebar edit test**

In `src/components/Sidebar.test.tsx`, import `Business`.

Add a `business` fixture:

```ts
const business = {
  id: "1",
  name: "SI사업 A",
  type: "철도",
  color: "#3b82f6",
  description: null,
  status: "active",
  sortOrder: 1,
  archivedAt: null,
} satisfies Business;
```

Add default props:

```ts
businesses: [business],
onUpdateBusiness: vi.fn(),
```

Add test:

```ts
it("사업 수정 버튼 → 편집 저장 시 onUpdateBusiness", async () => {
  const { props } = setup();
  await userEvent.click(screen.getByRole("button", { name: "SI사업 A 수정" }));
  await userEvent.clear(screen.getByLabelText("사업 유형"));
  await userEvent.type(screen.getByLabelText("사업 유형"), "플랫폼");
  await userEvent.click(screen.getByRole("button", { name: "저장" }));

  expect(props.onUpdateBusiness).toHaveBeenCalledWith(expect.objectContaining({
    id: "1",
    name: "SI사업 A",
    type: "플랫폼",
  }));
});
```

- [ ] **Step 6: Wire update in App**

In `src/App.tsx`, import:

```ts
import type { BusinessEditorInput } from "./components/BusinessEditorPopover";
```

Add callback:

```ts
const onUpdateBusiness = useCallback(
  async (input: BusinessEditorInput) => {
    try {
      await api.business.update(input);
      await loadBusinesses();
    } catch (e) {
      setError(String(e));
    }
  },
  [loadBusinesses],
);
```

Pass to Sidebar:

```tsx
businesses={businesses}
onUpdateBusiness={onUpdateBusiness}
```

- [ ] **Step 7: Run tests**

Run:

```bash
npm test -- src/components/BusinessEditorPopover.test.tsx src/components/Sidebar.test.tsx
npx tsc --noEmit
```

Expected: PASS.

- [ ] **Step 8: Commit**

```bash
git add src/components/BusinessEditorPopover.tsx src/components/BusinessEditorPopover.test.tsx src/components/Sidebar.tsx src/components/Sidebar.test.tsx src/App.tsx
git commit -m "feat: edit businesses from sidebar"
```

---

### Task 5: Drag-and-Drop Deliverable Upload

**Files:**
- Modify: `src/views/DeliverableList.tsx`
- Modify: `src/views/DeliverableList.test.tsx`
- Modify: `src/views/DeliverablesView.tsx`

- [ ] **Step 1: Write DeliverableList drop tests**

In `src/views/DeliverableList.test.tsx`, extend setup handlers:

```ts
onDropFiles: vi.fn(),
```

Add tests:

```ts
it("파일 드롭 시 onDropFiles(paths)", () => {
  const h = setup();
  const surface = screen.getByTestId("deliverable-drop-zone");
  const file = new File(["pdf"], "보고서.pdf", { type: "application/pdf" }) as File & { path: string };
  file.path = "/tmp/보고서.pdf";

  fireEvent.drop(surface, {
    dataTransfer: {
      files: [file],
    },
  });

  expect(h.onDropFiles).toHaveBeenCalledWith(["/tmp/보고서.pdf"]);
});

it("드래그 중 드롭 안내를 보여준다", () => {
  setup();
  const surface = screen.getByTestId("deliverable-drop-zone");
  fireEvent.dragOver(surface, { dataTransfer: { files: [] } });

  expect(screen.getByText("여기에 파일을 놓아 업로드")).toBeInTheDocument();
});
```

Update imports:

```ts
import { fireEvent, render, screen } from "@testing-library/react";
```

- [ ] **Step 2: Run failing DeliverableList tests**

Run:

```bash
npm test -- src/views/DeliverableList.test.tsx
```

Expected: FAIL because `onDropFiles` and the drop zone do not exist.

- [ ] **Step 3: Implement drop behavior**

In `src/views/DeliverableList.tsx`, update props:

```ts
onDropFiles?: (paths: string[]) => void;
```

Destructure `onDropFiles`.

Add state:

```ts
const [dragActive, setDragActive] = useState(false);
```

Add helper near the component:

```ts
function droppedPaths(files: FileList | File[] | null | undefined): string[] {
  if (!files) return [];
  return Array.from(files)
    .map((file) => (file as File & { path?: string }).path)
    .filter((path): path is string => Boolean(path));
}
```

Wrap the outer component div with drag/drop handlers and test id:

```tsx
<div
  data-testid="deliverable-drop-zone"
  onDragOver={(e) => {
    if (!onDropFiles) return;
    e.preventDefault();
    setDragActive(true);
  }}
  onDragLeave={(e) => {
    if (e.currentTarget === e.target) setDragActive(false);
  }}
  onDrop={(e) => {
    if (!onDropFiles) return;
    e.preventDefault();
    setDragActive(false);
    const paths = droppedPaths(e.dataTransfer.files);
    if (paths.length > 0) onDropFiles(paths);
  }}
  style={{
    display: "flex",
    flexDirection: "column",
    height: "100%",
    minHeight: 0,
    position: "relative",
    outline: dragActive ? "2px dashed var(--accent)" : "none",
    outlineOffset: -8,
  }}
>
```

Add overlay just after the header:

```tsx
{dragActive && (
  <div style={dropOverlay}>
    <Icon name="arrow-up" size={18} />
    <span>여기에 파일을 놓아 업로드</span>
  </div>
)}
```

Add style:

```ts
const dropOverlay: CSSProperties = {
  position: "absolute",
  inset: 10,
  zIndex: 20,
  display: "flex",
  alignItems: "center",
  justifyContent: "center",
  gap: 8,
  border: "1px dashed var(--accent)",
  borderRadius: "var(--radius-md)",
  background: "rgba(59,130,246,.08)",
  color: "var(--accent)",
  fontSize: 13,
  fontWeight: 600,
  pointerEvents: "none",
};
```

- [ ] **Step 4: Wire DeliverablesView**

In `src/views/DeliverablesView.tsx`, add:

```tsx
onDropFiles={(paths) => void d.upload(paths, selectedFolderId)}
```

to `DeliverableList`.

- [ ] **Step 5: Run Deliverable tests**

Run:

```bash
npm test -- src/views/DeliverableList.test.tsx
npx tsc --noEmit
```

Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add src/views/DeliverableList.tsx src/views/DeliverableList.test.tsx src/views/DeliverablesView.tsx
git commit -m "feat: upload deliverables by drag and drop"
```

---

### Task 6: Full Regression and Build Verification

**Files:**
- No source changes expected.

- [ ] **Step 1: Run full TypeScript and test suite**

Run:

```bash
npx tsc --noEmit
npm test
```

Expected:

- TypeScript exits 0.
- Vitest exits 0.
- Existing non-fatal `localStorage` and `DocEditor act(...)` warnings may still appear.

- [ ] **Step 2: Run release build**

Run:

```bash
npm run tauri -- build
```

Expected:

- Web build succeeds.
- Rust release compile succeeds.
- `.app` and `.dmg` bundles are generated under `src-tauri/target/release/bundle`.

- [ ] **Step 3: Verify final git state**

Run:

```bash
git status --short --branch
git log --oneline -8
```

Expected:

- On `codex/dnd-custom-business-types`.
- No unstaged or uncommitted source changes.
- Recent commits match the tasks above.

---

## Self-Review

- Spec coverage: The plan covers drag upload, dynamic business type filtering, and sidebar business editing.
- Placeholder scan: No task contains TBD/TODO/placeholder work. Each task names exact files and commands.
- Type consistency: Business type is consistently treated as `string`; `BusinessTypeOption` carries display metadata; `BusinessEditorInput` is used from Sidebar to App.
- Scope check: The plan intentionally avoids a global type-management model, recursive directory uploads, and backend storage migrations.
