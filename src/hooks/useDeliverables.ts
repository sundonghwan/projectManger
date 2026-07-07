import { useCallback, useEffect, useState } from "react";
import { api } from "../api/client";
import type { Deliverable, DeliverableStatus } from "../domain/types";

/** 사업 산출물(업로드 파일) 로딩 + 업로드/상태변경/이름변경/열기/보관. */
export function useDeliverables(
  businessId: string | null,
  projectId: string | null,
  onChanged?: () => void,
) {
  const [deliverables, setDeliverables] = useState<Deliverable[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [uploading, setUploading] = useState(false);

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
    async (paths: string[], folderId?: string | null) => {
      if (businessId == null || paths.length === 0) return;
      setUploading(true);
      try {
        const created = await api.deliverable.upload(businessId, projectId, paths, folderId ?? null);
        await reload();
        onChanged?.();
        if (created.length < paths.length) {
          setError(`${paths.length}개 중 ${created.length}개만 업로드되었습니다. (폴더·빈 파일·접근 불가 등 제외)`);
        }
      } catch (e) {
        setError(String(e));
      } finally {
        setUploading(false);
      }
    },
    [businessId, projectId, reload, onChanged],
  );

  const move = useCallback(
    async (id: string, folderId: string | null) => {
      try {
        await api.deliverable.move(id, folderId);
        await reload();
        onChanged?.();
      } catch (e) {
        setError(String(e));
      }
    },
    [reload, onChanged],
  );

  const rename = useCallback(
    async (id: string, title: string) => {
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
    async (id: string, status: DeliverableStatus) => {
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
      await api.deliverable.open(d.id);
    } catch {
      setError("파일을 열 수 없습니다. 파일이 이동되었거나 삭제되었을 수 있습니다.");
    }
  }, []);

  const archive = useCallback(
    async (id: string) => {
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

  return { deliverables, error, uploading, reload, upload, rename, setStatus, open, archive, move };
}
