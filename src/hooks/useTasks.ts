import { useCallback, useEffect, useState } from "react";
import { api, type TaskUpdateInput } from "../api/client";
import type { Label, Task, TaskStatus } from "../domain/types";
import { groupLabelsByTask } from "../domain/labels";

/** 선택된 사업/프로젝트의 태스크 로딩 + 생성/이동/완료토글 + 라벨. */
export function useTasks(businessId: number | null, projectId: number | null) {
  const [tasks, setTasks] = useState<Task[]>([]);
  const [labelsByTask, setLabelsByTask] = useState<Record<number, Label[]>>({});
  const [error, setError] = useState<string | null>(null);

  const reload = useCallback(async () => {
    if (businessId == null) {
      setTasks([]);
      setLabelsByTask({});
      return;
    }
    try {
      const [list, labelMap] = await Promise.all([
        api.task.list(businessId, projectId),
        api.label.map(businessId),
      ]);
      setTasks(list);
      setLabelsByTask(groupLabelsByTask(labelMap));
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

  const update = useCallback(
    async (input: TaskUpdateInput) => {
      try {
        await api.task.update(input);
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
        await api.task.archive(id);
        await reload();
      } catch (e) {
        setError(String(e));
      }
    },
    [reload],
  );

  const assignLabel = useCallback(
    async (taskId: number, name: string, color: string) => {
      try {
        const label = await api.label.create(name, color);
        await api.label.assign(taskId, label.id);
        await reload();
      } catch (e) {
        setError(String(e));
      }
    },
    [reload],
  );

  const removeLabel = useCallback(
    async (taskId: number, labelId: number) => {
      try {
        await api.label.unassign(taskId, labelId);
        await reload();
      } catch (e) {
        setError(String(e));
      }
    },
    [reload],
  );

  return {
    tasks,
    labelsByTask,
    error,
    reload,
    create,
    move,
    toggleDone,
    update,
    archive,
    assignLabel,
    removeLabel,
  };
}
