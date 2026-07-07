import "@blocknote/core/fonts/inter.css";
import "@blocknote/mantine/style.css";
import { BlockNoteView } from "@blocknote/mantine";
import type { PartialBlock } from "@blocknote/core";
import { useCreateBlockNote, useEditorChange } from "@blocknote/react";
import { useEffect, useMemo, useRef, useState } from "react";
import { createDocumentImageUploader } from "./documentAssets";
import { createDocumentCollaboration } from "./documentCollaboration";

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
  const [markdownLoaded, setMarkdownLoaded] = useState(false);
  const emittedInitial = useRef(false);
  const collaboration = useMemo(
    () =>
      createDocumentCollaboration({
        documentId: props.documentId,
        initialState: props.collaborationState,
        providerMode: "webrtc",
      }),
    [props.documentId, props.collaborationState],
  );
  const uploadFile = useMemo(() => createDocumentImageUploader({ documentId: props.documentId }), [props.documentId]);
  const initialContent = useMemo(
    () => (Array.isArray(props.initialBlocks) && props.initialBlocks.length > 0 ? (props.initialBlocks as PartialBlock[]) : undefined),
    [props.initialBlocks],
  );
  const editor = useCreateBlockNote(
    {
      initialContent,
      uploadFile,
      tables: {
        splitCells: true,
        cellBackgroundColor: true,
        cellTextColor: true,
        headers: true,
      },
      collaboration: {
        fragment: collaboration.fragment,
        provider: collaboration.provider ?? undefined,
        user: {
          name: "Work Vault",
          color: "#5b5bd6",
        },
        showCursorLabels: "activity",
      },
    },
    [props.documentId],
  );

  useEffect(() => {
    return () => collaboration.destroy();
  }, [collaboration]);

  useEffect(() => {
    if (initialContent || markdownLoaded) return;
    setMarkdownLoaded(true);
    if (!props.initialMarkdown.trim()) return;
    try {
      const blocks = editor.tryParseMarkdownToBlocks(props.initialMarkdown);
      editor.replaceBlocks(editor.document, blocks);
    } catch (e) {
      props.onWarning?.(String(e));
    }
  }, [editor, initialContent, markdownLoaded, props]);

  useEditorChange((currentEditor) => {
    try {
      const markdown = currentEditor.blocksToMarkdownLossy(currentEditor.document);
      props.onChange({
        markdown,
        blocks: currentEditor.document,
        collaborationState: collaboration.encodeState(),
      });
      emittedInitial.current = true;
    } catch (e) {
      props.onWarning?.(String(e));
    }
  }, editor);

  return (
    <div className="block-document-editor" onBlurCapture={props.onBlur}>
      <BlockNoteView editor={editor} theme="light" />
    </div>
  );
}
