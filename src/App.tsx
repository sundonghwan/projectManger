import { useCallback, useEffect, useMemo, useState } from "react";
import { api } from "./api/client";
import { businessTypeOptions, normalizeBusinessType } from "./domain/businessTypes";
import { buildTree, rowId, type TreeRow } from "./domain/tree";
import type { Business, BusinessType, Folder, FolderKind, Project } from "./domain/types";
import { Sidebar, type AddKind } from "./components/Sidebar";
import type { BusinessEditorInput } from "./components/BusinessEditorPopover";
import { GlobalSearch } from "./components/GlobalSearch";
import type { SearchHit } from "./domain/types";
import { businessColor } from "./ui/colors";
import { MainView, type ViewKind } from "./views/MainView";
import { useTheme } from "./hooks/useTheme";
import { Icon } from "./ui/icons/Icon";

export default function App() {
  const [businesses, setBusinesses] = useState<Business[]>([]);
  const [projects, setProjects] = useState<Project[]>([]);
  const [folders, setFolders] = useState<Folder[]>([]);
  const [expanded, setExpanded] = useState<Set<string>>(new Set());
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [view, setView] = useState<ViewKind>("dashboard");
  // 검색 등에서 특정 문서를 바로 편집기로 열도록 전달하는 대상 id (열린 뒤 소비됨)
  const [pendingDocId, setPendingDocId] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const { theme, toggle: toggleTheme } = useTheme();
  const [typeFilter, setTypeFilter] = useState<Set<string>>(new Set());
  const [sidebarCollapsed, setSidebarCollapsed] = useState(false);

  const onToggleType = useCallback((t: string) => {
    setTypeFilter((prev) => {
      const next = new Set(prev);
      if (next.has(t)) next.delete(t);
      else next.add(t);
      return next;
    });
  }, []);

  const loadBusinesses = useCallback(async () => {
    try {
      setBusinesses(await api.business.list());
      setError(null);
    } catch (e) {
      setError(String(e));
    }
  }, []);

  const loadProjects = useCallback(async (businessId: string) => {
    try {
      const list = await api.project.list(businessId);
      setProjects((prev) => [...prev.filter((p) => p.businessId !== businessId), ...list]);
    } catch (e) {
      setError(String(e));
    }
  }, []);

  const loadFolders = useCallback(async (businessId: string) => {
    try {
      const list = await api.folder.list(businessId);
      setFolders((prev) => [...prev.filter((f) => f.businessId !== businessId), ...list]);
    } catch (e) {
      setError(String(e));
    }
  }, []);

  useEffect(() => {
    void loadBusinesses();
  }, [loadBusinesses]);

  // 앱 시작 시 도래한 반복 태스크 자동 생성
  useEffect(() => {
    const today = new Date().toISOString().slice(0, 10);
    void api.recurring.generate(today).catch(() => {});
  }, []);

  const visibleBusinesses = useMemo(
    () =>
      typeFilter.size === 0
        ? businesses
        : businesses.filter((b) => typeFilter.has(normalizeBusinessType(b.type))),
    [businesses, typeFilter],
  );

  const typeOptions = useMemo(() => businessTypeOptions(businesses), [businesses]);

  const tree = useMemo(
    () => buildTree({ businesses: visibleBusinesses, projects, folders, expanded }),
    [visibleBusinesses, projects, folders, expanded],
  );

  const colorFor = useCallback(
    (entityId: string) => {
      const b = businesses.find((x) => x.id === entityId);
      return b ? businessColor(b.type, b.color) : "#94a3b8";
    },
    [businesses],
  );

  const onToggle = useCallback(
    (row: TreeRow) => {
      setExpanded((prev) => {
        const next = new Set(prev);
        if (next.has(row.id)) next.delete(row.id);
        else {
          next.add(row.id);
          if (row.type === "business") {
            void loadProjects(row.entityId);
            void loadFolders(row.entityId);
          }
        }
        return next;
      });
    },
    [loadProjects, loadFolders],
  );

  const onSelect = useCallback((row: TreeRow) => {
    setSelectedId(row.id);
    if (row.type === "dashboard" || row.type === "business") setView("dashboard");
    else if (row.type === "project") setView("kanban");
    else if (row.type === "document" || row.type === "docFolder") setView("doc");
    else if (row.type === "deliverable" || row.type === "delivFolder") setView("deliverables");
  }, []);

  const onAddBusiness = useCallback(
    async (type: BusinessType, name: string) => {
      try {
        const b = await api.business.create({ name, type });
        await loadBusinesses();
        setExpanded((prev) => new Set(prev).add(rowId("business", b.id)));
        void loadProjects(b.id);
        setSelectedId(rowId("business", b.id));
        setView("dashboard");
      } catch (e) {
        setError(String(e));
      }
    },
    [loadBusinesses, loadProjects],
  );

  const onUpdateBusiness = useCallback(
    async (input: BusinessEditorInput) => {
      try {
        await api.business.update(input);
        await loadBusinesses();
      } catch (e) {
        setError(String(e));
      }
    },
    [loadBusinesses],
  );

  const onAddChild = useCallback(
    async (row: TreeRow, kind: AddKind, name: string) => {
      try {
        // 1) 사업 아래 프로젝트 생성
        if (kind === "project" && row.type === "business") {
          setExpanded((prev) => new Set(prev).add(row.id));
          const p = await api.project.create({ businessId: row.entityId, name });
          await loadProjects(row.entityId);
          setSelectedId(rowId("project", p.id));
          setView("kanban");
          return;
        }
        // 2) 폴더 생성 — 진입 노드(문서/산출물) 아래 루트 폴더, 폴더 행 아래 하위 폴더
        if (kind === "folder") {
          let businessId: string;
          let folderKind: FolderKind;
          let parentId: string | null = null;
          if (row.type === "document" || row.type === "deliverable") {
            businessId = row.entityId; // 진입 노드의 entityId 는 사업 id
            folderKind = row.type === "document" ? "document" : "deliverable";
          } else if (row.type === "docFolder" || row.type === "delivFolder") {
            const parent = folders.find((f) => f.id === row.entityId);
            if (!parent) return;
            businessId = parent.businessId;
            folderKind = parent.kind;
            parentId = parent.id;
          } else {
            return;
          }
          setExpanded((prev) => new Set(prev).add(row.id));
          await api.folder.create({ businessId, kind: folderKind, parentId, name });
          await loadFolders(businessId);
        }
      } catch (e) {
        setError(String(e));
      }
    },
    [loadProjects, loadFolders, folders],
  );

  const onArchive = useCallback(
    async (row: TreeRow) => {
      try {
        if (row.type === "business") {
          if (!window.confirm(`'${row.label}'을(를) 보관할까요? (휴지통에서 복구 가능)`)) return;
          await api.business.archive(row.entityId);
          await loadBusinesses();
        } else if (row.type === "project") {
          if (!window.confirm(`'${row.label}'을(를) 보관할까요? (휴지통에서 복구 가능)`)) return;
          const p = projects.find((x) => x.id === row.entityId);
          await api.project.archive(row.entityId);
          if (p) await loadProjects(p.businessId);
        } else if (row.type === "docFolder" || row.type === "delivFolder") {
          if (!window.confirm(`폴더 '${row.label}'을(를) 삭제할까요? 안의 항목은 미분류로 이동하고, 하위 폴더도 함께 삭제됩니다.`))
            return;
          const f = folders.find((x) => x.id === row.entityId);
          await api.folder.remove(row.entityId);
          if (f) await loadFolders(f.businessId);
        } else {
          return;
        }
        if (selectedId === row.id) {
          setSelectedId(null);
          setView("dashboard");
        }
      } catch (e) {
        setError(String(e));
      }
    },
    [projects, folders, selectedId, loadBusinesses, loadProjects, loadFolders],
  );

  const onRename = useCallback(
    async (row: TreeRow, name: string) => {
      try {
        if (row.type === "business") {
          await api.business.rename(row.entityId, name);
          await loadBusinesses();
        } else if (row.type === "project") {
          const p = projects.find((x) => x.id === row.entityId);
          await api.project.rename(row.entityId, name);
          if (p) await loadProjects(p.businessId);
        } else if (row.type === "docFolder" || row.type === "delivFolder") {
          const f = folders.find((x) => x.id === row.entityId);
          await api.folder.rename(row.entityId, name);
          if (f) await loadFolders(f.businessId);
        }
      } catch (e) {
        setError(String(e));
      }
    },
    [projects, folders, loadBusinesses, loadProjects, loadFolders],
  );

  const onDataChanged = useCallback(async () => {
    await loadBusinesses();
    for (const b of businesses) {
      await loadProjects(b.id);
      await loadFolders(b.id);
    }
  }, [businesses, loadBusinesses, loadProjects, loadFolders]);

  const navigateTo = useCallback(
    async (hit: SearchHit) => {
      const bizRow = rowId("business", hit.businessId);
      setExpanded((prev) => new Set(prev).add(bizRow));
      await loadProjects(hit.businessId);
      void loadFolders(hit.businessId);
      if (hit.kind === "business") {
        setSelectedId(bizRow);
        setView("dashboard");
      } else if (hit.kind === "project") {
        setExpanded((prev) => new Set(prev).add(rowId("project", hit.id)));
        setSelectedId(rowId("project", hit.id));
        setView("kanban");
      } else if (hit.kind === "document") {
        // 문서는 단일 진입 노드(문서 목록) → 해당 문서를 자동으로 편집기로 연다.
        setSelectedId(rowId("document", hit.businessId));
        setPendingDocId(hit.id);
        setView("doc");
      } else if (hit.kind === "deliverable") {
        // 산출물도 단일 진입 노드(목록)로 이동.
        setSelectedId(rowId("deliverable", hit.businessId));
        setView("deliverables");
      } else {
        // task → 보드로 이동
        if (hit.projectId) {
          setExpanded((prev) => new Set(prev).add(rowId("project", hit.projectId!)));
          setSelectedId(rowId("project", hit.projectId));
        } else {
          setSelectedId(bizRow);
        }
        setView("kanban");
      }
    },
    [loadProjects, loadFolders],
  );

  const selectedRow = useMemo(() => tree.find((r) => r.id === selectedId) ?? null, [tree, selectedId]);

  const selectedBusiness = useMemo(() => {
    const row = selectedRow;
    if (!row) return businesses[0] ?? null;
    // 문서·산출물 진입 노드의 entityId 는 소속 사업 id (단일 진입 패턴).
    if (
      row.type === "business" ||
      row.type === "dashboard" ||
      row.type === "deliverable" ||
      row.type === "document"
    )
      return businesses.find((b) => b.id === row.entityId) ?? null;
    if (row.type === "project") {
      const p = projects.find((x) => x.id === row.entityId);
      return p ? (businesses.find((b) => b.id === p.businessId) ?? null) : null;
    }
    if (row.type === "docFolder" || row.type === "delivFolder") {
      const f = folders.find((x) => x.id === row.entityId);
      return f ? (businesses.find((b) => b.id === f.businessId) ?? null) : null;
    }
    return businesses[0] ?? null;
  }, [selectedRow, businesses, projects, folders]);

  const selectedProject = useMemo(() => {
    if (selectedRow?.type === "project") return projects.find((x) => x.id === selectedRow.entityId) ?? null;
    return null;
  }, [selectedRow, projects]);

  // 선택된 폴더 id (폴더 행 선택 시) — 진입 노드 선택이면 null(=전체).
  const selectedFolderId = useMemo(
    () => (selectedRow?.type === "docFolder" || selectedRow?.type === "delivFolder" ? selectedRow.entityId : null),
    [selectedRow],
  );

  // 현재 사업의 폴더 (뷰의 이동 드롭다운/필터용)
  const businessFolders = useMemo(
    () => folders.filter((f) => f.businessId === selectedBusiness?.id && !f.archivedAt),
    [folders, selectedBusiness],
  );

  return (
    <div style={{ display: "flex", height: "100%", overflow: "hidden" }}>
      {sidebarCollapsed ? (
        <div
          style={{
            width: 40,
            flexShrink: 0,
            height: "100%",
            background: "var(--sidebar)",
            borderRight: "1px solid var(--border)",
            display: "flex",
            flexDirection: "column",
            alignItems: "center",
            paddingTop: 8,
          }}
        >
          <button
            onClick={() => setSidebarCollapsed(false)}
            aria-label="사이드바 펼치기"
            title="사이드바 펼치기"
            style={{
              width: 28,
              height: 28,
              display: "flex",
              alignItems: "center",
              justifyContent: "center",
              border: "1px solid var(--border)",
              background: "var(--bg)",
              color: "var(--text2)",
              borderRadius: "var(--radius-md)",
              cursor: "pointer",
            }}
          >
            <Icon name="chevron-right" size={16} />
          </button>
        </div>
      ) : (
        <Sidebar
          rows={tree}
          selectedId={selectedId}
          header={<GlobalSearch onSearch={(q) => api.search(q)} onPick={navigateTo} />}
          businessTypeOptions={typeOptions}
          businesses={businesses}
          typeFilter={typeFilter}
          onToggleType={onToggleType}
          colorFor={colorFor}
          onSelect={onSelect}
          onToggle={onToggle}
          onAddBusiness={onAddBusiness}
          onUpdateBusiness={onUpdateBusiness}
          onAddChild={onAddChild}
          onRename={onRename}
          onArchive={onArchive}
          onCollapse={() => setSidebarCollapsed(true)}
        />
      )}
      <MainView
        business={selectedBusiness}
        project={selectedProject}
        view={view}
        onViewChange={setView}
        error={error}
        theme={theme}
        onToggleTheme={toggleTheme}
        onDataChanged={onDataChanged}
        openDocId={pendingDocId}
        onDocOpened={() => setPendingDocId(null)}
        folders={businessFolders}
        selectedFolderId={selectedFolderId}
      />
    </div>
  );
}
