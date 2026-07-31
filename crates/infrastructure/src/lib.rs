//! Infrastructure adapters.
//!
//! F1: SQLite/settings · F2: catalog · F3: assets/jobs generate stub.

pub mod bootstrap;
pub mod error;
pub mod paths;
pub mod sqlite;

pub use bootstrap::{bootstrap, Platform};
pub use paths::AppLayout;
pub use sqlite::SqliteSettingsStore;
pub use visual_library_application::health_message;

/// Confirms infrastructure crate links application + domain.
pub fn infrastructure_health() -> String {
    format!("infrastructure ok | {}", health_message())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sqlite::{
        applied_versions, migrate, open_database, pragma_foreign_keys, pragma_journal_mode,
    };
    use visual_library_application::ports::settings::SettingsStore;
    use visual_library_application::{get_settings, update_media_root};

    #[test]
    fn infrastructure_health_composes() {
        let msg = infrastructure_health();
        assert!(msg.contains("infrastructure ok"));
        assert!(msg.contains("Visual Library"));
    }

    #[test]
    fn migrate_empty_and_idempotent() {
        let dir = tempfile::tempdir().unwrap();
        let db = dir.path().join("t.sqlite");
        let conn = open_database(&db).unwrap();
        assert!(pragma_foreign_keys(&conn).unwrap());
        assert_eq!(pragma_journal_mode(&conn).unwrap(), "wal");

        migrate(&conn).unwrap();
        migrate(&conn).unwrap();
        let versions = applied_versions(&conn).unwrap();
        assert!(versions.iter().any(|v| v == "0001_init"));
        assert!(versions.iter().any(|v| v == "0002_domain_tables"));
    }

    #[test]
    fn catalog_ensure_and_list() {
        use visual_library_application::{
            ensure_concept, ensure_representation, ensure_theme, list_concepts,
            list_representations, list_themes,
        };

        let dir = tempfile::tempdir().unwrap();
        let platform = bootstrap(dir.path().to_path_buf()).unwrap();
        let store = platform.settings.as_ref();

        let theme = ensure_theme(store, "Nature", Some("outdoors")).unwrap();
        assert_eq!(theme.name, "Nature");
        assert_eq!(list_themes(store).unwrap().len(), 1);

        let concept = ensure_concept(store, "oak-tree", "Oak Tree", None).unwrap();
        let again = ensure_concept(store, "oak-tree", "Oak Tree", None).unwrap();
        assert_eq!(concept.id, again.id);
        assert_eq!(list_concepts(store).unwrap().len(), 1);

        let rep =
            ensure_representation(store, &concept.id, "hero", "Hero", Some("landscape")).unwrap();
        assert_eq!(rep.key, "hero");
        assert_eq!(list_representations(store, &concept.id).unwrap().len(), 1);
    }

    #[test]
    fn generate_stub_then_approve_to_library() {
        use visual_library_application::{
            approve_asset, ensure_concept, ensure_representation, generate_stub_asset,
            list_library_assets, list_waiting_review, media_writer_for, reject_asset,
            GenerateStubInput,
        };

        let dir = tempfile::tempdir().unwrap();
        let platform = bootstrap(dir.path().to_path_buf()).unwrap();
        let db = platform.settings.as_ref();
        let media_root = platform.layout.media_root.clone();
        let writer = media_writer_for(&media_root);

        let concept = ensure_concept(db, "hero", "Hero", None).unwrap();
        let rep =
            ensure_representation(db, &concept.id, "front", "Front", Some("landscape")).unwrap();

        let gen = generate_stub_asset(
            db,
            db,
            db,
            &writer,
            GenerateStubInput {
                concept_id: concept.id.clone(),
                representation_id: rep.id.clone(),
                prompt: Some("stub".into()),
                idempotency_key: Some("k1".into()),
            },
        )
        .unwrap();

        assert_eq!(gen.job_status, "waiting_review");
        assert_eq!(gen.asset_status, "waiting_review");
        assert_eq!(list_waiting_review(db).unwrap().len(), 1);
        assert!(list_library_assets(db).unwrap().is_empty());
        assert!(media_root.join(&gen.storage_path).exists());

        approve_asset(db, &gen.asset_id).unwrap();
        assert!(list_waiting_review(db).unwrap().is_empty());
        assert_eq!(list_library_assets(db).unwrap().len(), 1);

        // second generate + reject
        let gen2 = generate_stub_asset(
            db,
            db,
            db,
            &writer,
            GenerateStubInput {
                concept_id: concept.id,
                representation_id: rep.id,
                prompt: None,
                idempotency_key: Some("k2".into()),
            },
        )
        .unwrap();
        reject_asset(db, &gen2.asset_id, Some("nope")).unwrap();
        assert_eq!(list_library_assets(db).unwrap().len(), 1);
    }

    #[test]
    fn plans_approve_and_automatic_factory() {
        use visual_library_application::{
            add_plan_item, approve_coverage_plan, create_plan, list_waiting_review,
            media_writer_for, run_automatic_from_plan,
        };

        let dir = tempfile::tempdir().unwrap();
        let platform = bootstrap(dir.path().to_path_buf()).unwrap();
        let db = platform.settings.as_ref();
        let writer = media_writer_for(&platform.layout.media_root);

        let plan = create_plan(db, "Growth A", Some("test"), None).unwrap();
        add_plan_item(
            db,
            &plan.id,
            "mountain",
            "wide",
            None,
            Some(10),
            Some("landscape"),
            Some("any"),
        )
        .unwrap();

        assert!(run_automatic_from_plan(db, db, db, db, &writer, &plan.id).is_err());

        approve_coverage_plan(db, &plan.id).unwrap();
        let run = run_automatic_from_plan(db, db, db, db, &writer, &plan.id).unwrap();
        assert_eq!(run.batch.generate_count, 1);
        assert_eq!(list_waiting_review(db).unwrap().len(), 1);
    }

    #[test]
    fn settings_roundtrip_via_bootstrap() {
        let dir = tempfile::tempdir().unwrap();
        let platform = bootstrap(dir.path().to_path_buf()).unwrap();
        assert!(platform.layout.db_path.exists() || platform.layout.root.join("db").exists());

        let default = platform.layout.media_root.clone();
        let s0 = get_settings(platform.settings.as_ref(), &default).unwrap();
        assert_eq!(s0.media_root, default.to_string_lossy());

        let custom = dir.path().join("custom-media");
        std::fs::create_dir_all(&custom).unwrap();
        update_media_root(
            platform.settings.as_ref(),
            custom.to_string_lossy().as_ref(),
        )
        .unwrap();

        let s1 = get_settings(platform.settings.as_ref(), &default).unwrap();
        assert_eq!(s1.media_root, custom.to_string_lossy());

        // Re-open should keep setting
        let platform2 = bootstrap(dir.path().to_path_buf()).unwrap();
        let s2 = get_settings(platform2.settings.as_ref(), &default).unwrap();
        assert_eq!(s2.media_root, custom.to_string_lossy());
    }

    #[test]
    fn settings_store_set_get() {
        let dir = tempfile::tempdir().unwrap();
        let db = dir.path().join("s.sqlite");
        let conn = open_database(&db).unwrap();
        migrate(&conn).unwrap();
        let store = SqliteSettingsStore::new(conn);
        store.set_json("k", "\"v\"").unwrap();
        assert_eq!(store.get_json("k").unwrap().as_deref(), Some("\"v\""));
    }
}
