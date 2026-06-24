import { useMemos } from "../hooks/useMemos";
import { MemoList } from "./MemoList";

export interface MemosViewProps {
  businessId: number;
  onChanged: () => void;
}

/** 메모 뷰 컨테이너 — 데이터 로딩을 useMemos 에 위임. */
export function MemosView({ businessId, onChanged }: MemosViewProps) {
  const m = useMemos(businessId, onChanged);
  return (
    <MemoList
      memos={m.memos}
      error={m.error}
      onCreate={(title, body) => void m.create(title, body)}
      onUpdate={(id, title, body) => void m.update(id, title, body)}
      onSetColor={(id, color) => void m.setColor(id, color)}
      onSetPinned={(id, pinned) => void m.setPinned(id, pinned)}
      onArchive={(id) => {
        if (window.confirm("이 메모를 삭제할까요? (휴지통에서 복구 가능)")) void m.archive(id);
      }}
    />
  );
}
