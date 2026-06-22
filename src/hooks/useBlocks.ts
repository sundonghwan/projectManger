import { useCallback, useEffect, useRef, useState } from "react";
import { api } from "../api/client";
import type { Block, BlockType } from "../domain/types";
import { parseContent, stringify, withChecked, withText } from "../domain/blockContent";

/** 문서의 블록 로딩 + 추가/수정(디바운스 저장)/체크 토글/삭제. */
export function useBlocks(documentId: number | null) {
  const [blocks, setBlocks] = useState<Block[]>([]);
  const [error, setError] = useState<string | null>(null);
  const timers = useRef<Map<number, ReturnType<typeof setTimeout>>>(new Map());
  const seeded = useRef<Set<number>>(new Set());

  const reload = useCallback(async () => {
    if (documentId == null) {
      setBlocks([]);
      return;
    }
    try {
      const list = await api.block.list(documentId);
      // 빈 문서는 클릭 없이 바로 쓸 수 있도록 문단 한 줄을 자동 생성한다(문서당 1회).
      if (list.length === 0 && !seeded.current.has(documentId)) {
        seeded.current.add(documentId);
        const b = await api.block.create({
          documentId,
          type: "paragraph",
          content: stringify({ text: "", checked: false }),
          sortOrder: 1,
        });
        setBlocks([b]);
      } else {
        setBlocks(list);
      }
      setError(null);
    } catch (e) {
      setError(String(e));
    }
  }, [documentId]);

  useEffect(() => {
    void reload();
  }, [reload]);

  /** 지정한 sort_order 위치에 빈 블록을 만들고 생성된 블록을 반환. */
  const insert = useCallback(
    async (type: BlockType, sortOrder: number): Promise<Block | null> => {
      if (documentId == null) return null;
      try {
        const b = await api.block.create({
          documentId,
          type,
          content: stringify({ text: "", checked: false }),
          sortOrder,
        });
        await reload();
        return b;
      } catch (e) {
        setError(String(e));
        return null;
      }
    },
    [documentId, reload],
  );

  const addBlock = useCallback(
    (type: BlockType) => {
      const last = blocks[blocks.length - 1];
      return insert(type, (last ? last.sortOrder : 0) + 1);
    },
    [blocks, insert],
  );

  /** 특정 블록 바로 다음에 새 블록 삽입(Enter 로 이어쓰기). */
  const addBlockAfter = useCallback(
    (block: Block, type: BlockType = "paragraph") => {
      const idx = blocks.findIndex((b) => b.id === block.id);
      const next = blocks[idx + 1];
      const sortOrder = next ? (block.sortOrder + next.sortOrder) / 2 : block.sortOrder + 1;
      return insert(type, sortOrder);
    },
    [blocks, insert],
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

  return { blocks, error, addBlock, addBlockAfter, changeText, toggleCheck, remove };
}
