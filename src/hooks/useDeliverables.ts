import { useCallback, useEffect, useState } from "react";
import { api } from "../api/client";
import type { Deliverable, DeliverableStatus, DeliverableVersion } from "../domain/types";

/** 사업 산출물 로딩 + 생성/상태변경/버전추가/보관 + 선택 산출물의 버전 히스토리. */
export function useDeliverables(
  businessId: number | null,
  projectId: number | null,
  onChanged?: () => void,
) {
  const [deliverables, setDeliverables] = useState<Deliverable[]>([]);
  const [selectedId, setSelectedId] = useState<number | null>(null);
  const [versions, setVersions] = useState<DeliverableVersion[]>([]);
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

  const loadVersions = useCallback(async (id: number) => {
    setVersions(await api.deliverable.versions(id));
  }, []);

  const select = useCallback(
    async (id: number) => {
      setSelectedId(id);
      try {
        await loadVersions(id);
      } catch (e) {
        setError(String(e));
      }
    },
    [loadVersions],
  );

  const create = useCallback(async () => {
    if (businessId == null) return;
    try {
      const d = await api.deliverable.create({
        businessId,
        projectId,
        title: "새 산출물",
        kind: "file",
      });
      await reload();
      onChanged?.();
      await select(d.id);
    } catch (e) {
      setError(String(e));
    }
  }, [businessId, projectId, reload, onChanged, select]);

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

  const addVersion = useCallback(
    async (id: number) => {
      try {
        await api.deliverable.addVersion(id, "새 버전");
        await reload();
        await loadVersions(id);
      } catch (e) {
        setError(String(e));
      }
    },
    [reload, loadVersions],
  );

  const archive = useCallback(
    async (id: number) => {
      try {
        await api.deliverable.archive(id);
        if (selectedId === id) setSelectedId(null);
        await reload();
        onChanged?.();
      } catch (e) {
        setError(String(e));
      }
    },
    [reload, onChanged, selectedId],
  );

  return { deliverables, selectedId, versions, error, select, create, setStatus, addVersion, archive };
}
