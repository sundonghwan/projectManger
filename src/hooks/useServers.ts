import { useCallback, useEffect, useState } from "react";
import { api } from "../api/client";
import type { ServerConnection } from "../domain/types";
import type { ServerFormData } from "../views/ServerPanel";

export function useServers(businessId: string | null, projectId: string | null) {
  const [servers, setServers] = useState<ServerConnection[]>([]);
  const [error, setError] = useState<string | null>(null);

  const reload = useCallback(async () => {
    if (businessId == null) {
      setServers([]);
      return;
    }
    try {
      setServers(await api.server.list(businessId));
      setError(null);
    } catch (e) {
      setError(String(e));
    }
  }, [businessId]);

  useEffect(() => {
    void reload();
  }, [reload]);

  const create = useCallback(
    async (data: ServerFormData) => {
      if (businessId == null) return;
      try {
        await api.server.create({ businessId, projectId, ...data });
        await reload();
      } catch (e) {
        setError(String(e));
      }
    },
    [businessId, projectId, reload],
  );

  const update = useCallback(
    async (data: ServerFormData & { id: string }) => {
      try {
        const { id, ...rest } = data;
        await api.server.update({ id, ...rest });
        await reload();
      } catch (e) {
        setError(String(e));
      }
    },
    [reload],
  );

  const archive = useCallback(
    async (id: string) => {
      try {
        await api.server.archive(id);
        await reload();
      } catch (e) {
        setError(String(e));
      }
    },
    [reload],
  );

  const setSecret = useCallback(
    async (id: string, secret: string) => {
      try {
        await api.server.setSecret(id, secret);
        await reload();
      } catch (e) {
        setError(String(e));
      }
    },
    [reload],
  );

  return { servers, error, create, update, archive, setSecret };
}
