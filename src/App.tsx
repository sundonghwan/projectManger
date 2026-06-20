import { useCallback, useEffect, useMemo, useState } from "react";
import { api } from "./api/client";
import { buildTree, rowId, type TreeRow } from "./domain/tree";
import type { Business, Project } from "./domain/types";
import { Sidebar } from "./components/Sidebar";
import { businessColor } from "./ui/colors";
import { MainView, type ViewKind } from "./views/MainView";

export default function App() {
  const [businesses, setBusinesses] = useState<Business[]>([]);
  const [projects, setProjects] = useState<Project[]>([]);
  const [expanded, setExpanded] = useState<Set<string>>(new Set());
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [view, setView] = useState<ViewKind>("dashboard");
  const [error, setError] = useState<string | null>(null);

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

  const tree = useMemo(
    () => buildTree({ businesses, projects, documents: [], deliverables: [], expanded }),
    [businesses, projects, expanded],
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
  }, []);

  const onAddBusiness = useCallback(async () => {
    try {
      const b = await api.business.create({ name: "새 사업", type: "etc" });
      await loadBusinesses();
      setExpanded((prev) => new Set(prev).add(rowId("business", b.id)));
      void loadProjects(b.id);
      setSelectedId(rowId("business", b.id));
      setView("dashboard");
    } catch (e) {
      setError(String(e));
    }
  }, [loadBusinesses, loadProjects]);

  const onAddChild = useCallback(
    async (row: TreeRow) => {
      if (row.type === "business") {
        try {
          await api.project.create({ businessId: row.entityId, name: "새 프로젝트" });
          setExpanded((prev) => new Set(prev).add(row.id));
          await loadProjects(row.entityId);
        } catch (e) {
          setError(String(e));
        }
      }
      // 프로젝트 하위(문서·산출물) 생성은 다음 단계에서 지원
    },
    [loadProjects],
  );

  const selectedBusiness = useMemo(() => {
    const row = tree.find((r) => r.id === selectedId);
    if (!row) return businesses[0] ?? null;
    if (row.type === "business" || row.type === "dashboard")
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
        colorFor={colorFor}
        onSelect={onSelect}
        onToggle={onToggle}
        onAddBusiness={onAddBusiness}
        onAddChild={onAddChild}
      />
      <MainView
        business={selectedBusiness}
        project={selectedProject}
        view={view}
        onViewChange={setView}
        error={error}
      />
    </div>
  );
}
