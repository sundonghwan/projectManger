import { useCallback, useEffect, useState } from "react";
import { api } from "../api/client";
import type { Task, TaskStatus } from "../domain/types";

/** 선택된 사업/프로젝트의 태스크 로딩 + 생성/이동/완료토글. */
export function useTasks(businessId: number | null, projectId: number | null) {
  const [tasks, setTasks] = useState<Task[]>([]);
  const [error, setError] = useState<string | null>(null);

  const reload = useCallback(async () => {
    if (businessId == null) {
      setTasks([]);
      return;
    }
    try {
      setTasks(await api.task.list(businessId, projectId));
      setError(null);
    } catch (e) {
      setError(String(e));
    }
  }, [businessId, projectId]);

  useEffect(() => {
    void reload();
  }, [reload]);

  const create = useCallback(
    async (status: TaskStatus) => {
      if (businessId == null) return;
      try {
        const t = await api.task.create({ businessId, projectId, title: "새 태스크" });
        // 생성은 기본 todo. 다른 컬럼에서 추가했으면 그 상태로 이동.
        if (status !== "todo") {
          await api.task.move({ id: t.id, status, sortOrder: t.sortOrder });
        }
        await reload();
      } catch (e) {
        setError(String(e));
      }
    },
    [businessId, projectId, reload],
  );

  const move = useCallback(
    async (id: number, status: TaskStatus, sortOrder: number) => {
      try {
        await api.task.move({ id, status, sortOrder });
        await reload();
      } catch (e) {
        setError(String(e));
      }
    },
    [reload],
  );

  const toggleDone = useCallback(
    async (t: Task) => {
      const next: TaskStatus = t.status === "done" ? "todo" : "done";
      try {
        await api.task.move({ id: t.id, status: next, sortOrder: t.sortOrder });
        await reload();
      } catch (e) {
        setError(String(e));
      }
    },
    [reload],
  );

  return { tasks, error, reload, create, move, toggleDone };
}
