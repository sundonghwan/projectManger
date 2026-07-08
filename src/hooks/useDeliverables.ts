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

  // 뷰 진입/사업 전환 시 디스크 미러와 메타를 재조정한 뒤 목록을 불러온다.
  // (Finder 등에서 직접 추가/삭제한 파일을 반영. 실패해도 목록 로딩은 진행.)
  useEffect(() => {
    let alive = true;
    (async () => {
      if (businessId != null) {
        try {
          await api.deliverable.reconcile();
        } catch {
          /* reconcile 실패는 무시하고 목록은 그대로 로드 */
        }
      }
      if (alive) await reload();
    })();
    return () => {
      alive = false;
    };
  }, [reload, businessId]);

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
  const uploadFiles = useCallback(
    async (files: File[], folderId?: string | null) => {
      if (businessId == null || files.length === 0) return;
      setUploading(true);
      try {
        const inputs = await Promise.all(files.map(async (file) => ({
          fileName: file.name,
          bytes: Array.from(new Uint8Array(await file.arrayBuffer())),
        })));
        const created = await api.deliverable.uploadFiles(businessId, projectId, inputs, folderId ?? null);
        await reload();
        onChanged?.();
        if (created.length < files.length) {
          setError(`${files.length}개 중 ${created.length}개만 업로드되었습니다. (빈 파일·저장 실패 등 제외)`);
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

  const showInFolder = useCallback(async (d: Deliverable) => {
    if (!d.filePath) {
      setError("파일 경로가 없습니다.");
      return;
    }
    try {
      await api.deliverable.showInFolder(d.id);
    } catch {
      setError("파일 폴더를 열 수 없습니다. 파일이 이동되었거나 삭제되었을 수 있습니다.");
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

  return { deliverables, error, uploading, reload, upload, uploadFiles, rename, setStatus, open, showInFolder, archive, move };
}
