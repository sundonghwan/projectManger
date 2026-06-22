import { useCallback, useEffect, useState, type CSSProperties } from "react";
import { api } from "../api/client";
import type { RecurringTask, Template } from "../domain/types";
import { Icon } from "../ui/icons/Icon";
import { RecurringPanel, type RecurringFormData } from "./RecurringPanel";
import { TemplatePanel, type TemplateFormData } from "./TemplatePanel";

export interface AutomationModalProps {
  businessId: number;
  projectId: number | null;
  onChanged: () => void;
  onClose: () => void;
}

/** 템플릿 + 반복 태스크 관리 모달. */
export function AutomationModal({ businessId, projectId, onChanged, onClose }: AutomationModalProps) {
  const [templates, setTemplates] = useState<Template[]>([]);
  const [recurring, setRecurring] = useState<RecurringTask[]>([]);
  const [msg, setMsg] = useState<string | null>(null);

  const reload = useCallback(async () => {
    setTemplates(await api.template.list());
    setRecurring(await api.recurring.list(businessId));
  }, [businessId]);

  useEffect(() => {
    void reload();
  }, [reload]);

  const applyTemplate = async (t: Template) => {
    try {
      if (t.kind === "project") await api.template.applyProject(t.id, businessId);
      else await api.template.applyDocument(t.id, businessId, projectId);
      setMsg(`"${t.name}" 적용됨`);
      onChanged();
    } catch (e) {
      setMsg(String(e));
    }
  };
  const createTemplate = async (d: TemplateFormData) => {
    try {
      await api.template.create(d.name, d.kind, d.payload);
      await reload();
    } catch (e) {
      setMsg(String(e));
    }
  };
  const deleteTemplate = async (id: number) => {
    await api.template.delete(id);
    await reload();
  };

  const createRecurring = async (d: RecurringFormData) => {
    try {
      await api.recurring.create({ businessId, projectId, ...d });
      await reload();
    } catch (e) {
      setMsg(String(e));
    }
  };
  const toggleRecurring = async (r: RecurringTask) => {
    await api.recurring.setActive(r.id, r.active !== 1);
    await reload();
  };
  const deleteRecurring = async (id: number) => {
    await api.recurring.delete(id);
    await reload();
  };
  const generate = async () => {
    const today = new Date().toISOString().slice(0, 10);
    const n = await api.recurring.generate(today);
    setMsg(`${n}개 생성됨`);
    await reload();
    onChanged();
  };

  return (
    <div style={overlay} onClick={onClose} data-testid="automation-overlay">
      <div style={modal} role="dialog" aria-label="자동화" onClick={(e) => e.stopPropagation()}>
        <div style={{ display: "flex", alignItems: "center", marginBottom: 14 }}>
          <span style={{ fontSize: 15, fontWeight: 600, flex: 1 }}>자동화 (템플릿 · 반복)</span>
          <button aria-label="닫기" onClick={onClose} style={{ display: "inline-flex", alignItems: "center", justifyContent: "center", border: "none", background: "transparent", color: "var(--text2)", cursor: "pointer", padding: 0 }}><Icon name="close" size={16} /></button>
        </div>
        {msg && <div style={{ fontSize: 12, color: "var(--text2)", marginBottom: 10 }}>{msg}</div>}
        <TemplatePanel templates={templates} onApply={applyTemplate} onCreate={createTemplate} onDelete={deleteTemplate} />
        <div style={{ height: 1, background: "var(--border)", margin: "16px 0" }} />
        <RecurringPanel items={recurring} onCreate={createRecurring} onToggle={toggleRecurring} onDelete={deleteRecurring} onGenerate={generate} />
      </div>
    </div>
  );
}

const overlay: CSSProperties = { position: "fixed", inset: 0, background: "rgba(0,0,0,.4)", display: "flex", alignItems: "center", justifyContent: "center", zIndex: 100 };
const modal: CSSProperties = { width: 540, maxWidth: "92vw", maxHeight: "85vh", overflowY: "auto", background: "var(--card)", border: "1px solid var(--border)", borderRadius: "var(--radius-lg)", boxShadow: "var(--shadow-modal)", padding: 20 };
