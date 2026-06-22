import type { Document } from "../domain/types";
import { useBlocks } from "../hooks/useBlocks";
import { Icon } from "../ui/icons/Icon";
import { BlockEditor } from "./BlockEditor";

export interface DocEditorProps {
  document: Document;
}

export function DocEditor({ document }: DocEditorProps) {
  const { blocks, error, addBlock, addBlockAfter, changeText, toggleCheck, remove } = useBlocks(document.id);

  return (
    <div style={{ padding: "32px 0 0" }}>
      <div style={{ maxWidth: 720, margin: "0 auto", padding: "0 24px" }}>
        <div style={{ display: "flex", alignItems: "center", gap: 8, fontSize: 12, color: "var(--text2)", marginBottom: 10 }}>
          <span style={{ width: 6, height: 6, borderRadius: "50%", background: "var(--st-done)" }} />
          자동 저장
        </div>
        <h1 style={{ margin: "0 0 18px", fontSize: 30, fontWeight: 700, letterSpacing: "-0.5px" }}>
          {document.title}
        </h1>
        {error && (
          <div style={{ display: "flex", alignItems: "center", gap: 6, color: "#ef4444", fontSize: 13, marginBottom: 8 }}>
            <Icon name="alert" size={15} />
            <span>{error}</span>
          </div>
        )}
      </div>
      <BlockEditor
        blocks={blocks}
        onChangeText={changeText}
        onToggleCheck={toggleCheck}
        onAddBlock={addBlock}
        onAddBlockAfter={addBlockAfter}
        onDelete={remove}
      />
    </div>
  );
}
