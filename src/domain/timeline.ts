import type { Task, TaskStatus } from "./types";

export interface TimelineItem {
  id: number;
  title: string;
  dueDate: string;
  status: TaskStatus;
  /** 전체 기간 내 위치 0~1 */
  ratio: number;
}

export interface Timeline {
  items: TimelineItem[];
  minDate: string | null;
  maxDate: string | null;
}

/** 마감일이 있는 태스크를 날짜순으로 배치하고 기간 내 상대 위치(ratio)를 계산. */
export function buildTimeline(tasks: Task[]): Timeline {
  const dated = tasks
    .filter((t): t is Task & { dueDate: string } => !!t.dueDate)
    .sort((a, b) => (a.dueDate < b.dueDate ? -1 : a.dueDate > b.dueDate ? 1 : 0));

  if (dated.length === 0) return { items: [], minDate: null, maxDate: null };

  const minDate = dated[0].dueDate;
  const maxDate = dated[dated.length - 1].dueDate;
  const min = new Date(minDate).getTime();
  const max = new Date(maxDate).getTime();
  const span = max - min;

  const items: TimelineItem[] = dated.map((t) => ({
    id: t.id,
    title: t.title,
    dueDate: t.dueDate,
    status: t.status,
    ratio: span === 0 ? 0.5 : (new Date(t.dueDate).getTime() - min) / span,
  }));

  return { items, minDate, maxDate };
}
