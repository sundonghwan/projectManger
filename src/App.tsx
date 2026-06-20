import { useCallback, useEffect, useMemo, useState } from "react";
import { api } from "./api/client";
import { buildTree, rowId, type TreeRow } from "./domain/tree";
import type { Business, BusinessType, Deliverable, Document, Project } from "./domain/types";
import { Sidebar, type AddKind } from "./components/Sidebar";
import { GlobalSearch } from "./components/GlobalSearch";
import type { SearchHit } from "./domain/types";
import { businessColor } from "./ui/colors";
import { MainView, type ViewKind } from "./views/MainView";
import { useTheme } from "./hooks/useTheme";

export default function App() {
  const [businesses, setBusinesses] = useState<Business[]>([]);
  const [projects, setProjects] = useState<Project[]>([]);
  const [documents, setDocuments] = useState<Document[]>([]);
  const [deliverables, setDeliverables] = useState<Deliverable[]>([]);
  const [expanded, setExpanded] = useState<Set<string>>(new Set());
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [view, setView] = useState<ViewKind>("dashboard");
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

  const loadDocuments = useCallback(async (businessId: number) => {
    try {
      const list = await api.document.list(businessId);
      setDocuments((prev) => [...prev.filter((d) => d.businessId !== businessId), ...list]);
    } catch (e) {
      setError(String(e));
    }
  }, []);

  const loadDeliverables = useCallback(async (businessId: number) => {
    try {
      const list = await api.deliverable.list(businessId);
      setDeliverables((prev) => [...prev.filter((d) => d.businessId !== businessId), ...list]);
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
    () => buildTree({ businesses: visibleBusinesses, projects, documents, deliverables, expanded }),
    [visibleBusinesses, projects, documents, deliverables, expanded],
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
          if (row.type === "business") {
            void loadProjects(row.entityId);
            void loadDocuments(row.entityId);
            void loadDeliverables(row.entityId);
          }
        }
        return next;
      });
    },
    [loadProjects, loadDocuments, loadDeliverables],
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
        void loadDocuments(b.id);
        void loadDeliverables(b.id);
        setSelectedId(rowId("business", b.id));
        setView("dashboard");
      } catch (e) {
        setError(String(e));
      }
    },
    [loadBusinesses, loadProjects, loadDocuments, loadDeliverables],
  );

  const onAddChild = useCallback(
    async (row: TreeRow, kind: AddKind, name: string) => {
      // 추가 위치의 사업/프로젝트 해석
      let businessId: number;
      let projectId: number | null;
      if (row.type === "business") {
        businessId = row.entityId;
        projectId = null;
      } else {
        const proj = projects.find((p) => p.id === row.entityId);
        if (!proj) return;
        businessId = proj.businessId;
        projectId = proj.id;
      }
      try {
        setExpanded((prev) => new Set(prev).add(row.id));
        if (kind === "project") {
          const p = await api.project.create({ businessId, name });
          await loadProjects(businessId);
          setSelectedId(rowId("project", p.id));
          setView("kanban");
        } else if (kind === "document") {
          const d = await api.document.create({ businessId, projectId, title: name });
          await loadDocuments(businessId);
          setSelectedId(rowId("document", d.id));
          setView("doc");
        } else {
          const dv = await api.deliverable.create({
            businessId,
            projectId,
            title: name,
            kind: "file",
          });
          await loadDeliverables(businessId);
          setSelectedId(rowId("deliverable", dv.id));
          setView("deliverables");
        }
      } catch (e) {
        setError(String(e));
      }
    },
    [projects, loadProjects, loadDocuments, loadDeliverables],
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
        } else if (row.type === "document") {
          const d = documents.find((x) => x.id === row.entityId);
          await api.document.archive(row.entityId);
          if (d) await loadDocuments(d.businessId);
        } else if (row.type === "deliverable") {
          const dv = deliverables.find((x) => x.id === row.entityId);
          await api.deliverable.archive(row.entityId);
          if (dv) await loadDeliverables(dv.businessId);
        }
        if (selectedId === row.id) {
          setSelectedId(null);
          setView("dashboard");
        }
      } catch (e) {
        setError(String(e));
      }
    },
    [projects, documents, deliverables, selectedId, loadBusinesses, loadProjects, loadDocuments, loadDeliverables],
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
        } else if (row.type === "document") {
          const d = documents.find((x) => x.id === row.entityId);
          await api.document.rename(row.entityId, name);
          if (d) await loadDocuments(d.businessId);
        }
      } catch (e) {
        setError(String(e));
      }
    },
    [projects, documents, loadBusinesses, loadProjects, loadDocuments],
  );

  const onDataChanged = useCallback(async () => {
    await loadBusinesses();
    for (const b of businesses) {
      await loadProjects(b.id);
      await loadDocuments(b.id);
      await loadDeliverables(b.id);
    }
  }, [businesses, loadBusinesses, loadProjects, loadDocuments, loadDeliverables]);

  const navigateTo = useCallback(
    async (hit: SearchHit) => {
      const bizRow = rowId("business", hit.businessId);
      setExpanded((prev) => new Set(prev).add(bizRow));
      await Promise.all([loadProjects(hit.businessId), loadDocuments(hit.businessId)]);
      if (hit.kind === "business") {
        setSelectedId(bizRow);
        setView("dashboard");
      } else if (hit.kind === "project") {
        setExpanded((prev) => new Set(prev).add(rowId("project", hit.id)));
        setSelectedId(rowId("project", hit.id));
        setView("kanban");
      } else if (hit.kind === "document") {
        if (hit.projectId) setExpanded((prev) => new Set(prev).add(rowId("project", hit.projectId!)));
        setSelectedId(rowId("document", hit.id));
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
    [loadProjects, loadDocuments],
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
    if (row.type === "document") {
      const d = documents.find((x) => x.id === row.entityId);
      return d ? (businesses.find((b) => b.id === d.businessId) ?? null) : null;
    }
    return businesses[0] ?? null;
  }, [tree, selectedId, businesses, projects, documents]);

  const selectedProject = useMemo(() => {
    const row = tree.find((r) => r.id === selectedId);
    if (row?.type === "project") return projects.find((x) => x.id === row.entityId) ?? null;
    return null;
  }, [tree, selectedId, projects]);

  const selectedDocument = useMemo(() => {
    const row = tree.find((r) => r.id === selectedId);
    if (row?.type === "document") return documents.find((x) => x.id === row.entityId) ?? null;
    return null;
  }, [tree, selectedId, documents]);

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
        document={selectedDocument}
        view={view}
        onViewChange={setView}
        error={error}
        theme={theme}
        onToggleTheme={toggleTheme}
        onDataChanged={onDataChanged}
      />
    </div>
  );
}
