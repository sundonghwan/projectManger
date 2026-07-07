import { open as openDialog } from "@tauri-apps/plugin-dialog";
import { useDeliverables } from "../hooks/useDeliverables";
import { DeliverableList } from "./DeliverableList";
import type { Folder } from "../domain/types";

export interface DeliverablesViewProps {
  businessId: string;
  projectId: string | null;
  onChanged: () => void;
  /** 이 사업의 산출물 폴더(이동 드롭다운/필터용) */
  folders?: Folder[];
  /** 선택된 폴더 id (없으면 전체). 새 업로드는 이 폴더에 들어간다. */
  selectedFolderId?: string | null;
}

/** 산출물 뷰 컨테이너 — 파일 선택 다이얼로그 + 데이터 로딩을 useDeliverables에 위임. */
export function DeliverablesView({
  businessId,
  projectId,
  onChanged,
  folders = [],
  selectedFolderId = null,
}: DeliverablesViewProps) {
  const d = useDeliverables(businessId, projectId, onChanged);

  // 선택된 폴더가 있으면 그 폴더 직속만, 없으면 전체.
  const shown = selectedFolderId == null
    ? d.deliverables
    : d.deliverables.filter((x) => x.folderId === selectedFolderId);

  const onUpload = async () => {
    const selected = await openDialog({ multiple: true, title: "산출물 파일 선택" });
    if (!selected) return;
    const paths = Array.isArray(selected) ? selected : [selected];
    await d.upload(paths, selectedFolderId);
  };

  return (
    <DeliverableList
      deliverables={shown}
      error={d.error}
      uploading={d.uploading}
      folders={folders}
      currentFolderId={selectedFolderId}
      onUpload={() => void onUpload()}
      onDropFiles={(paths) => void d.upload(paths, selectedFolderId)}
      onSetStatus={(id, s) => void d.setStatus(id, s)}
      onRename={(id, title) => void d.rename(id, title)}
      onMove={(id, folderId) => void d.move(id, folderId)}
      onOpen={(deliv) => void d.open(deliv)}
      onArchive={(id) => {
        if (window.confirm("이 산출물을 삭제할까요? (휴지통에서 복구 가능)")) void d.archive(id);
      }}
    />
  );
}
