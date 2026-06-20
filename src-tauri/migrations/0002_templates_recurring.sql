-- 0002 — 템플릿 + 반복 태스크

-- 템플릿: 프로젝트/문서 구조를 저장해 재사용.
-- payload(JSON):
--   kind=project   → {"tasks":[{"title":..,"priority":..}], "documents":[{"title":..}]}
--   kind=document  → {"blocks":[{"type":..,"content":..}]}
CREATE TABLE template (
    id         INTEGER PRIMARY KEY,
    name       TEXT NOT NULL,
    kind       TEXT NOT NULL CHECK (kind IN ('project', 'document')),
    payload    TEXT NOT NULL DEFAULT '{}',
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

-- 반복 태스크: next_run 이 도래하면 task 를 생성하고 interval_days 만큼 다음으로 미룬다.
CREATE TABLE recurring_task (
    id            INTEGER PRIMARY KEY,
    business_id   INTEGER NOT NULL REFERENCES business(id) ON DELETE CASCADE,
    project_id    INTEGER REFERENCES project(id) ON DELETE CASCADE,
    title         TEXT    NOT NULL,
    priority      INTEGER NOT NULL DEFAULT 2 CHECK (priority BETWEEN 0 AND 4),
    interval_days INTEGER NOT NULL CHECK (interval_days >= 1),
    next_run      TEXT    NOT NULL,
    active        INTEGER NOT NULL DEFAULT 1,
    created_at    TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);
CREATE INDEX idx_recurring_business ON recurring_task(business_id);
CREATE INDEX idx_recurring_next ON recurring_task(next_run);
