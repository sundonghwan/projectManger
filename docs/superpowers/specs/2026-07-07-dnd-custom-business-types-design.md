# Drag Upload and Custom Business Types Design

## Audience

This spec is for Work Vault implementation work after the 1.0 release. It covers three related improvements:

- Upload deliverable files by dragging files into the deliverables screen.
- Replace the hardcoded business type filter with user-defined type strings.
- Add a business edit UI from the sidebar so users can change business name, type, color, status, and description.

## Current State

Deliverable upload already exists in the backend and hook layer:

- `src-tauri/src/commands.rs` exposes `deliverable_upload`.
- `src/api/client.ts` exposes `api.deliverable.upload(businessId, projectId, paths, folderId)`.
- `src/hooks/useDeliverables.ts` exposes `upload(paths, folderId)`.
- `src/views/DeliverablesView.tsx` currently calls upload after an OS file picker.

Business editing is partially present:

- `src-tauri/src/commands.rs` exposes `business_update`.
- `src/api/client.ts` exposes `api.business.update`.
- `src/App.tsx` only uses `api.business.rename` from sidebar inline rename.
- `src/components/Sidebar.tsx` has no full business edit action.

Business type is hardcoded on the frontend:

- `src/domain/types.ts` defines `BusinessType = "si" | "internal" | "ops" | "etc"`.
- `src/ui/colors.ts` defines fixed `TYPE_LABEL` and `TYPE_COLOR` maps.
- `src/components/CreatePopover.tsx` shows the fixed four type chips.
- `src/components/Sidebar.tsx` renders the filter from fixed `TYPE_LABEL` keys.

The Rust model already stores `Business.r#type` as `String`, so the backend can persist custom type names without a storage migration.

## Decision

Use free-form business type strings. Do not create a separate type-management collection yet.

The active type list will be derived from the current non-archived businesses in memory. A user can create a new type while creating or editing a business. Once at least one active business uses that type, the type appears in the sidebar filter. If no active business uses a type, it naturally disappears from the filter.

This keeps the 1.0 follow-up feature small and avoids a separate settings screen. If later users need global type presets, type ordering, or deleting unused types, that can be added as a separate feature.

## Feature 1: Drag-and-Drop Deliverable Upload

The deliverables view will accept dropped files on the whole deliverable list surface.

Behavior:

- Dragging one or more files over the deliverables list shows a restrained drop highlight.
- Dropping files uploads them through the existing `useDeliverables.upload(paths, selectedFolderId)` function.
- If a deliverable folder is selected in the sidebar, dropped files go into that folder, matching the current file-picker behavior.
- Dropping directories is ignored by the backend today, because `deliverable_upload` only accepts paths whose metadata is a regular file. The existing partial-upload error remains the feedback path.
- The file upload button stays as a fallback.

Tauri provides dropped file paths through browser drag events in the desktop WebView, so the implementation should first use `event.dataTransfer.files` and read the `path` property when available. In tests, the component should expose an `onDropFiles(paths)` prop so path extraction can be tested without depending on a real Tauri WebView.

## Feature 2: Dynamic Business Type Filter

The sidebar filter will use type metadata derived from `businesses`, not the hardcoded four-type map.

Metadata rules:

- Type key is the trimmed `business.type` string.
- Display label is the same string.
- Color is the first non-empty `business.color` found for that type.
- If no business color exists for that type, use a deterministic fallback palette based on the type string.
- Filter state remains a `Set<string>`.
- Empty filter means all businesses.

Compatibility rules:

- Existing values `si`, `internal`, `ops`, `etc` remain valid stored strings.
- They are displayed as their raw strings unless we explicitly map legacy labels. For this feature, use raw strings to avoid another hidden mapping.
- Existing businesses can be edited to more meaningful names like `철도`, `플랫폼`, or `운영`.

This removes the misleading hardcoded type list and makes the filter useful immediately.

## Feature 3: Sidebar Business Editing

Business rows need a full edit action, separate from quick inline rename.

UI behavior:

- Keep double-click inline rename for fast name edits.
- Add a small edit button on business rows.
- Opening it shows a compact popover or modal anchored near the sidebar.
- The edit form fields are:
  - Name
  - Type
  - Color
  - Status
  - Description
- Save calls `api.business.update`.
- Cancel closes without changes.
- Archive remains the existing trash button and confirmation flow.

Type editing behavior:

- Type is a text input with datalist suggestions from existing business types.
- Users can type a new value directly.
- Empty type is rejected client-side.

Color editing behavior:

- Color is stored on the business as the existing `color` field.
- Provide a small palette plus a native color input.
- The business dot uses `business.color` first, then deterministic type fallback.

Status editing behavior:

- The backend already accepts `status`.
- Initial options are `active`, `onhold`, and `done`.
- The list view only displays non-archived businesses; status does not hide a business.

## Architecture

Add small frontend helpers instead of broad refactors.

Proposed units:

- `src/domain/businessTypes.ts`
  - Derive unique type options from businesses.
  - Provide deterministic fallback color for a string type.
  - Normalize type input.
- `src/components/BusinessEditorPopover.tsx`
  - Sidebar business editor form.
  - Receives a `Business`, derived type options, and `onSave`.
- `src/components/CreatePopover.tsx`
  - Replace fixed business type chips with type text input and suggestions.
- `src/components/Sidebar.tsx`
  - Accept `businesses` or `businessTypeOptions`.
  - Use dynamic type options in the filter.
  - Expose `onEditBusiness(id/update)` or `onUpdateBusiness`.
  - Add business edit button.
- `src/App.tsx`
  - Change `typeFilter` from `Set<BusinessType>` to `Set<string>`.
  - Create/update businesses using string types.
  - Pass dynamic type options into Sidebar.
- `src/views/DeliverableList.tsx`
  - Add drag-over/drop UI states and an `onDropFiles(paths)` callback.
- `src/views/DeliverablesView.tsx`
  - Wire dropped paths to `d.upload(paths, selectedFolderId)`.

Backend changes should be minimal. Rust already persists `type` as `String`. TypeScript types need to stop using the hardcoded `BusinessType` union for business `type`.

## Data Flow

Business type filter:

1. `App` loads `Business[]`.
2. `businessTypeOptions(businesses)` derives visible filter choices.
3. `Sidebar` renders those choices.
4. Toggling a filter updates `Set<string>`.
5. `visibleBusinesses` filters by `b.type`.

Business edit:

1. User clicks edit on a business row.
2. `BusinessEditorPopover` starts with the selected business fields.
3. Save calls `App.onUpdateBusiness`.
4. `App.onUpdateBusiness` calls `api.business.update`.
5. `App` reloads businesses, preserving the selected row where possible.

Drag upload:

1. User drags files over the deliverables list.
2. The list highlights the drop surface.
3. Drop extracts file paths.
4. `DeliverablesView` passes paths to `useDeliverables.upload`.
5. Existing backend copy/storage flow creates deliverables.

## Error Handling

- Empty business name remains rejected by backend and should also be prevented in the editor.
- Empty business type is prevented in create/edit UI.
- Uploading directories or inaccessible files continues to produce the existing partial-upload message.
- If dropped files do not expose paths, show a short error that this drop source is unsupported and keep the file picker available.
- If business update fails, keep the editor open and surface the error in the app-level error bar.

## Testing

Frontend tests:

- `businessTypes` helper derives unique type options from active businesses and assigns deterministic colors.
- `CreatePopover` creates a business with a custom type string.
- `Sidebar` filter renders dynamic business types and calls `onToggleType` with the selected string.
- `Sidebar` business edit button opens the editor and save calls update with name/type/color/status/description.
- `DeliverableList` calls `onDropFiles` when file paths are dropped and shows active drop state during drag-over.
- `DeliverablesView` wires dropped files to `useDeliverables.upload(paths, selectedFolderId)`.

Existing tests that assumed fixed business types should be updated to the dynamic type model.

Validation commands:

```bash
npx tsc --noEmit
npm test
```

For packaging after implementation:

```bash
npm run tauri -- build
```

## Out of Scope

- A global business type settings screen.
- Type deletion independent of businesses.
- User-defined type ordering.
- Recursive directory upload.
- Cloud upload or remote storage.
- Changing the existing vault file layout.

## Open Questions Resolved

- Business type management mode: use free-form strings per business.
- Filter source: derive from current businesses.
- Drag upload target: deliverables screen, using the current selected deliverable folder.
