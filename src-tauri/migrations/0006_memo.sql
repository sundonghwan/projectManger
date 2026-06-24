-- 0006 — 사업별 메모(Google Keep식). 색상·고정·보관.
CREATE TABLE memo (
    id          INTEGER PRIMARY KEY,
    business_id INTEGER NOT NULL REFERENCES business(id) ON DELETE CASCADE,
    title       TEXT    NOT NULL DEFAULT '',
    body        TEXT    NOT NULL DEFAULT '',
    color       TEXT,                        -- 팔레트 키(default/red/…) NULL=기본
    pinned      INTEGER NOT NULL DEFAULT 0,  -- 0/1
    sort_order  REAL    NOT NULL DEFAULT 0,
    archived_at TEXT,
    created_at  TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at  TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);
CREATE INDEX idx_memo_business ON memo(business_id);
