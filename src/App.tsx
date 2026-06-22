import { useCallback, useEffect, useMemo, useState } from "react";
import { api } from "./api/client";
import { buildTree, rowId, type TreeRow } from "./domain/tree";
import type { Business, BusinessType, Project } from "./domain/types";
import { Sidebar, type AddKind } from "./components/Sidebar";
import { GlobalSearch } from "./components/GlobalSearch";
import type { SearchHit } from "./domain/types";
import { businessColor } from "./ui/colors";
import { MainView, type ViewKind } from "./views/MainView";
import { useTheme } from "./hooks/useTheme";

export default function App() {
  const [businesses, setBusinesses] = useState<Business[]>([]);
  const [projects, setProjects] = useState<Project[]>([]);
  const [expanded, setExpanded] = useState<Set<string>>(new Set());
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [view, setView] = useState<ViewKind>("dashboard");
  // 검색 등에서 특정 문서를 바로 편집기로 열도록 전달하는 대상 id (열린 뒤 소비됨)
  const [pendingDocId, setPendingDocId] = useState<number | null>(null);
  const [error, setError] = useState<string | null>(null);
  const { theme, toggle: toggleTheme } = useTheme();
  const [typeFilter, setTypeFilter] = useState<Set<BusinessType>>(new Set());

  const onToggleType = useCallback((t: BusinessType) => {
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

  const loadProjects = useCallback(async (businessId: number) => {
    try {
      const list = await api.project.list(businessId);
      setProjects((prev) => [...prev.filter((p) => p.businessId !== businessId), ...list]);
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
    () => (typeFilter.size === 0 ? businesses : businesses.filter((b) => typeFilter.has(b.type))),
    [businesses, typeFilter],
  );

  const tree = useMemo(
    () => buildTree({ businesses: visibleBusinesses, projects, expanded }),
    [visibleBusinesses, projects, expanded],
  );

  const colorFor = useCallback(
    (entityId: number) => {
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
          if (row.type === "business") void loadProjects(row.entityId);
        }
        return next;
      });
    },
    [loadProjects],
  );

  const onSelect = useCallback((row: TreeRow) => {
    setSelectedId(row.id);
    if (row.type === "dashboard" || row.type === "business") setView("dashboard");
    else if (row.type === "project") setView("kanban");
    else if (row.type === "document") setView("doc");
    else if (row.type === "deliverable") setView("deliverables");
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

  const onAddChild = useCallback(
    async (row: TreeRow, kind: AddKind, name: string) => {
      // 트리 추가 메뉴는 사업 아래 프로젝트 생성만 담당한다.
      // (문서는 '문서' 노드의 '새 문서', 산출물은 '산출물' 노드의 '파일 업로드'로 생성)
      if (kind !== "project" || row.type !== "business") return;
      try {
        setExpanded((prev) => new Set(prev).add(row.id));
        const p = await api.project.create({ businessId: row.entityId, name });
        await loadProjects(row.entityId);
        setSelectedId(rowId("project", p.id));
        setView("kanban");
      } catch (e) {
        setError(String(e));
      }
    },
    [loadProjects],
  );

  const onArchive = useCallback(
    async (row: TreeRow) => {
      const ok = window.confirm(`'${row.label}'을(를) 보관할까요? (휴지통에서 복구 가능)`);
      if (!ok) return;
      try {
        if (row.type === "business") {
          await api.business.archive(row.entityId);
          await loadBusinesses();
        } else if (row.type === "project") {
          const p = projects.find((x) => x.id === row.entityId);
          await api.project.archive(row.entityId);
          if (p) await loadProjects(p.businessId);
        }
        if (selectedId === row.id) {
          setSelectedId(null);
          setView("dashboard");
        }
      } catch (e) {
        setError(String(e));
      }
    },
    [projects, selectedId, loadBusinesses, loadProjects],
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
        }
      } catch (e) {
        setError(String(e));
      }
    },
    [projects, loadBusinesses, loadProjects],
  );

  const onDataChanged = useCallback(async () => {
    await loadBusinesses();
    for (const b of businesses) await loadProjects(b.id);
  }, [businesses, loadBusinesses, loadProjects]);

  const navigateTo = useCallback(
    async (hit: SearchHit) => {
      const bizRow = rowId("business", hit.businessId);
      setExpanded((prev) => new Set(prev).add(bizRow));
      await loadProjects(hit.businessId);
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
    [loadProjects],
  );

  const selectedBusiness = useMemo(() => {
    const row = tree.find((r) => r.id === selectedId);
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
    return businesses[0] ?? null;
  }, [tree, selectedId, businesses, projects]);

  const selectedProject = useMemo(() => {
    const row = tree.find((r) => r.id === selectedId);
    if (row?.type === "project") return projects.find((x) => x.id === row.entityId) ?? null;
    return null;
  }, [tree, selectedId, projects]);

  return (
    <div style={{ display: "flex", height: "100%", overflow: "hidden" }}>
      <Sidebar
        rows={tree}
        selectedId={selectedId}
        header={<GlobalSearch onSearch={(q) => api.search(q)} onPick={navigateTo} />}
        typeFilter={typeFilter}
        onToggleType={onToggleType}
        colorFor={colorFor}
        onSelect={onSelect}
        onToggle={onToggle}
        onAddBusiness={onAddBusiness}
        onAddChild={onAddChild}
        onRename={onRename}
        onArchive={onArchive}
      />
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
      />
    </div>
  );
}
