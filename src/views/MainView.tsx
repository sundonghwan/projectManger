import { useState, useEffect, useRef } from "react";
import { api } from "../api/client";
import type { Business, Folder, Project, TrashItem } from "../domain/types";
import { TrashPanel } from "./TrashPanel";
import { businessColor } from "../ui/colors";
import { Icon } from "../ui/icons/Icon";
import { useTasks } from "../hooks/useTasks";
import { dashboardStats } from "../domain/dashboard";
import { groupByStatus } from "../domain/kanban";
import { Dashboard } from "./Dashboard";
import { Kanban } from "./Kanban";
import { TaskList } from "./TaskList";
import { DocumentsView } from "./DocumentsView";
import { TaskEditor } from "./TaskEditor";
import { DeliverablesView } from "./DeliverablesView";
import { MemosView } from "./MemosView";
import { ServerView } from "./ServerView";
import { TimelineView } from "./TimelineView";
import { AutomationModal } from "./AutomationModal";
import { open as openDialog } from "@tauri-apps/plugin-dialog";

export type ViewKind =
  | "dashboard"
  | "kanban"
  | "list"
  | "timeline"
  | "doc"
  | "deliverables"
  | "memo"
  | "ssh";

const TABS: { key: ViewKind; label: string }[] = [
  { key: "dashboard", label: "대시보드" },
  { key: "kanban", label: "칸반" },
  { key: "list", label: "리스트" },
  { key: "timeline", label: "타임라인" },
  { key: "doc", label: "문서" },
  { key: "deliverables", label: "산출물" },
  { key: "memo", label: "메모" },
  { key: "ssh", label: "터미널" },
];

const VIEW_LABEL: Record<ViewKind, string> = {
  dashboard: "대시보드",
  kanban: "칸반",
  list: "리스트",
  timeline: "타임라인",
  doc: "문서",
  deliverables: "산출물",
  memo: "메모",
  ssh: "SSH 터미널",
};

export interface MainViewProps {
  business: Business | null;
  project: Project | null;
  view: ViewKind;
  onViewChange: (v: ViewKind) => void;
  error: string | null;
  theme: "light" | "dark";
  onToggleTheme: () => void;
  onDataChanged: () => void;
  /** 검색 등에서 문서 뷰 진입 시 자동으로 열 문서 id */
  openDocId?: string | null;
  onDocOpened?: () => void;
  /** 현재 사업의 분류 폴더(문서+산출물) */
  folders?: Folder[];
  /** 사이드바에서 선택된 폴더 id (없으면 진입 노드=전체) */
  selectedFolderId?: string | null;
}

export function MainView({
  business,
  project,
  view,
  onViewChange,
  error,
  theme,
  onToggleTheme,
  onDataChanged,
  openDocId,
  onDocOpened,
  folders = [],
  selectedFolderId = null,
}: MainViewProps) {
  const {
    tasks,
    labelsByTask,
    error: taskError,
    create,
    move,
    toggleDone,
    update,
    archive,
    assignLabel,
    removeLabel,
  } = useTasks(business?.id ?? null, project?.id ?? null);
  const [editingTaskId, setEditingTaskId] = useState<string | null>(null);
  const [trashOpen, setTrashOpen] = useState(false);
  const [trashItems, setTrashItems] = useState<TrashItem[]>([]);
  const [automationOpen, setAutomationOpen] = useState(false);
  const [vaultPath, setVaultPath] = useState<string | null>(null);
  const [vaultPopover, setVaultPopover] = useState(false);
  const vaultBtnRef = useRef<HTMLButtonElement>(null);

  useEffect(() => {
    api.vault.path().then(setVaultPath).catch(() => {});
  }, []);

  const changeVault = async () => {
    const picked = await openDialog({ directory: true, multiple: false, title: "Vault 폴더 선택" });
    if (typeof picked === "string") {
      await api.vault.set(picked);
      setVaultPath(picked);
      window.location.reload();
    }
  };
  const editingTask = tasks.find((t) => t.id === editingTaskId) ?? null;
  const shownError = error ?? taskError;

  const openTrash = async () => {
    setTrashItems(await api.trash.list());
    setTrashOpen(true);
  };
  const restoreItem = async (item: TrashItem) => {
    await api.trash.restore(item.kind, item.id);
    setTrashItems(await api.trash.list());
    onDataChanged();
  };
  const purgeItem = async (item: TrashItem) => {
    await api.trash.purge(item.kind, item.id);
    setTrashItems(await api.trash.list());
    onDataChanged();
  };

  return (
    <section
      style={{
        flex: 1,
        height: "100%",
        display: "flex",
        flexDirection: "column",
        minWidth: 0,
        background: "var(--bg)",
      }}
    >
      <div style={topbar}>
        {business ? (
          <div style={{ display: "flex", alignItems: "center", gap: 6 }}>
            <span
              style={{
                width: 8,
                height: 8,
                borderRadius: "50%",
                background: businessColor(business.type, business.color),
              }}
            />
            <span style={{ fontSize: 13, fontWeight: 600 }}>{business.name}</span>
            {project && (
              <>
                <Icon name="chevron-right" size={14} style={{ color: "var(--text3)" }} />
                <span style={{ fontSize: 13 }}>{project.name}</span>
              </>
            )}
            <Icon name="chevron-right" size={14} style={{ color: "var(--text3)" }} />
            <span style={{ fontSize: 13, color: "var(--text2)" }}>{VIEW_LABEL[view]}</span>
          </div>
        ) : (
          <span style={{ fontSize: 13, color: "var(--text2)" }}>선택된 항목 없음</span>
        )}
        <span style={{ flex: 1 }} />
        <div style={{ display: "flex", alignItems: "center", gap: 8 }}>
          <button
            onClick={onToggleTheme}
            style={iconBtn}
            aria-label="테마 전환"
            title={theme === "light" ? "다크 모드" : "라이트 모드"}
          >
            <Icon name={theme === "light" ? "moon" : "sun"} size={16} />
          </button>
          {business && (
            <button onClick={() => setAutomationOpen(true)} style={iconBtn} aria-label="자동화" title="템플릿·반복 태스크">
              <Icon name="settings" size={16} />
            </button>
          )}
          <div style={{ position: "relative" }}>
            <button
              ref={vaultBtnRef}
              onClick={() => setVaultPopover((v) => !v)}
              style={iconBtn}
              aria-label="Vault 설정"
              title="Vault 폴더 설정"
            >
              <Icon name="folder" size={16} />
            </button>
            {vaultPopover && (
              <div style={vaultPopoverStyle}>
                <div style={{ fontSize: 12, fontWeight: 600, color: "var(--text)", marginBottom: 4 }}>
                  Vault 폴더
                </div>
                <div style={{ fontSize: 11, color: "var(--text2)", wordBreak: "break-all", marginBottom: 8 }}>
                  {vaultPath ?? "기본 위치(appData)"}
                </div>
                <div style={{ fontSize: 11, color: "var(--text3)", marginBottom: 10, lineHeight: 1.5 }}>
                  iCloud Drive / Dropbox 등 동기화 폴더에 두면<br />여러 기기에서 동기화됩니다.
                </div>
                <button onClick={() => { setVaultPopover(false); void changeVault(); }} style={vaultChangeBtn}>
                  Vault 변경
                </button>
              </div>
            )}
          </div>
          <button onClick={() => void openTrash()} style={iconBtn} aria-label="휴지통" title="휴지통">
            <Icon name="trash" size={16} />
          </button>
        </div>
      </div>

      {business && (
        <div style={tabbar}>
          {TABS.map((t) => {
            const active = t.key === view;
            return (
              <button
                key={t.key}
                onClick={() => onViewChange(t.key)}
                style={{
                  ...tabStyle,
                  color: active ? "var(--text)" : "var(--text2)",
                  fontWeight: active ? 600 : 500,
                  borderBottom: `2px solid ${active ? "var(--accent)" : "transparent"}`,
                }}
              >
                {t.label}
              </button>
            );
          })}
        </div>
      )}

      <div style={{ flex: 1, overflow: "auto", minHeight: 0 }}>
        {shownError && (
          <div style={errorBar}>
            <Icon name="alert" size={15} />
            <span>{shownError}</span>
          </div>
        )}
        {!business ? (
          <Empty />
        ) : view === "dashboard" ? (
          <Dashboard business={business} stats={dashboardStats(tasks)} />
        ) : view === "kanban" ? (
          <Kanban
            columns={groupByStatus(tasks)}
            onMove={move}
            onAddTask={(s) => create(s)}
            labelsByTask={labelsByTask}
            onCardClick={setEditingTaskId}
          />
        ) : view === "list" ? (
          <TaskList
            tasks={tasks}
            onToggleDone={toggleDone}
            labelsByTask={labelsByTask}
            onRowClick={setEditingTaskId}
          />
        ) : view === "timeline" ? (
          <TimelineView tasks={tasks} />
        ) : view === "doc" ? (
          <DocumentsView
            key={business.id}
            businessId={business.id}
            projectId={project?.id ?? null}
            onChanged={onDataChanged}
            initialOpenDocId={openDocId}
            onOpened={onDocOpened}
            folders={folders.filter((f) => f.kind === "document")}
            selectedFolderId={view === "doc" ? selectedFolderId : null}
          />
        ) : view === "deliverables" ? (
          <DeliverablesView
            key={business.id}
            businessId={business.id}
            projectId={project?.id ?? null}
            onChanged={onDataChanged}
            folders={folders.filter((f) => f.kind === "deliverable")}
            selectedFolderId={view === "deliverables" ? selectedFolderId : null}
          />
        ) : view === "memo" ? (
          <MemosView key={business.id} businessId={business.id} onChanged={onDataChanged} />
        ) : view === "ssh" ? null : (
          <div style={{ padding: "24px 28px", color: "var(--text2)" }}>
            <strong style={{ color: "var(--text)" }}>{VIEW_LABEL[view]}</strong> — 다음 단계에서 지원
          </div>
        )}

        {/* SSH 뷰는 탭을 옮겨도 세션·터미널 스크롤백이 유지되도록 언마운트하지 않고 숨김 처리한다. */}
        {business && (
          <div style={{ height: "100%", display: view === "ssh" ? "block" : "none" }}>
            <ServerView key={business.id} businessId={business.id} projectId={project?.id ?? null} />
          </div>
        )}
      </div>

      {editingTask && (
        <TaskEditor
          task={editingTask}
          labels={labelsByTask[editingTask.id] ?? []}
          onSave={(patch) => {
            void update({ id: editingTask.id, ...patch });
            setEditingTaskId(null);
          }}
          onAddLabel={(name, color) => void assignLabel(editingTask.id, name, color)}
          onRemoveLabel={(l) => void removeLabel(editingTask.id, l.id)}
          onArchive={() => {
            void archive(editingTask.id);
            setEditingTaskId(null);
          }}
          onClose={() => setEditingTaskId(null)}
        />
      )}

      {trashOpen && (
        <TrashPanel
          items={trashItems}
          onRestore={(item) => void restoreItem(item)}
          onPurge={(item) => void purgeItem(item)}
          onClose={() => setTrashOpen(false)}
        />
      )}

      {automationOpen && business && (
        <AutomationModal
          businessId={business.id}
          projectId={project?.id ?? null}
          onChanged={onDataChanged}
          onClose={() => setAutomationOpen(false)}
        />
      )}
    </section>
  );
}

function Empty() {
  return (
    <div style={{ padding: "60px 28px", textAlign: "center", color: "var(--text3)" }}>
      <div style={{ fontSize: 15, marginBottom: 8 }}>
        왼쪽에서 사업을 선택하거나 새로 만들어 시작하세요.
      </div>
    </div>
  );
}

const topbar: React.CSSProperties = {
  height: 44,
  flexShrink: 0,
  display: "flex",
  alignItems: "center",
  padding: "0 14px",
  borderBottom: "1px solid var(--border)",
};
const tabbar: React.CSSProperties = {
  height: 40,
  flexShrink: 0,
  display: "flex",
  gap: 2,
  padding: "0 10px",
  borderBottom: "1px solid var(--border)",
};
const tabStyle: React.CSSProperties = {
  padding: "0 11px",
  fontSize: 13,
  border: "none",
  background: "transparent",
  cursor: "pointer",
  marginBottom: -1,
};
const iconBtn: React.CSSProperties = {
  width: 30,
  height: 28,
  display: "flex",
  alignItems: "center",
  justifyContent: "center",
  border: "1px solid var(--border)",
  background: "var(--bg)",
  color: "var(--text2)",
  borderRadius: "var(--radius-md)",
  fontSize: 13,
  cursor: "pointer",
};
const errorBar: React.CSSProperties = {
  display: "flex",
  alignItems: "center",
  gap: 6,
  margin: "12px 16px",
  padding: "8px 12px",
  borderRadius: "var(--radius-md)",
  background: "rgba(239,68,68,.1)",
  color: "#ef4444",
  fontSize: 13,
};
const vaultPopoverStyle: React.CSSProperties = {
  position: "absolute",
  top: "calc(100% + 6px)",
  right: 0,
  width: 240,
  padding: "12px 14px",
  background: "var(--bg)",
  border: "1px solid var(--border)",
  borderRadius: "var(--radius-md)",
  boxShadow: "0 4px 16px rgba(0,0,0,.12)",
  zIndex: 100,
};
const vaultChangeBtn: React.CSSProperties = {
  width: "100%",
  padding: "5px 10px",
  fontSize: 12,
  border: "1px solid var(--border)",
  background: "var(--bg)",
  color: "var(--text2)",
  borderRadius: "var(--radius-md)",
  cursor: "pointer",
};
