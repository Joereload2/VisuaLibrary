-- Foundation 2: catalog + plan + jobs skeleton (no business factory yet).
-- D-025: never edit this file after publish; add 0003+ instead.

CREATE TABLE IF NOT EXISTS themes (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL UNIQUE,
    description TEXT,
    status TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS concepts (
    id TEXT PRIMARY KEY NOT NULL,
    key TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    description TEXT,
    status TEXT NOT NULL,
    min_representations INTEGER NOT NULL DEFAULT 1,
    min_approved_assets INTEGER NOT NULL DEFAULT 1,
    max_approved_assets INTEGER,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS concept_themes (
    concept_id TEXT NOT NULL REFERENCES concepts(id),
    theme_id TEXT NOT NULL REFERENCES themes(id),
    PRIMARY KEY (concept_id, theme_id)
);

CREATE TABLE IF NOT EXISTS representations (
    id TEXT PRIMARY KEY NOT NULL,
    concept_id TEXT NOT NULL REFERENCES concepts(id),
    key TEXT NOT NULL,
    name TEXT NOT NULL,
    description TEXT,
    orientation_default TEXT NOT NULL DEFAULT 'any',
    style_hints TEXT,
    status TEXT NOT NULL,
    min_approved_assets INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    UNIQUE (concept_id, key)
);

CREATE TABLE IF NOT EXISTS assets (
    id TEXT PRIMARY KEY NOT NULL,
    concept_id TEXT NOT NULL REFERENCES concepts(id),
    representation_id TEXT NOT NULL REFERENCES representations(id),
    status TEXT NOT NULL,
    storage_path TEXT NOT NULL,
    content_hash TEXT,
    width INTEGER,
    height INTEGER,
    mime TEXT,
    format TEXT,
    orientation TEXT,
    style TEXT,
    provider TEXT,
    prompt TEXT,
    generation_request_id TEXT,
    review_notes TEXT,
    reject_reason TEXT,
    duplicate_of_asset_id TEXT REFERENCES assets(id),
    batch_id TEXT,
    approved_at TEXT,
    rejected_at TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_assets_status_created ON assets(status, created_at);
CREATE INDEX IF NOT EXISTS idx_assets_status_concept ON assets(status, concept_id);
CREATE INDEX IF NOT EXISTS idx_assets_representation_status ON assets(representation_id, status);
CREATE INDEX IF NOT EXISTS idx_assets_content_hash ON assets(content_hash);

CREATE TABLE IF NOT EXISTS coverage_plans (
    id TEXT PRIMARY KEY NOT NULL,
    theme_id TEXT REFERENCES themes(id),
    name TEXT NOT NULL,
    description TEXT,
    status TEXT NOT NULL,
    approved_at TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS coverage_plan_items (
    id TEXT PRIMARY KEY NOT NULL,
    plan_id TEXT NOT NULL REFERENCES coverage_plans(id),
    concept_id TEXT REFERENCES concepts(id),
    representation_id TEXT REFERENCES representations(id),
    concept_key TEXT,
    representation_key TEXT,
    action TEXT NOT NULL,
    priority INTEGER NOT NULL DEFAULT 100,
    target_count INTEGER NOT NULL DEFAULT 1,
    constraints_json TEXT,
    status TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_plan_items_plan_status ON coverage_plan_items(plan_id, status, priority);

CREATE TABLE IF NOT EXISTS generation_requests (
    id TEXT PRIMARY KEY NOT NULL,
    source TEXT NOT NULL,
    batch_id TEXT,
    concept_id TEXT,
    representation_id TEXT,
    concept_key TEXT,
    representation_key TEXT,
    prompt TEXT,
    orientation TEXT,
    style TEXT,
    provider TEXT,
    decision TEXT,
    found_asset_id TEXT,
    coverage_plan_item_id TEXT,
    status TEXT NOT NULL,
    result_asset_id TEXT,
    error TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS jobs (
    id TEXT PRIMARY KEY NOT NULL,
    job_type TEXT NOT NULL,
    payload_json TEXT NOT NULL,
    status TEXT NOT NULL,
    priority INTEGER NOT NULL DEFAULT 100,
    attempts INTEGER NOT NULL DEFAULT 0,
    max_attempts INTEGER NOT NULL DEFAULT 3,
    scheduled_at TEXT,
    started_at TEXT,
    finished_at TEXT,
    heartbeat_at TEXT,
    last_error TEXT,
    related_entity_type TEXT,
    related_entity_id TEXT,
    idempotency_key TEXT,
    progress_json TEXT,
    outputs_json TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_jobs_status_priority ON jobs(status, priority, created_at);
CREATE UNIQUE INDEX IF NOT EXISTS idx_jobs_idempotency ON jobs(idempotency_key) WHERE idempotency_key IS NOT NULL;
