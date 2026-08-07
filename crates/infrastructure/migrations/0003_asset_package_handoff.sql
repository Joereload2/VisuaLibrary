-- Package handoff provenance on assets (FacelessStudio write-back on Review approve).
-- D-025: never edit this file after publish; add 0004+ instead.

ALTER TABLE assets ADD COLUMN package_id TEXT;
ALTER TABLE assets ADD COLUMN package_path TEXT;
ALTER TABLE assets ADD COLUMN beat_id TEXT;
ALTER TABLE assets ADD COLUMN package_concept_key TEXT;

CREATE INDEX IF NOT EXISTS idx_assets_package_path ON assets(package_path);
CREATE INDEX IF NOT EXISTS idx_assets_package_beat ON assets(package_id, beat_id);
