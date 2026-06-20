import { useCallback, useEffect, useRef, useState } from "react";
import { api } from "../api/client";
import type { Block, BlockType } from "../domain/types";
import { parseContent, stringify, withChecked, withText } from "../domain/blockContent";

/** 문서의 블록 로딩 + 추가/수정(디바운스 저장)/체크 토글/삭제. */
export function useBlocks(documentId: number | null) {
  const [blocks, setBlocks] = useState<Block[]>([]);
  const [error, setError] = useState<string | null>(null);
  const timers = useRef<Map<number, ReturnType<typeof setTimeout>>>(new Map());

  const reload = useCallback(async () => {
    if (documentId == null) {
      setBlocks([]);
      return;
    }
    try {
      setBlocks(await api.block.list(documentId));
      setError(null);
    } catch (e) {
      setError(String(e));
    }
  }, [documentId]);

  useEffect(() => {
    void reload();
  }, [reload]);

  const addBlock = useCallback(
    async (type: BlockType) => {
      if (documentId == null) return;
      try {
        const last = blocks[blocks.length - 1];
        const sortOrder = (last ? last.sortOrder : 0) + 1;
        await api.block.create({ documentId, type, content: stringify({ text: "", checked: false }), sortOrder });
        await reload();
      } catch (e) {
        setError(String(e));
      }
    },
    [documentId, blocks, reload],
  );

  /** 텍스트는 즉시 로컬 반영 + 디바운스 저장 */
  const changeText = useCallback((block: Block, text: string) => {
    const content = withText(block.content, text);
    setBlocks((prev) => prev.map((b) => (b.id === block.id ? { ...b, content } : b)));
    const map = timers.current;
    const existing = map.get(block.id);
    if (existing) clearTimeout(existing);
    map.set(
      block.id,
      setTimeout(() => {
        void api.block.update({ id: block.id, type: block.type, content }).catch((e) => setError(String(e)));
        map.delete(block.id);
      }, 500),
    );
  }, []);

  const toggleCheck = useCallback(
    async (block: Block) => {
      const next = !parseContent(block.content).checked;
      const content = withChecked(block.content, next);
      setBlocks((prev) => prev.map((b) => (b.id === block.id ? { ...b, content } : b)));
      try {
        await api.block.update({ id: block.id, type: block.type, content });
      } catch (e) {
        setError(String(e));
      }
    },
    [],
  );

  const remove = useCallback(
    async (block: Block) => {
      try {
        await api.block.delete(block.id);
        await reload();
      } catch (e) {
        setError(String(e));
      }
    },
    [reload],
  );

  return { blocks, error, addBlock, changeText, toggleCheck, remove };
}
