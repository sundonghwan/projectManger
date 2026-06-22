import { open as openDialog } from "@tauri-apps/plugin-dialog";
import { useDeliverables } from "../hooks/useDeliverables";
import { DeliverableList } from "./DeliverableList";

export interface DeliverablesViewProps {
  businessId: number;
  projectId: number | null;
  onChanged: () => void;
}

/** 산출물 뷰 컨테이너 — 파일 선택 다이얼로그 + 데이터 로딩을 useDeliverables에 위임. */
export function DeliverablesView({ businessId, projectId, onChanged }: DeliverablesViewProps) {
  const d = useDeliverables(businessId, projectId, onChanged);

  const onUpload = async () => {
    const selected = await openDialog({ multiple: true, title: "산출물 파일 선택" });
    if (!selected) return;
    const paths = Array.isArray(selected) ? selected : [selected];
    await d.upload(paths);
  };

  return (
    <DeliverableList
      deliverables={d.deliverables}
      error={d.error}
      onUpload={() => void onUpload()}
      onSetStatus={(id, s) => void d.setStatus(id, s)}
      onRename={(id, title) => void d.rename(id, title)}
      onOpen={(deliv) => void d.open(deliv)}
      onArchive={(id) => void d.archive(id)}
    />
  );
}
