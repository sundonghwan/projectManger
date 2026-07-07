# Work Vault Live Block Document Editor Design

## Audience

This design is for the Work Vault maintainer implementing the second document-editor scope: table editing, image upload, block drag ordering, Notion-style slash commands, and collaborative editing.

## Objective

Replace the current textarea plus Markdown preview document editor with a block-based live editor while preserving Markdown as the user-facing saved/exported document body.

## Current State

The current editor lives in `src/views/DocEditor.tsx`.

- It loads the latest document through `api.document.get(document.id)`.
- It edits a single Markdown string in a `textarea`.
- It saves through `api.document.setBody(document.id, text)` with a 600 ms debounce and flush-on-blur/unmount.
- It renders preview HTML with `marked` and `DOMPurify`.
- Tests in `src/views/DocEditor.test.tsx` cover Markdown loading, preview rendering, sanitization, and blur save.

This shape is correct for plain Markdown, but it cannot support the requested second scope without becoming a fragile custom editor.

## Requirements

The completed feature must support:

- live document editing without a separate Markdown preview toggle
- table insertion and editing
- image upload from the local machine into the Work Vault document
- block-level drag ordering for rich blocks
- slash command insertion for common Notion-like blocks
- collaborative editing architecture
- continued Markdown persistence for compatibility and export
- existing document list, open, rename, search, and archive behavior

## Chosen Approach

Use BlockNote as the document editor engine.

BlockNote is a React block editor that provides ready-made block UI, built-in table/image/file blocks, drag-based block editing, slash-menu-oriented block insertion, Markdown interoperability, and Yjs collaboration integration. This maps directly to the requested feature set with less custom ProseMirror surface area than building on Tiptap directly.

Work Vault will store two document representations:

1. `body`: Markdown string
   - remains the compatibility/export/search body
   - continues to satisfy the user's "save as Markdown" requirement
   - generated from the editor on each save

2. `blocks`: BlockNote JSON string
   - becomes the lossless editor source
   - preserves table settings, block IDs, image metadata, ordering, and editor-specific properties
   - used first when reopening a document

Markdown alone is not sufficient for the full requested scope because complex tables, block IDs, image display metadata, nested block properties, and collaborative state can be lossy when converted to Markdown.

## Data Model

Extend the document model with optional editor metadata.

Frontend `Document`:

```ts
interface Document {
  body: string;
  editorBody?: string | null;
  editorBodyFormat?: "blocknote-json" | null;
  collaborationState?: string | null;
}
```

Rust store `Document`:

```rust
pub struct Document {
    pub body: String,
    pub editor_body: Option<String>,
    pub editor_body_format: Option<String>,
    pub collaboration_state: Option<String>,
}
```

Compatibility rules:

- Existing documents without `editorBody` open by converting `body` Markdown into BlockNote blocks.
- New saves write both Markdown `body` and lossless `editorBody`.
- Search continues to use `body`, so existing search behavior remains simple and portable.
- If BlockNote JSON cannot be parsed, the editor falls back to Markdown import from `body` and shows a recoverable error.

## Frontend Components

Replace the current `DocEditor` internals with a shell plus focused helpers.

### `src/views/DocEditor.tsx`

Responsibilities:

- load the latest document
- own save state and error state
- call the block editor with loaded content
- debounce and flush saves
- keep the same title/topbar footprint as the current UI

### `src/views/document-editor/BlockDocumentEditor.tsx`

Responsibilities:

- create the BlockNote editor instance
- load `editorBody` when present
- import Markdown when only `body` exists
- emit both Markdown and BlockNote JSON on change
- configure tables, image upload, drag behavior, slash menu, and collaboration hooks

### `src/views/document-editor/documentBody.ts`

Responsibilities:

- parse BlockNote JSON defensively
- serialize blocks to JSON
- convert blocks to Markdown
- convert Markdown to initial blocks
- provide deterministic fallback behavior for tests

### `src/views/document-editor/documentAssets.ts`

Responsibilities:

- upload or copy selected image files through Tauri commands
- return an editor-displayable URL
- reject unsupported file types with a clear message

### `src/views/document-editor/documentCollaboration.ts`

Responsibilities:

- create a Yjs document per Work Vault document id
- provide a stable room/document key
- persist collaboration snapshots when available
- keep provider choice isolated so local-only and server-backed collaboration can be swapped without changing the editor component

## Backend Commands

Add document save/load support for editor metadata.

Recommended frontend API:

```ts
api.document.setEditorBody(id, {
  body: markdown,
  editorBody: JSON.stringify(blocks),
  editorBodyFormat: "blocknote-json",
  collaborationState,
});
```

Recommended Rust command:

```rust
document_set_editor_body(id, body, editor_body, editor_body_format, collaboration_state)
```

Image upload should use a document-scoped asset command:

```rust
document_asset_upload(document_id, path) -> DocumentAsset
```

The command copies the source file into the Work Vault store root under a document asset directory, then returns a URL/path that the Tauri webview can render.

Suggested storage layout:

```text
<store-root>/files/documents/<document-id>/assets/<asset-id>/<filename>
```

## Collaboration Scope

The first implementation must add the collaboration architecture, not pretend that local JSON saves are full multi-user collaboration.

Minimum acceptable collaboration implementation:

- BlockNote configured through its Yjs integration boundary
- document id used as the collaboration room/document key
- Yjs state/snapshot persistence designed into the save model
- local provider or isolated provider adapter implemented behind `documentCollaboration.ts`
- UI status that can distinguish local editing from connected collaboration when a provider is available

Full network collaboration requires a provider. Candidate providers include `y-websocket`, Hocuspocus, Liveblocks, PartyKit, or another self-hosted server. The implementation must keep this provider swappable.

## Slash Command Scope

"Notion-style slash command 전체" is interpreted as the complete practical block menu for Work Vault v1:

- paragraph
- heading levels
- bullet list
- numbered list
- checklist
- quote
- code block
- table
- image
- file
- divider/horizontal rule if supported by the selected schema

AI commands, database views, calendars, external embeds, and synced blocks are excluded from this implementation scope.

## Error Handling

- If latest document load fails, show the existing error bar and keep the editor disabled.
- If BlockNote JSON parse fails, fall back to Markdown import from `body`, keep the document editable, and show a warning.
- If image upload fails, keep the editor content unchanged and show a clear upload error.
- If Markdown export fails, do not mark the document as saved.
- If collaboration provider connection fails, continue local editing and show collaboration as disconnected.

## Testing Strategy

Use TDD for behavior that can be tested in Vitest/jsdom.

Frontend tests:

- existing Markdown-only documents load into the live editor path
- editor changes call the save API with both Markdown and BlockNote JSON
- broken `editorBody` falls back to Markdown body
- image upload rejects unsupported files
- image upload calls the asset API and returns a URL
- save flushes on blur/unmount

Backend Rust tests:

- document metadata fields round-trip through store serialization
- `document_set_editor_body` updates Markdown and editor metadata together
- document asset upload stores files under the document asset root
- asset upload rejects missing or invalid paths

Manual verification:

- create a document
- type normal text and Korean text
- insert and edit a table
- upload an image and reopen the document
- create blocks from slash commands
- drag blocks to reorder and reopen
- confirm Markdown body still updates
- run `npm test`
- run `npm run build`
- run `npm run tauri -- build` before packaging

## Alternatives Considered

### Keep textarea and improve Markdown preview

This preserves the current code, but it does not support table editing, block drag ordering, slash commands, or collaboration in a credible way. It is rejected for this scope.

### Build directly on Tiptap

Tiptap is powerful and flexible, but Work Vault would need to assemble more UI: slash menu, drag handles, table menus, upload handling, serialization, and collaboration behavior. It is viable but slower and riskier for this requested scope.

### Use BlockNote

BlockNote is the closest fit because the requested features are aligned with its native abstraction. The main trade-off is that Markdown conversion can be lossy, so Work Vault must save BlockNote JSON alongside Markdown.

## Risks

- BlockNote package size can increase the app bundle.
- Markdown export is lossy for some advanced editor features.
- Tauri asset URL rendering must respect the existing CSP in `src-tauri/tauri.conf.json`.
- jsdom may not fully exercise rich editor drag behavior; some validation will remain manual or browser-level.
- Real multi-user collaboration needs an actual provider/server decision beyond local editing.

## Open Questions

- Which network collaboration provider should Work Vault use for production multi-user editing?
- Should document assets be shown in the existing deliverables area, or remain private to documents?
- Should users be able to open a raw Markdown source mode for recovery/debugging?

For the first implementation, use local document assets and keep raw Markdown source mode out of the default UI unless needed for recovery.
