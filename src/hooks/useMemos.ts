import { useCallback, useEffect, useState } from "react";
import { api } from "../api/client";
import type { Memo, MemoColor } from "../domain/types";

/** 사업 메모 로딩 + 생성/수정/색상/고정/보관. */
export function useMemos(businessId: number | null, onChanged?: () => void) {
  const [memos, setMemos] = useState<Memo[]>([]);
  const [error, setError] = useState<string | null>(null);

  const reload = useCallback(async () => {
    if (businessId == null) {
      setMemos([]);
      return;
    }
    try {
      setMemos(await api.memo.list(businessId));
      setError(null);
    } catch (e) {
      setError(String(e));
    }
  }, [businessId]);

  useEffect(() => {
    void reload();
  }, [reload]);

  const create = useCallback(
    async (title: string, body: string): Promise<Memo | null> => {
      if (businessId == null) return null;
      if (!title.trim() && !body.trim()) return null; // 빈 메모 폐기
      try {
        const m = await api.memo.create(businessId, title, body);
        await reload();
        onChanged?.();
        return m;
      } catch (e) {
        setError(String(e));
        return null;
      }
    },
    [businessId, reload, onChanged],
  );

  const update = useCallback(
    async (id: number, title: string, body: string) => {
      try {
        await api.memo.update(id, title, body);
        await reload();
      } catch (e) {
        setError(String(e));
      }
    },
    [reload],
  );

  const setColor = useCallback(
    async (id: number, color: MemoColor | null) => {
      try {
        await api.memo.setColor(id, color);
        await reload();
      } catch (e) {
        setError(String(e));
      }
    },
    [reload],
  );

  const setPinned = useCallback(
    async (id: number, pinned: boolean) => {
      try {
        await api.memo.setPinned(id, pinned);
        await reload();
      } catch (e) {
        setError(String(e));
      }
    },
    [reload],
  );

  const archive = useCallback(
    async (id: number) => {
      try {
        await api.memo.archive(id);
        await reload();
        onChanged?.();
      } catch (e) {
        setError(String(e));
      }
    },
    [reload, onChanged],
  );

  return { memos, error, reload, create, update, setColor, setPinned, archive };
}
