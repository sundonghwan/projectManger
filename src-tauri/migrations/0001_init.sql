-- 0001_init — 초기 스키마 (사업>프로젝트 계층 + 태스크/문서/산출물/서버연결)
-- 설계 근거는 비공개 docs/02-데이터모델.md 참고. 이 파일이 빌드/런타임 기준.

PRAGMA foreign_keys = ON;

-- 1. 사업 (Business) — 최상위 계층
CREATE TABLE business (
    id           INTEGER PRIMARY KEY,
    name         TEXT    NOT NULL,
    type         TEXT    NOT NULL DEFAULT 'etc'
                 CHECK (type IN ('si', 'internal', 'ops', 'etc')),
    color        TEXT,
    description  TEXT,
    status       TEXT    NOT NULL DEFAULT 'active'
                 CHECK (status IN ('active', 'onhold', 'done')),
    sort_order   REAL    NOT NULL DEFAULT 0,
    archived_at  TEXT,
    created_at   TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at   TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

-- 2. 프로젝트 (Project)
CREATE TABLE project (
    id           INTEGER PRIMARY KEY,
    business_id  INTEGER NOT NULL REFERENCES business(id) ON DELETE CASCADE,
    name         TEXT    NOT NULL,
    description  TEXT,
    status       TEXT    NOT NULL DEFAULT 'active'
                 CHECK (status IN ('active', 'onhold', 'done')),
    start_date   TEXT,
    due_date     TEXT,
    sort_order   REAL    NOT NULL DEFAULT 0,
    archived_at  TEXT,
    created_at   TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at   TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);
CREATE INDEX idx_project_business ON project(business_id);

-- 3. 태스크 (Task)
CREATE TABLE task (
    id              INTEGER PRIMARY KEY,
    business_id     INTEGER NOT NULL REFERENCES business(id) ON DELETE CASCADE,
    project_id      INTEGER REFERENCES project(id) ON DELETE CASCADE,
    parent_task_id  INTEGER REFERENCES task(id) ON DELETE CASCADE,
    title           TEXT    NOT NULL,
    description     TEXT,
    status          TEXT    NOT NULL DEFAULT 'todo'
                    CHECK (status IN ('todo', 'doing', 'review', 'done')),
    priority        INTEGER NOT NULL DEFAULT 2 CHECK (priority BETWEEN 0 AND 4),
    due_date        TEXT,
    sort_order      REAL    NOT NULL DEFAULT 0,
    completed_at    TEXT,
    archived_at     TEXT,
    created_at      TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at      TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);
CREATE INDEX idx_task_business ON task(business_id);
CREATE INDEX idx_task_project  ON task(project_id);
CREATE INDEX idx_task_parent   ON task(parent_task_id);
CREATE INDEX idx_task_status   ON task(status);

-- 4. 라벨
CREATE TABLE label (
    id     INTEGER PRIMARY KEY,
    name   TEXT NOT NULL UNIQUE,
    color  TEXT
);
CREATE TABLE task_label (
    task_id   INTEGER NOT NULL REFERENCES task(id)  ON DELETE CASCADE,
    label_id  INTEGER NOT NULL REFERENCES label(id) ON DELETE CASCADE,
    PRIMARY KEY (task_id, label_id)
);

-- 5. 문서 + 블록
CREATE TABLE document (
    id           INTEGER PRIMARY KEY,
    business_id  INTEGER NOT NULL REFERENCES business(id) ON DELETE CASCADE,
    project_id   INTEGER REFERENCES project(id) ON DELETE CASCADE,
    title        TEXT    NOT NULL DEFAULT '제목 없음',
    icon         TEXT,
    sort_order   REAL    NOT NULL DEFAULT 0,
    archived_at  TEXT,
    created_at   TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at   TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);
CREATE INDEX idx_document_business ON document(business_id);
CREATE INDEX idx_document_project  ON document(project_id);

CREATE TABLE block (
    id              INTEGER PRIMARY KEY,
    document_id     INTEGER NOT NULL REFERENCES document(id) ON DELETE CASCADE,
    parent_block_id INTEGER REFERENCES block(id) ON DELETE CASCADE,
    type            TEXT    NOT NULL,
    content         TEXT    NOT NULL DEFAULT '{}',
    sort_order      REAL    NOT NULL DEFAULT 0,
    created_at      TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at      TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);
CREATE INDEX idx_block_document ON block(document_id);

-- 6. 산출물 + 버전
CREATE TABLE deliverable (
    id              INTEGER PRIMARY KEY,
    business_id     INTEGER NOT NULL REFERENCES business(id) ON DELETE CASCADE,
    project_id      INTEGER REFERENCES project(id) ON DELETE CASCADE,
    title           TEXT    NOT NULL,
    kind            TEXT    NOT NULL DEFAULT 'file' CHECK (kind IN ('file', 'document')),
    document_id     INTEGER REFERENCES document(id) ON DELETE SET NULL,
    file_path       TEXT,
    status          TEXT    NOT NULL DEFAULT 'draft'
                    CHECK (status IN ('draft', 'review', 'done')),
    current_version INTEGER NOT NULL DEFAULT 1,
    sort_order      REAL    NOT NULL DEFAULT 0,
    archived_at     TEXT,
    created_at      TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at      TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);
CREATE INDEX idx_deliverable_business ON deliverable(business_id);
CREATE INDEX idx_deliverable_project  ON deliverable(project_id);

CREATE TABLE deliverable_version (
    id              INTEGER PRIMARY KEY,
    deliverable_id  INTEGER NOT NULL REFERENCES deliverable(id) ON DELETE CASCADE,
    version         INTEGER NOT NULL,
    file_path       TEXT,
    note            TEXT,
    created_at      TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    UNIQUE (deliverable_id, version)
);

-- 7. 서버 연결 (SSH 프로파일) — 시크릿은 secret_ref(키체인 참조)만
CREATE TABLE server_connection (
    id           INTEGER PRIMARY KEY,
    business_id  INTEGER NOT NULL REFERENCES business(id) ON DELETE CASCADE,
    project_id   INTEGER REFERENCES project(id) ON DELETE CASCADE,
    name         TEXT    NOT NULL,
    host         TEXT    NOT NULL,
    port         INTEGER NOT NULL DEFAULT 22,
    username     TEXT    NOT NULL,
    auth_type    TEXT    NOT NULL DEFAULT 'key'
                 CHECK (auth_type IN ('key', 'password', 'agent')),
    key_path     TEXT,
    secret_ref   TEXT,
    last_used_at TEXT,
    archived_at  TEXT,
    created_at   TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at   TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);
CREATE INDEX idx_server_business ON server_connection(business_id);
CREATE INDEX idx_server_project  ON server_connection(project_id);

CREATE TABLE command_snippet (
    id                   INTEGER PRIMARY KEY,
    server_connection_id INTEGER REFERENCES server_connection(id) ON DELETE CASCADE,
    name                 TEXT    NOT NULL,
    command              TEXT    NOT NULL,
    sort_order           REAL    NOT NULL DEFAULT 0,
    created_at           TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);
