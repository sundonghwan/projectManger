import { useCallback, useEffect, useState } from "react";
import { api } from "../api/client";
import type { Document } from "../domain/types";

/** 사업 문서 로딩 + 생성/이름변경/보관. (개별 문서는 목록에서 선택해 편집기로 연다) */
export function useDocuments(
  businessId: number | null,
  projectId: number | null,
  onChanged?: () => void,
) {
  const [documents, setDocuments] = useState<Document[]>([]);
  const [error, setError] = useState<string | null>(null);

  const reload = useCallback(async () => {
    if (businessId == null) {
      setDocuments([]);
      return;
    }
    try {
      setDocuments(await api.document.list(businessId));
      setError(null);
    } catch (e) {
      setError(String(e));
    }
  }, [businessId]);

  useEffect(() => {
    void reload();
  }, [reload]);

  /** 주어진 제목으로 새 문서 생성 후 생성된 문서를 반환(호출측에서 바로 편집기로 열 수 있게). */
  const create = useCallback(
    async (title: string, folderId?: number | null): Promise<Document | null> => {
      if (businessId == null) return null;
      const name = title.trim();
      if (!name) return null;
      try {
        const doc = await api.document.create({ businessId, projectId, folderId: folderId ?? null, title: name });
        await reload();
        onChanged?.();
        return doc;
      } catch (e) {
        setError(String(e));
        return null;
      }
    },
    [businessId, projectId, reload, onChanged],
  );

  const move = useCallback(
    async (id: number, folderId: number | null) => {
      try {
        await api.document.move(id, folderId);
        await reload();
        onChanged?.();
      } catch (e) {
        setError(String(e));
      }
    },
    [reload, onChanged],
  );

  const rename = useCallback(
    async (id: number, title: string) => {
      try {
        await api.document.rename(id, title);
        await reload();
        onChanged?.();
      } catch (e) {
        setError(String(e));
      }
    },
    [reload, onChanged],
  );

  const archive = useCallback(
    async (id: number) => {
      try {
        await api.document.archive(id);
        await reload();
        onChanged?.();
      } catch (e) {
        setError(String(e));
      }
    },
    [reload, onChanged],
  );

  return { documents, error, reload, create, rename, archive, move };
}
