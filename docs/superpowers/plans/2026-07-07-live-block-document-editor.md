# Live Block Document Editor Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace Work Vault's Markdown textarea document editor with a BlockNote-based live block editor that supports tables, image upload, block drag ordering, slash commands, collaboration architecture, and external Markdown sharing.

**Architecture:** Store BlockNote JSON as the lossless internal editor source while keeping `Document.body` as the Markdown export/share/search body. Add document asset upload commands for images and isolate collaboration behind a Yjs adapter so a real provider can be added without rewriting the editor.

**Tech Stack:** React 19, Tauri v2, Rust store commands, BlockNote React/Mantine, Yjs, y-webrtc, Vitest, Rust unit tests.

---

## File Structure

- Modify `package.json` and `package-lock.json`
  - Add BlockNote and Yjs dependencies.
- Modify `src/domain/types.ts`
  - Add optional document editor metadata and `DocumentAsset`.
- Modify `src/api/client.ts` and `src/api/client.test.ts`
  - Add `document.setEditorBody` and `document.uploadAsset`.
- Modify `src-tauri/src/store/model.rs`
  - Add optional `editor_body`, `editor_body_format`, and `collaboration_state`.
- Modify `src-tauri/src/store/ops/document.rs`
  - Add editor-body save operation and document asset upload helper tests.
- Modify `src-tauri/src/commands.rs`
  - Add Tauri commands for editor-body save and document image asset upload.
- Modify `src-tauri/src/lib.rs`
  - Register new Tauri commands.
- Modify `src-tauri/tauri.conf.json`
  - Enable asset protocol scope for document images if the build requires it.
- Create `src/views/document-editor/documentBody.ts`
  - Testable helpers for editor JSON parsing, Markdown fallback, save payload preparation, and Markdown sharing.
- Create `src/views/document-editor/documentAssets.ts`
  - Testable helpers for image MIME validation and asset upload URL conversion.
- Create `src/views/document-editor/documentCollaboration.ts`
  - Yjs document creation and y-webrtc provider adapter boundary.
- Create `src/views/document-editor/BlockDocumentEditor.tsx`
  - BlockNote UI integration.
- Modify `src/views/DocEditor.tsx`
  - Keep the Work Vault document shell, replace textarea/preview with block editor.
- Modify `src/views/DocEditor.test.tsx`
  - Replace preview tests with live editor load/save/fallback tests.
- Add focused tests under `src/views/document-editor/*.test.ts`.

## Task 1: Install Editor Dependencies

**Files:**
- Modify: `package.json`
- Modify: `package-lock.json`

- [ ] **Step 1: Install dependencies**

Run:

```bash
npm install @blocknote/core @blocknote/react @blocknote/mantine yjs y-webrtc
```

Expected:

- `package.json` contains the five dependencies.
- `package-lock.json` is updated.

- [ ] **Step 2: Build type graph**

Run:

```bash
npx tsc --noEmit
```

Expected:

- It may fail because code has not integrated the packages yet.
- Dependency resolution should not fail with missing package errors.

- [ ] **Step 3: Commit dependency change**

```bash
git add package.json package-lock.json
git commit -m "chore: add block editor dependencies"
```

## Task 2: Extend Document Types and API Client

**Files:**
- Modify: `src/domain/types.ts`
- Modify: `src/api/client.ts`
- Modify: `src/api/client.test.ts`

- [ ] **Step 1: Write failing API tests**

Add tests to `src/api/client.test.ts`:

```ts
describe("api.document editor body", () => {
  it("saves markdown and block editor metadata together", async () => {
    await api.document.setEditorBody("doc-1", {
      body: "# 공유용",
      editorBody: "[{\"type\":\"paragraph\",\"content\":\"공유용\"}]",
      editorBodyFormat: "blocknote-json",
      collaborationState: "snapshot",
    });

    expect(invoke).toHaveBeenCalledWith("document_set_editor_body", {
      id: "doc-1",
      body: "# 공유용",
      editorBody: "[{\"type\":\"paragraph\",\"content\":\"공유용\"}]",
      editorBodyFormat: "blocknote-json",
      collaborationState: "snapshot",
    });
  });

  it("uploads a document asset", async () => {
    await api.document.uploadAsset("doc-1", "image.png", [1, 2, 3]);

    expect(invoke).toHaveBeenCalledWith("document_asset_upload", {
      documentId: "doc-1",
      fileName: "image.png",
      bytes: [1, 2, 3],
    });
  });
});
```

- [ ] **Step 2: Verify tests fail**

Run:

```bash
npm test -- src/api/client.test.ts
```

Expected:

- FAIL because `setEditorBody` and `uploadAsset` do not exist.

- [ ] **Step 3: Add TypeScript types**

Add to `src/domain/types.ts`:

```ts
export type DocumentEditorBodyFormat = "blocknote-json";

export interface DocumentAsset {
  id: string;
  documentId: string;
  fileName: string;
  filePath: string;
  url: string;
}
```

Extend `Document`:

```ts
export interface Document {
  id: string;
  businessId: string;
  projectId?: string | null;
  folderId?: string | null;
  title: string;
  icon?: string | null;
  body: string;
  editorBody?: string | null;
  editorBodyFormat?: DocumentEditorBodyFormat | null;
  collaborationState?: string | null;
  blocks: Block[];
  sortOrder: number;
  archivedAt?: Timestamp | null;
  createdAt: Timestamp;
}
```

- [ ] **Step 4: Add API client methods**

Add imports/types in `src/api/client.ts`:

```ts
import type { DocumentAsset, DocumentEditorBodyFormat } from "../domain/types";

export interface DocumentEditorBodyInput {
  body: string;
  editorBody: string;
  editorBodyFormat: DocumentEditorBodyFormat;
  collaborationState?: string | null;
}
```

Add methods inside `api.document`:

```ts
setEditorBody: (id: string, input: DocumentEditorBodyInput) =>
  invoke<void>("document_set_editor_body", {
    id,
    body: input.body,
    editorBody: input.editorBody,
    editorBodyFormat: input.editorBodyFormat,
    collaborationState: input.collaborationState ?? null,
  }),
uploadAsset: (documentId: string, fileName: string, bytes: number[]) =>
  invoke<DocumentAsset>("document_asset_upload", { documentId, fileName, bytes }),
```

- [ ] **Step 5: Verify API tests pass**

Run:

```bash
npm test -- src/api/client.test.ts
```

Expected:

- PASS.

- [ ] **Step 6: Commit**

```bash
git add src/domain/types.ts src/api/client.ts src/api/client.test.ts
git commit -m "feat: add document editor metadata API"
```

## Task 3: Extend Rust Document Storage

**Files:**
- Modify: `src-tauri/src/store/model.rs`
- Modify: `src-tauri/src/store/ops/document.rs`

- [ ] **Step 1: Write failing Rust tests**

Add tests to `src-tauri/src/store/ops/document.rs` test module:

```rust
#[test]
fn set_editor_body_updates_markdown_and_editor_metadata() {
    let (mut s, biz, _) = setup();
    let d = create(&mut s, &biz, None, None, "문서").unwrap();

    set_editor_body(
        &mut s,
        &d.id,
        "# 공유용",
        Some(r#"[{"type":"paragraph","content":"공유용"}]"#),
        Some("blocknote-json"),
        Some("snapshot"),
    )
    .unwrap();

    let updated = get(&s, &d.id).unwrap();
    assert_eq!(updated.body, "# 공유용");
    assert_eq!(updated.editor_body.as_deref(), Some(r#"[{"type":"paragraph","content":"공유용"}]"#));
    assert_eq!(updated.editor_body_format.as_deref(), Some("blocknote-json"));
    assert_eq!(updated.collaboration_state.as_deref(), Some("snapshot"));
}

#[test]
fn set_editor_body_rejects_unknown_format() {
    let (mut s, biz, _) = setup();
    let d = create(&mut s, &biz, None, None, "문서").unwrap();

    let result = set_editor_body(&mut s, &d.id, "body", Some("{}"), Some("html"), None);

    assert!(result.is_err());
}
```

- [ ] **Step 2: Verify tests fail**

Run:

```bash
cd src-tauri && cargo test store::ops::document::tests::set_editor_body --lib
```

Expected:

- FAIL because fields and `set_editor_body` are missing.

- [ ] **Step 3: Add model fields**

In `src-tauri/src/store/model.rs`, extend `Document`:

```rust
pub editor_body: Option<String>,
pub editor_body_format: Option<String>,
pub collaboration_state: Option<String>,
```

Initialize new documents in `ops/document.rs`:

```rust
editor_body: None,
editor_body_format: None,
collaboration_state: None,
```

- [ ] **Step 4: Add store operation**

Add to `src-tauri/src/store/ops/document.rs`:

```rust
const EDITOR_FORMAT_BLOCKNOTE_JSON: &str = "blocknote-json";

pub fn set_editor_body(
    store: &mut Store,
    id: &str,
    body: &str,
    editor_body: Option<&str>,
    editor_body_format: Option<&str>,
    collaboration_state: Option<&str>,
) -> Result<()> {
    if let Some(format) = editor_body_format {
        if format != EDITOR_FORMAT_BLOCKNOTE_JSON {
            return Err(AppError::Invalid("지원하지 않는 문서 에디터 형식입니다".into()));
        }
    }

    let mut d = get(store, id)?;
    d.body = body.to_string();
    d.editor_body = editor_body.map(|value| value.to_string());
    d.editor_body_format = editor_body_format.map(|value| value.to_string());
    d.collaboration_state = collaboration_state.map(|value| value.to_string());
    d.updated_at = now();
    store.documents.put(d)?;
    Ok(())
}
```

- [ ] **Step 5: Verify Rust tests pass**

Run:

```bash
cd src-tauri && cargo test store::ops::document::tests::set_editor_body --lib
```

Expected:

- PASS.

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/store/model.rs src-tauri/src/store/ops/document.rs
git commit -m "feat: persist document editor metadata"
```

## Task 4: Add Document Asset Upload Backend

**Files:**
- Modify: `src-tauri/src/commands.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Write failing Rust tests**

Add command-helper tests in `src-tauri/src/commands.rs` test module:

```rust
#[test]
fn document_asset_files_root_lives_under_store_root() {
    let store_root = std::env::temp_dir().join("work-vault-store");
    assert_eq!(
        super::document_asset_files_root(&store_root, "doc-1"),
        store_root.join("files").join("documents").join("doc-1").join("assets")
    );
}

#[test]
fn is_supported_document_image_accepts_common_images() {
    assert!(super::is_supported_document_image("a.png"));
    assert!(super::is_supported_document_image("a.jpg"));
    assert!(super::is_supported_document_image("a.jpeg"));
    assert!(super::is_supported_document_image("a.gif"));
    assert!(super::is_supported_document_image("a.webp"));
    assert!(!super::is_supported_document_image("a.pdf"));
}
```

- [ ] **Step 2: Verify tests fail**

Run:

```bash
cd src-tauri && cargo test commands::tests::document_asset --lib
```

Expected:

- FAIL because helper functions do not exist.

- [ ] **Step 3: Add response type and helpers**

Add to `src-tauri/src/commands.rs` imports:

```rust
use serde::{Deserialize, Serialize};
```

If `Deserialize` is already imported separately, change it to the combined import.

Add near deliverable file helpers:

```rust
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentAsset {
    pub id: String,
    pub document_id: String,
    pub file_name: String,
    pub file_path: String,
    pub url: String,
}

fn document_asset_files_root(store_root: &Path, document_id: &str) -> PathBuf {
    store_root.join("files").join("documents").join(document_id).join("assets")
}

fn is_supported_document_image(file_name: &str) -> bool {
    let lower = file_name.to_ascii_lowercase();
    lower.ends_with(".png")
        || lower.ends_with(".jpg")
        || lower.ends_with(".jpeg")
        || lower.ends_with(".gif")
        || lower.ends_with(".webp")
}
```

- [ ] **Step 4: Add upload command**

Add to `src-tauri/src/commands.rs`:

```rust
#[tauri::command]
pub fn document_asset_upload(
    state: State<AppState>,
    document_id: String,
    file_name: String,
    bytes: Vec<u8>,
) -> Result<DocumentAsset> {
    let store = state.store.lock().unwrap();
    let document = ops::document::get(&store, &document_id)?;
    let file_name = Path::new(&file_name)
        .file_name()
        .and_then(|value| value.to_str())
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| AppError::Invalid("이미지 파일 이름을 확인할 수 없습니다".into()))?
        .to_string();

    if !is_supported_document_image(&file_name) {
        return Err(AppError::Invalid("지원하지 않는 이미지 형식입니다".into()));
    }
    if bytes.is_empty() {
        return Err(AppError::Invalid("빈 이미지 파일은 업로드할 수 없습니다".into()));
    }

    let asset_id = crate::store::ids::new_id();
    let dest_dir = document_asset_files_root(&store.root, &document.id).join(&asset_id);
    std::fs::create_dir_all(&dest_dir)
        .map_err(|_| AppError::Invalid("문서 이미지 폴더를 만들 수 없습니다".into()))?;
    let dest = dest_dir.join(&file_name);
    std::fs::write(&dest, bytes)
        .map_err(|_| AppError::Invalid("문서 이미지를 저장할 수 없습니다".into()))?;

    let file_path = dest.to_string_lossy().to_string();
    Ok(DocumentAsset {
        id: asset_id,
        document_id,
        file_name,
        url: file_path.clone(),
        file_path,
    })
}
```

- [ ] **Step 5: Register command**

Add to `src-tauri/src/lib.rs` handler list:

```rust
commands::document_asset_upload,
```

- [ ] **Step 6: Verify Rust tests pass**

Run:

```bash
cd src-tauri && cargo test commands::tests::document_asset --lib
```

Expected:

- PASS.

- [ ] **Step 7: Commit**

```bash
git add src-tauri/src/commands.rs src-tauri/src/lib.rs
git commit -m "feat: add document image asset upload"
```

## Task 5: Add Tauri Command for Editor Body Save

**Files:**
- Modify: `src-tauri/src/commands.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Add Tauri command**

Add:

```rust
#[tauri::command]
pub fn document_set_editor_body(
    state: State<AppState>,
    id: String,
    body: String,
    editor_body: String,
    editor_body_format: String,
    collaboration_state: Option<String>,
) -> Result<()> {
    let mut store = state.store.lock().unwrap();
    ops::document::set_editor_body(
        &mut store,
        &id,
        &body,
        Some(&editor_body),
        Some(&editor_body_format),
        collaboration_state.as_deref(),
    )
}
```

- [ ] **Step 2: Register command**

Add to `src-tauri/src/lib.rs` handler list:

```rust
commands::document_set_editor_body,
```

- [ ] **Step 3: Verify compile**

Run:

```bash
cd src-tauri && cargo test store::ops::document::tests::set_editor_body commands::tests::document_asset --lib
```

Expected:

- PASS.

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/commands.rs src-tauri/src/lib.rs
git commit -m "feat: expose document editor save command"
```

## Task 6: Enable Tauri Asset Rendering for Document Images

**Files:**
- Modify: `src-tauri/tauri.conf.json`

- [ ] **Step 1: Add asset protocol configuration**

Modify `app.security` in `src-tauri/tauri.conf.json`:

```json
{
  "csp": "default-src 'self'; img-src 'self' data: asset: http://asset.localhost; style-src 'self' 'unsafe-inline'; script-src 'self'; connect-src 'self' ipc: http://ipc.localhost; font-src 'self'",
  "assetProtocol": {
    "enable": true,
    "scope": ["**"]
  }
}
```

Rationale:

- Work Vault can store the vault under a user-selected iCloud path, not only under `$APPDATA`.
- The renderer should only insert asset URLs returned by `document_asset_upload`; do not expose a raw path entry UI for arbitrary files.

- [ ] **Step 2: Verify Tauri config parses**

Run:

```bash
npm run tauri -- info
```

Expected:

- The config is accepted without schema errors.

- [ ] **Step 3: Commit**

```bash
git add src-tauri/tauri.conf.json
git commit -m "chore: enable document image asset rendering"
```

## Task 7: Add Testable Document Body Helpers

**Files:**
- Create: `src/views/document-editor/documentBody.ts`
- Create: `src/views/document-editor/documentBody.test.ts`

- [ ] **Step 1: Write failing tests**

Create `src/views/document-editor/documentBody.test.ts`:

```ts
import { describe, expect, it } from "vitest";
import { buildInitialEditorSource, parseEditorBody, prepareEditorSavePayload } from "./documentBody";

describe("documentBody", () => {
  it("prefers valid BlockNote JSON over Markdown", () => {
    const parsed = buildInitialEditorSource({
      body: "# 공유용",
      editorBody: '[{"type":"paragraph","content":"원본"}]',
      editorBodyFormat: "blocknote-json",
    });

    expect(parsed.kind).toBe("blocks");
    expect(parsed.warning).toBeNull();
  });

  it("falls back to Markdown when BlockNote JSON is broken", () => {
    const parsed = buildInitialEditorSource({
      body: "# 공유용",
      editorBody: "{broken",
      editorBodyFormat: "blocknote-json",
    });

    expect(parsed.kind).toBe("markdown");
    expect(parsed.markdown).toBe("# 공유용");
    expect(parsed.warning).toContain("블록 문서");
  });

  it("prepares Markdown as the external sharing body", () => {
    const payload = prepareEditorSavePayload({
      markdown: "# 공유용\n\n![이미지](asset://image.png)",
      blocks: [{ type: "paragraph", content: "공유용" }],
      collaborationState: null,
    });

    expect(payload.body).toContain("# 공유용");
    expect(payload.editorBodyFormat).toBe("blocknote-json");
    expect(JSON.parse(payload.editorBody)).toEqual([{ type: "paragraph", content: "공유용" }]);
  });

  it("rejects non-array editor JSON", () => {
    expect(parseEditorBody("{}")).toBeNull();
  });
});
```

- [ ] **Step 2: Verify tests fail**

Run:

```bash
npm test -- src/views/document-editor/documentBody.test.ts
```

Expected:

- FAIL because helper module does not exist.

- [ ] **Step 3: Implement helpers**

Create `src/views/document-editor/documentBody.ts`:

```ts
import type { DocumentEditorBodyInput } from "../../api/client";

export type EditorSource =
  | { kind: "blocks"; blocks: unknown[]; markdown: string; warning: null }
  | { kind: "markdown"; blocks: null; markdown: string; warning: string | null };

export function parseEditorBody(editorBody?: string | null): unknown[] | null {
  if (!editorBody) return null;
  try {
    const parsed = JSON.parse(editorBody);
    return Array.isArray(parsed) ? parsed : null;
  } catch {
    return null;
  }
}

export function buildInitialEditorSource(document: {
  body?: string | null;
  editorBody?: string | null;
  editorBodyFormat?: string | null;
}): EditorSource {
  if (document.editorBodyFormat === "blocknote-json" && document.editorBody) {
    const blocks = parseEditorBody(document.editorBody);
    if (blocks) {
      return { kind: "blocks", blocks, markdown: document.body ?? "", warning: null };
    }
    return {
      kind: "markdown",
      blocks: null,
      markdown: document.body ?? "",
      warning: "블록 문서를 읽지 못해 Markdown 본문으로 열었습니다.",
    };
  }

  return { kind: "markdown", blocks: null, markdown: document.body ?? "", warning: null };
}

export function prepareEditorSavePayload(input: {
  markdown: string;
  blocks: unknown[];
  collaborationState?: string | null;
}): DocumentEditorBodyInput {
  return {
    body: input.markdown,
    editorBody: JSON.stringify(input.blocks),
    editorBodyFormat: "blocknote-json",
    collaborationState: input.collaborationState ?? null,
  };
}
```

- [ ] **Step 4: Verify helper tests pass**

Run:

```bash
npm test -- src/views/document-editor/documentBody.test.ts
```

Expected:

- PASS.

- [ ] **Step 5: Commit**

```bash
git add src/views/document-editor/documentBody.ts src/views/document-editor/documentBody.test.ts
git commit -m "feat: add document editor body helpers"
```

## Task 8: Add Asset Helper Tests and Implementation

**Files:**
- Create: `src/views/document-editor/documentAssets.ts`
- Create: `src/views/document-editor/documentAssets.test.ts`

- [ ] **Step 1: Write failing tests**

Create `src/views/document-editor/documentAssets.test.ts`:

```ts
import { describe, expect, it, vi } from "vitest";
import { assertSupportedImageName, createDocumentImageUploader } from "./documentAssets";

describe("documentAssets", () => {
  it("accepts supported image file names", () => {
    expect(() => assertSupportedImageName("a.png")).not.toThrow();
    expect(() => assertSupportedImageName("a.jpg")).not.toThrow();
    expect(() => assertSupportedImageName("a.jpeg")).not.toThrow();
    expect(() => assertSupportedImageName("a.gif")).not.toThrow();
    expect(() => assertSupportedImageName("a.webp")).not.toThrow();
  });

  it("rejects unsupported image file names", () => {
    expect(() => assertSupportedImageName("a.pdf")).toThrow("지원하지 않는 이미지 형식입니다");
  });

  it("uploads through the API and converts the returned file path to an asset URL", async () => {
    const uploadAsset = vi.fn().mockResolvedValue({
      id: "asset-1",
      documentId: "doc-1",
      fileName: "a.png",
      filePath: "/vault/files/documents/doc-1/assets/asset-1/a.png",
      url: "/vault/files/documents/doc-1/assets/asset-1/a.png",
    });
    const convertFileSrc = vi.fn((path: string) => `asset://${path}`);
    const upload = createDocumentImageUploader({ documentId: "doc-1", uploadAsset, convertFileSrc });

    const result = await upload(new File(["x"], "a.png", { type: "image/png" }));

    expect(uploadAsset).toHaveBeenCalledWith("doc-1", "a.png", [120]);
    expect(result).toBe("asset:///vault/files/documents/doc-1/assets/asset-1/a.png");
  });
});
```

- [ ] **Step 2: Verify tests fail**

Run:

```bash
npm test -- src/views/document-editor/documentAssets.test.ts
```

Expected:

- FAIL because helper module does not exist.

- [ ] **Step 3: Implement helpers**

Create `src/views/document-editor/documentAssets.ts`:

```ts
import { convertFileSrc as tauriConvertFileSrc } from "@tauri-apps/api/core";
import type { DocumentAsset } from "../../domain/types";
import { api } from "../../api/client";

export function assertSupportedImageName(name: string) {
  const lower = name.toLowerCase();
  if (!/\.(png|jpe?g|gif|webp)$/.test(lower)) {
    throw new Error("지원하지 않는 이미지 형식입니다. PNG, JPG, GIF, WEBP 파일만 사용할 수 있습니다.");
  }
}

export function createDocumentImageUploader(deps: {
  documentId: string;
  uploadAsset?: (documentId: string, fileName: string, bytes: number[]) => Promise<DocumentAsset>;
  convertFileSrc?: (path: string) => string;
}) {
  const uploadAsset = deps.uploadAsset ?? api.document.uploadAsset;
  const convertFileSrc = deps.convertFileSrc ?? tauriConvertFileSrc;

  return async (file: File) => {
    assertSupportedImageName(file.name);
    const bytes = Array.from(new Uint8Array(await file.arrayBuffer()));
    const asset = await uploadAsset(deps.documentId, file.name, bytes);
    return convertFileSrc(asset.filePath || asset.url);
  };
}
```

- [ ] **Step 4: Verify asset helper tests pass**

Run:

```bash
npm test -- src/views/document-editor/documentAssets.test.ts
```

Expected:

- PASS.

- [ ] **Step 5: Commit**

```bash
git add src/views/document-editor/documentAssets.ts src/views/document-editor/documentAssets.test.ts
git commit -m "feat: add document image upload helpers"
```

## Task 9: Add Collaboration Boundary

**Files:**
- Create: `src/views/document-editor/documentCollaboration.ts`
- Create: `src/views/document-editor/documentCollaboration.test.ts`

- [ ] **Step 1: Write failing tests**

Create `src/views/document-editor/documentCollaboration.test.ts`:

```ts
import { describe, expect, it } from "vitest";
import { createDocumentCollaboration } from "./documentCollaboration";

describe("documentCollaboration", () => {
  it("creates a stable room key from the document id", () => {
    const collaboration = createDocumentCollaboration({ documentId: "doc-1", providerMode: "local" });

    expect(collaboration.room).toBe("work-vault:document:doc-1");
    expect(collaboration.status).toBe("local");
    collaboration.destroy();
  });

  it("encodes and restores snapshots as base64 strings", () => {
    const collaboration = createDocumentCollaboration({ documentId: "doc-1", providerMode: "local" });
    const snapshot = collaboration.encodeState();
    const restored = createDocumentCollaboration({ documentId: "doc-1", initialState: snapshot, providerMode: "local" });

    expect(snapshot.length).toBeGreaterThan(0);
    expect(restored.room).toBe(collaboration.room);
    collaboration.destroy();
    restored.destroy();
  });

  it("creates a provider-backed collaboration object when enabled", () => {
    const collaboration = createDocumentCollaboration({ documentId: "doc-1", providerMode: "webrtc" });

    expect(collaboration.room).toBe("work-vault:document:doc-1");
    expect(collaboration.provider).not.toBeNull();
    collaboration.destroy();
  });
});
```

- [ ] **Step 2: Verify tests fail**

Run:

```bash
npm test -- src/views/document-editor/documentCollaboration.test.ts
```

Expected:

- FAIL because helper module does not exist.

- [ ] **Step 3: Implement Yjs boundary**

Create `src/views/document-editor/documentCollaboration.ts`:

```ts
import * as Y from "yjs";
import { WebrtcProvider } from "y-webrtc";

export type CollaborationStatus = "local" | "connected" | "disconnected";
export type CollaborationProviderMode = "local" | "webrtc";

function bytesToBase64(bytes: Uint8Array) {
  let binary = "";
  bytes.forEach((byte) => {
    binary += String.fromCharCode(byte);
  });
  return btoa(binary);
}

function base64ToBytes(value: string) {
  const binary = atob(value);
  return Uint8Array.from(binary, (char) => char.charCodeAt(0));
}

export function createDocumentCollaboration(input: {
  documentId: string;
  initialState?: string | null;
  providerMode?: CollaborationProviderMode;
}) {
  const doc = new Y.Doc();
  const room = `work-vault:document:${input.documentId}`;
  if (input.initialState) {
    Y.applyUpdate(doc, base64ToBytes(input.initialState));
  }
  const provider = input.providerMode === "webrtc" ? new WebrtcProvider(room, doc) : null;

  return {
    doc,
    provider,
    room,
    status: provider ? ("connected" as CollaborationStatus) : ("local" as CollaborationStatus),
    fragment: doc.getXmlFragment("document-store"),
    encodeState: () => bytesToBase64(Y.encodeStateAsUpdate(doc)),
    destroy: () => {
      provider?.destroy();
      doc.destroy();
    },
  };
}
```

- [ ] **Step 4: Verify collaboration tests pass**

Run:

```bash
npm test -- src/views/document-editor/documentCollaboration.test.ts
```

Expected:

- PASS.

- [ ] **Step 5: Commit**

```bash
git add src/views/document-editor/documentCollaboration.ts src/views/document-editor/documentCollaboration.test.ts
git commit -m "feat: add document collaboration boundary"
```

## Task 10: Integrate BlockNote Editor

**Files:**
- Create: `src/views/document-editor/BlockDocumentEditor.tsx`
- Modify: `src/views/DocEditor.tsx`
- Modify: `src/views/DocEditor.test.tsx`
- Modify: `src/styles/tokens.css`

- [ ] **Step 1: Mock BlockNote for component tests**

In `src/views/DocEditor.test.tsx`, mock the new editor component:

```ts
vi.mock("./document-editor/BlockDocumentEditor", () => ({
  BlockDocumentEditor: ({
    initialMarkdown,
    onChange,
    onBlur,
  }: {
    initialMarkdown: string;
    onChange: (payload: { markdown: string; blocks: unknown[]; collaborationState?: string | null }) => void;
    onBlur: () => void;
  }) => (
    <textarea
      aria-label="라이브 문서 본문"
      defaultValue={initialMarkdown}
      onChange={(event) =>
        onChange({
          markdown: event.currentTarget.value,
          blocks: [{ type: "paragraph", content: event.currentTarget.value }],
          collaborationState: null,
        })
      }
      onBlur={onBlur}
    />
  ),
}));
```

- [ ] **Step 2: Replace old preview tests with failing live editor tests**

Update tests in `src/views/DocEditor.test.tsx`:

```ts
it("마운트 시 최신 본문을 라이브 에디터에 전달", async () => {
  render(<DocEditor document={doc} />);
  const editor = screen.getByLabelText("라이브 문서 본문") as HTMLTextAreaElement;
  await waitFor(() => expect(editor.value).toBe("# 안녕\n본문"));
});

it("입력 후 blur 시 Markdown과 BlockNote JSON을 함께 저장", async () => {
  render(<DocEditor document={doc} />);
  const editor = screen.getByLabelText("라이브 문서 본문") as HTMLTextAreaElement;
  await waitFor(() => expect(editor.value).toBe("# 안녕\n본문"));

  await userEvent.type(editor, " 끝");
  editor.blur();

  expect(api.document.setEditorBody).toHaveBeenCalledWith("1", {
    body: "# 안녕\n본문 끝",
    editorBody: JSON.stringify([{ type: "paragraph", content: "# 안녕\n본문 끝" }]),
    editorBodyFormat: "blocknote-json",
    collaborationState: null,
  });
});
```

- [ ] **Step 3: Verify tests fail**

Run:

```bash
npm test -- src/views/DocEditor.test.tsx
```

Expected:

- FAIL because `DocEditor` still renders old textarea and calls `setBody`.

- [ ] **Step 4: Implement `BlockDocumentEditor`**

Create `src/views/document-editor/BlockDocumentEditor.tsx`:

```tsx
import "@blocknote/core/fonts/inter.css";
import "@blocknote/mantine/style.css";
import { BlockNoteView } from "@blocknote/mantine";
import { useCreateBlockNote, useEditorChange } from "@blocknote/react";
import { useEffect, useMemo, useState } from "react";
import { createDocumentImageUploader } from "./documentAssets";
import { createDocumentCollaboration } from "./documentCollaboration";
import { withCollaboration } from "@blocknote/core/yjs";

export interface BlockDocumentEditorProps {
  documentId: string;
  initialBlocks: unknown[] | null;
  initialMarkdown: string;
  collaborationState?: string | null;
  onChange: (payload: { markdown: string; blocks: unknown[]; collaborationState?: string | null }) => void;
  onBlur: () => void;
  onWarning?: (message: string) => void;
}

export function BlockDocumentEditor(props: BlockDocumentEditorProps) {
  const collaboration = useMemo(
    () =>
      createDocumentCollaboration({
        documentId: props.documentId,
        initialState: props.collaborationState,
        providerMode: "webrtc",
      }),
    [props.documentId, props.collaborationState],
  );
  const [loadedMarkdown, setLoadedMarkdown] = useState(false);
  const uploadFile = useMemo(() => createDocumentImageUploader({ documentId: props.documentId }), [props.documentId]);
  const editor = useCreateBlockNote(
    withCollaboration({
      initialContent: props.initialBlocks ?? undefined,
      uploadFile,
      tables: {
        splitCells: true,
        cellBackgroundColor: true,
        cellTextColor: true,
        headers: true,
      },
      collaboration: {
        provider: collaboration.provider!,
        fragment: collaboration.fragment,
        user: {
          name: "Work Vault",
          color: "#5b5bd6",
        },
        showCursorLabels: "activity",
      },
    }),
    [props.documentId],
  );

  useEffect(() => {
    return () => collaboration.destroy();
  }, [collaboration]);

  useEffect(() => {
    if (props.initialBlocks || loadedMarkdown) return;
    setLoadedMarkdown(true);
    if (props.initialMarkdown.trim()) {
      void editor.tryParseMarkdownToBlocks(props.initialMarkdown).then((blocks) => {
        editor.replaceBlocks(editor.document, blocks);
      });
    }
  }, [editor, loadedMarkdown, props.initialBlocks, props.initialMarkdown]);

  useEditorChange((currentEditor) => {
    void currentEditor.blocksToMarkdownLossy(currentEditor.document).then((markdown) => {
      props.onChange({
        markdown,
        blocks: currentEditor.document,
        collaborationState: collaboration.encodeState(),
      });
    });
  }, editor);

  return (
    <div className="block-document-editor" onBlurCapture={props.onBlur}>
      <BlockNoteView editor={editor} theme="light" />
    </div>
  );
}
```

- [ ] **Step 5: Replace `DocEditor` internals**

In `src/views/DocEditor.tsx`:

- remove `marked`, `DOMPurify`, `mode`, `html`, `textarea`, and preview toggle.
- use `buildInitialEditorSource` and `prepareEditorSavePayload`.
- call `api.document.setEditorBody`.

The save function should be:

```ts
const save = (payload: ReturnType<typeof prepareEditorSavePayload>) => {
  api.document
    .setEditorBody(document.id, payload)
    .then(() => setSaveState("saved"))
    .catch((e) => setError(String(e)));
};
```

The editor render should be:

```tsx
<BlockDocumentEditor
  key={loaded.id}
  documentId={loaded.id}
  initialBlocks={source.kind === "blocks" ? source.blocks : null}
  initialMarkdown={source.markdown}
  collaborationState={loaded.collaborationState}
  onChange={(value) => onChange(prepareEditorSavePayload(value))}
  onBlur={flush}
  onWarning={setError}
/>
```

- [ ] **Step 6: Add editor styles**

Add to `src/styles/tokens.css`:

```css
.block-document-editor {
  height: 100%;
  min-height: 0;
  overflow: auto;
  background: var(--bg);
}

.block-document-editor .bn-container {
  min-height: 100%;
  background: var(--bg);
  color: var(--text);
}
```

- [ ] **Step 7: Verify DocEditor tests pass**

Run:

```bash
npm test -- src/views/DocEditor.test.tsx
```

Expected:

- PASS.

- [ ] **Step 8: Commit**

```bash
git add src/views/DocEditor.tsx src/views/DocEditor.test.tsx src/views/document-editor/BlockDocumentEditor.tsx src/styles/tokens.css
git commit -m "feat: replace markdown preview with live block editor"
```

## Task 11: Full Verification and Packaging

**Files:**
- No planned source files unless verification exposes defects.

- [ ] **Step 1: Run frontend tests**

Run:

```bash
npm test
```

Expected:

- PASS.

- [ ] **Step 2: Run TypeScript build**

Run:

```bash
npx tsc --noEmit
```

Expected:

- PASS.

- [ ] **Step 3: Run production web build**

Run:

```bash
npm run build
```

Expected:

- PASS.

- [ ] **Step 4: Run Rust tests**

Run:

```bash
cd src-tauri && cargo test --lib
```

Expected:

- PASS.

- [ ] **Step 5: Build Tauri package**

Run:

```bash
npm run tauri -- build
```

Expected:

- PASS and produce a macOS bundle/DMG under `src-tauri/target/release/bundle`.

- [ ] **Step 6: Manual verification**

Open the app and verify:

- existing Markdown document opens in the live editor
- typing Korean text works
- slash menu creates paragraph, heading, list, checklist, quote, code block, table, image, and file blocks
- table edits persist after reopen
- uploaded image persists after reopen
- dragging blocks changes order and persists after reopen
- `Document.body` contains readable Markdown for external sharing
- collaboration status remains local/disconnected instead of falsely claiming network collaboration

- [ ] **Step 7: Commit any fixes**

If verification required fixes:

```bash
git add src src-tauri package.json package-lock.json docs/superpowers/plans/2026-07-07-live-block-document-editor.md
git commit -m "fix: stabilize live block document editor"
```

- [ ] **Step 8: Push branch**

```bash
git push -u origin codex/live-block-document-editor
```
