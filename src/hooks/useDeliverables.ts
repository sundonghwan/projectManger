import { useCallback, useEffect, useState } from "react";
import { api } from "../api/client";
import type { Deliverable, DeliverableStatus } from "../domain/types";

/** 사업 산출물(업로드 파일) 로딩 + 업로드/상태변경/이름변경/열기/보관. */
export function useDeliverables(
  businessId: number | null,
  projectId: number | null,
  onChanged?: () => void,
) {
  const [deliverables, setDeliverables] = useState<Deliverable[]>([]);
  const [error, setError] = useState<string | null>(null);

  const reload = useCallback(async () => {
    if (businessId == null) {
      setDeliverables([]);
      return;
    }
    try {
      setDeliverables(await api.deliverable.list(businessId));
      setError(null);
    } catch (e) {
      setError(String(e));
    }
  }, [businessId]);

  useEffect(() => {
    void reload();
  }, [reload]);

  const upload = useCallback(
    async (paths: string[]) => {
      if (businessId == null || paths.length === 0) return;
      try {
        const created = await api.deliverable.upload(businessId, projectId, paths);
        await reload();
        onChanged?.();
        if (created.length < paths.length) {
          setError(`${paths.length}개 중 ${created.length}개만 업로드됨`);
        }
      } catch (e) {
        setError(String(e));
      }
    },
    [businessId, projectId, reload, onChanged],
  );

  const rename = useCallback(
    async (id: number, title: string) => {
      try {
        await api.deliverable.rename(id, title);
        await reload();
        onChanged?.();
      } catch (e) {
        setError(String(e));
      }
    },
    [reload, onChanged],
  );

  const setStatus = useCallback(
    async (id: number, status: DeliverableStatus) => {
      try {
        await api.deliverable.setStatus(id, status);
        await reload();
      } catch (e) {
        setError(String(e));
      }
    },
    [reload],
  );

  const open = useCallback(async (d: Deliverable) => {
    if (!d.filePath) {
      setError("파일 경로가 없습니다.");
      return;
    }
    try {
      await api.deliverable.open(d.filePath);
    } catch (e) {
      setError(String(e));
    }
  }, []);

  const archive = useCallback(
    async (id: number) => {
      try {
        await api.deliverable.archive(id);
        await reload();
        onChanged?.();
      } catch (e) {
        setError(String(e));
      }
    },
    [reload, onChanged],
  );

  return { deliverables, error, reload, upload, rename, setStatus, open, archive };
}
