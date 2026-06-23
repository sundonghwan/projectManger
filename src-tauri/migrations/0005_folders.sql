-- 0005 — 산출물·문서 폴더(분류). 공용 folder 테이블 + kind 구분자(A안).
-- 최대 2단계: parent_id 가 있으면 그 부모는 루트(parent_id IS NULL)여야 한다(검증은 repo).
-- 폴더 삭제 시 자식 폴더는 CASCADE, 항목은 folder_id→NULL(미분류). 물리 파일은 불변.

CREATE TABLE folder (
    id          INTEGER PRIMARY KEY,
    business_id INTEGER NOT NULL REFERENCES business(id) ON DELETE CASCADE,
    kind        TEXT    NOT NULL CHECK (kind IN ('document', 'deliverable')),
    parent_id   INTEGER REFERENCES folder(id) ON DELETE CASCADE,
    name        TEXT    NOT NULL,
    sort_order  REAL    NOT NULL DEFAULT 0,
    archived_at TEXT,
    created_at  TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at  TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);
CREATE INDEX idx_folder_business ON folder(business_id);
CREATE INDEX idx_folder_parent   ON folder(parent_id);

ALTER TABLE document    ADD COLUMN folder_id INTEGER REFERENCES folder(id) ON DELETE SET NULL;
ALTER TABLE deliverable ADD COLUMN folder_id INTEGER REFERENCES folder(id) ON DELETE SET NULL;
