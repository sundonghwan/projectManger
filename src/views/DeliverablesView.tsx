import { useDeliverables } from "../hooks/useDeliverables";
import { DeliverableList } from "./DeliverableList";

export interface DeliverablesViewProps {
  businessId: number;
  projectId: number | null;
  onChanged: () => void;
}

/** 산출물 뷰 컨테이너 — 데이터 로딩을 useDeliverables에 위임. */
export function DeliverablesView({ businessId, projectId, onChanged }: DeliverablesViewProps) {
  const d = useDeliverables(businessId, projectId, onChanged);
  return (
    <DeliverableList
      deliverables={d.deliverables}
      selectedId={d.selectedId}
      versions={d.versions}
      onSelect={(id) => void d.select(id)}
      onCreate={() => void d.create()}
      onSetStatus={(id, s) => void d.setStatus(id, s)}
      onAddVersion={(id) => void d.addVersion(id)}
      onArchive={(id) => void d.archive(id)}
    />
  );
}
