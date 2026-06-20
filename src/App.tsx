import { useCallback, useEffect, useMemo, useState } from "react";
import { api } from "./api/client";
import { buildTree, rowId, type TreeRow } from "./domain/tree";
import type { Business, Document, Project } from "./domain/types";
import { Sidebar } from "./components/Sidebar";
import { businessColor } from "./ui/colors";
import { MainView, type ViewKind } from "./views/MainView";

export default function App() {
  const [businesses, setBusinesses] = useState<Business[]>([]);
  const [projects, setProjects] = useState<Project[]>([]);
  const [documents, setDocuments] = useState<Document[]>([]);
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

  const loadDocuments = useCallback(async (businessId: number) => {
    try {
      const list = await api.document.list(businessId);
      setDocuments((prev) => [...prev.filter((d) => d.businessId !== businessId), ...list]);
    } catch (e) {
      setError(String(e));
    }
  }, []);

  useEffect(() => {
    void loadBusinesses();
  }, [loadBusinesses]);

  const tree = useMemo(
    () => buildTree({ businesses, projects, documents, deliverables: [], expanded }),
    [businesses, projects, documents, expanded],
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
          }
        }
        return next;
      });
    },
    [loadProjects, loadDocuments],
  );

  const onSelect = useCallback((row: TreeRow) => {
    setSelectedId(row.id);
    if (row.type === "dashboard" || row.type === "business") setView("dashboard");
    else if (row.type === "project") setView("kanban");
    else if (row.type === "document") setView("doc");
  }, []);

  const onAddBusiness = useCallback(async () => {
    try {
      const b = await api.business.create({ name: "새 사업", type: "etc" });
      await loadBusinesses();
      setExpanded((prev) => new Set(prev).add(rowId("business", b.id)));
      void loadProjects(b.id);
      void loadDocuments(b.id);
      setSelectedId(rowId("business", b.id));
      setView("dashboard");
    } catch (e) {
      setError(String(e));
    }
  }, [loadBusinesses, loadProjects, loadDocuments]);

  const onAddChild = useCallback(
    async (row: TreeRow) => {
      try {
        if (row.type === "business") {
          await api.project.create({ businessId: row.entityId, name: "새 프로젝트" });
          setExpanded((prev) => new Set(prev).add(row.id));
          await loadProjects(row.entityId);
        } else if (row.type === "project") {
          // 프로젝트 하위에는 문서 생성
          const proj = projects.find((p) => p.id === row.entityId);
          if (proj) {
            const d = await api.document.create({
              businessId: proj.businessId,
              projectId: proj.id,
              title: "제목 없음",
            });
            setExpanded((prev) => new Set(prev).add(row.id));
            await loadDocuments(proj.businessId);
            setSelectedId(rowId("document", d.id));
            setView("doc");
          }
        }
      } catch (e) {
        setError(String(e));
      }
    },
    [projects, loadProjects, loadDocuments],
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
        colorFor={colorFor}
        onSelect={onSelect}
        onToggle={onToggle}
        onAddBusiness={onAddBusiness}
        onAddChild={onAddChild}
        onRename={onRename}
      />
      <MainView
        business={selectedBusiness}
        project={selectedProject}
        document={selectedDocument}
        view={view}
        onViewChange={setView}
        error={error}
      />
    </div>
  );
}
