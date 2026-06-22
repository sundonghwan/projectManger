import type { Label } from "../domain/types";
import { Icon } from "../ui/icons/Icon";

export interface LabelChipsProps {
  labels?: Label[];
  onRemove?: (label: Label) => void;
}

/** 태스크 라벨 칩 묶음. onRemove 가 있으면 × 로 제거 가능. */
export function LabelChips({ labels, onRemove }: LabelChipsProps) {
  if (!labels || labels.length === 0) return null;
  return (
    <span style={{ display: "inline-flex", flexWrap: "wrap", gap: 4 }}>
      {labels.map((l) => {
        const color = l.color ?? "#94a3b8";
        return (
          <span
            key={l.id}
            data-testid={`label-${l.id}`}
            style={{
              display: "inline-flex",
              alignItems: "center",
              gap: 4,
              fontSize: 11,
              fontWeight: 500,
              padding: "2px 7px",
              borderRadius: 4,
              background: color + "22",
              color,
            }}
          >
            {l.name}
            {onRemove && (
              <button
                aria-label={`${l.name} 라벨 제거`}
                onClick={(e) => {
                  e.stopPropagation();
                  onRemove(l);
                }}
                style={{ display: "inline-flex", alignItems: "center", border: "none", background: "transparent", color, cursor: "pointer", padding: 0, lineHeight: 1 }}
              >
                <Icon name="close" size={11} strokeWidth={2} />
              </button>
            )}
          </span>
        );
      })}
    </span>
  );
}
