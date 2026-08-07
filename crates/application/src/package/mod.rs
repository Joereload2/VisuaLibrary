//! Production Package handoff (FacelessStudio) — import script + write-back images.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::error::AppError;
use crate::factory::{
    build_prompt_template, select_image_provider, ManualNeed, ProposeNeedsResult,
};
use crate::factory::propose::split_script_chunks;
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageSummary {
    pub package_id: String,
    pub title: String,
    pub path: String,
    pub package_dir: String,
    pub beats: usize,
    pub script_status: String,
    pub meta_status: String,
    pub smoke: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageDetail {
    pub summary: PackageSummary,
    pub script_text: String,
    pub full_text: String,
    pub beats: Vec<PackageBeat>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageBeat {
    pub beat_id: String,
    pub role: String,
    pub spoken_text: String,
    pub visual_intent: String,
    pub concept_key: String,
    pub representation_key: String,
    pub est_duration_sec: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WritePackageImageItem {
    pub beat_id: String,
    /// Absolute path or path relative to media_root.
    pub source_path: String,
    pub asset_id: Option<String>,
    pub concept_key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WritePackageImagesResult {
    pub package_id: String,
    pub package_path: String,
    pub written: Vec<WrittenImage>,
    pub image_count: usize,
    pub notes: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WrittenImage {
    pub beat_id: String,
    pub dest_relative: String,
    pub asset_id: Option<String>,
}

/// Default packages root: Documents/FacelessStudio/packages (override FACELESS_STUDIO_PACKAGES).
pub fn default_packages_root() -> PathBuf {
    if let Ok(v) = env::var("FACELESS_STUDIO_PACKAGES") {
        let t = v.trim();
        if !t.is_empty() {
            return PathBuf::from(t);
        }
    }
    dirs_home()
        .join("Documents")
        .join("FacelessStudio")
        .join("packages")
}

fn dirs_home() -> PathBuf {
    env::var_os("USERPROFILE")
        .or_else(|| env::var_os("HOME"))
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
}

pub fn list_packages(root: Option<&Path>) -> Result<Vec<PackageSummary>, AppError> {
    let base = root.map(Path::to_path_buf).unwrap_or_else(default_packages_root);
    if !base.is_dir() {
        return Ok(vec![]);
    }
    let mut out = Vec::new();
    let mut seen = std::collections::HashSet::new();

    // Prefer studio channels tree if present
    let studio = base
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| base.clone());
    let channels = studio.join("channels");
    if channels.is_dir() {
        collect_packages_rglob(&channels, &mut out, &mut seen)?;
    }
    collect_packages_flat(&base, &mut out, &mut seen)?;

    out.sort_by(|a, b| a.title.to_lowercase().cmp(&b.title.to_lowercase()));
    Ok(out)
}

fn collect_packages_flat(
    base: &Path,
    out: &mut Vec<PackageSummary>,
    seen: &mut std::collections::HashSet<String>,
) -> Result<(), AppError> {
    let entries = fs::read_dir(base).map_err(|e| AppError::Storage(e.to_string()))?;
    for entry in entries.flatten() {
        let pkg = entry.path().join("package.yaml");
        if pkg.is_file() {
            if let Ok(s) = load_package_summary(&pkg) {
                if seen.insert(s.package_id.clone()) {
                    out.push(s);
                }
            }
        }
    }
    Ok(())
}

fn collect_packages_rglob(
    dir: &Path,
    out: &mut Vec<PackageSummary>,
    seen: &mut std::collections::HashSet<String>,
) -> Result<(), AppError> {
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return Ok(()),
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            let pkg = path.join("package.yaml");
            if pkg.is_file() {
                if let Ok(s) = load_package_summary(&pkg) {
                    if seen.insert(s.package_id.clone()) {
                        out.push(s);
                    }
                }
            }
            collect_packages_rglob(&path, out, seen)?;
        }
    }
    Ok(())
}

pub fn load_package_summary(path: &Path) -> Result<PackageSummary, AppError> {
    let data = load_package_json(path)?;
    let package_id = data
        .get("package_id")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();
    let script = data.get("script").cloned().unwrap_or(Value::Null);
    let title = script
        .get("title")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .or_else(|| data.get("meta").and_then(|m| m.get("idea_title")).and_then(|v| v.as_str()))
        .unwrap_or(&package_id)
        .to_string();
    let beats = script
        .get("beats")
        .and_then(|v| v.as_array())
        .map(|a| a.len())
        .unwrap_or(0);
    let script_status = script
        .get("status")
        .and_then(|v| v.as_str())
        .unwrap_or("pending")
        .to_string();
    let meta = data.get("meta").cloned().unwrap_or(Value::Null);
    let meta_status = meta
        .get("status")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let smoke = meta
        .get("smoke")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    Ok(PackageSummary {
        package_id,
        title,
        path: path.to_string_lossy().into_owned(),
        package_dir: path
            .parent()
            .map(|p| p.to_string_lossy().into_owned())
            .unwrap_or_default(),
        beats,
        script_status,
        meta_status,
        smoke,
    })
}

pub fn load_package_detail(path: &Path) -> Result<PackageDetail, AppError> {
    let data = load_package_json(path)?;
    let summary = load_package_summary(path)?;
    let script = data.get("script").cloned().unwrap_or(Value::Null);
    let full_text = script
        .get("full_text")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let mut beats = Vec::new();
    if let Some(arr) = script.get("beats").and_then(|v| v.as_array()) {
        for (i, beat) in arr.iter().enumerate() {
            let spoken = beat
                .get("spoken_text")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .trim()
                .to_string();
            if spoken.is_empty() {
                continue;
            }
            let beat_id = beat
                .get("beat_id")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .unwrap_or_else(|| format!("b{:02}", i + 1));
            let concept = beat
                .get("concept_key")
                .and_then(|v| v.as_str())
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string())
                .unwrap_or_else(|| slug_concept(&spoken, i));
            beats.push(PackageBeat {
                beat_id,
                role: beat
                    .get("role")
                    .and_then(|v| v.as_str())
                    .unwrap_or("block")
                    .to_string(),
                spoken_text: spoken,
                visual_intent: beat
                    .get("visual_intent")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                concept_key: concept,
                representation_key: beat
                    .get("representation_key")
                    .and_then(|v| v.as_str())
                    .unwrap_or("lesson")
                    .to_string(),
                est_duration_sec: beat
                    .get("est_duration_sec")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(8.0),
            });
        }
    }

    // Fallback: brief structure if no beats yet
    if beats.is_empty() {
        if let Some(brief) = data.get("brief") {
            if let Some(structure) = brief.get("structure").and_then(|v| v.as_array()) {
                for (i, block) in structure.iter().enumerate() {
                    let role = block
                        .get("role")
                        .and_then(|v| v.as_str())
                        .unwrap_or("block");
                    let intent = block
                        .get("intent")
                        .or_else(|| block.get("note"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    let spoken = if intent.is_empty() {
                        format!("Bloque {role} del brief")
                    } else {
                        intent.clone()
                    };
                    beats.push(PackageBeat {
                        beat_id: format!("b{:02}", i + 1),
                        role: role.to_string(),
                        spoken_text: spoken.clone(),
                        visual_intent: intent,
                        concept_key: slug_concept(&spoken, i),
                        representation_key: "lesson".into(),
                        est_duration_sec: 8.0,
                    });
                }
            }
            if beats.is_empty() {
                if let Some(hook) = brief.get("hook").and_then(|v| v.as_str()) {
                    if hook.trim().len() >= 10 {
                        beats.push(PackageBeat {
                            beat_id: "b01".into(),
                            role: "hook".into(),
                            spoken_text: hook.trim().to_string(),
                            visual_intent: "apertura".into(),
                            concept_key: slug_concept(hook, 0),
                            representation_key: "lesson".into(),
                            est_duration_sec: 8.0,
                        });
                    }
                }
            }
        }
    }

    let script_text = if !full_text.trim().is_empty() {
        full_text.clone()
    } else if !beats.is_empty() {
        beats
            .iter()
            .map(|b| b.spoken_text.as_str())
            .collect::<Vec<_>>()
            .join("\n\n")
    } else {
        // last resort: script.md sibling
        let md = path.parent().map(|p| p.join("script.md"));
        if let Some(md) = md {
            if md.is_file() {
                fs::read_to_string(&md).unwrap_or_default()
            } else {
                String::new()
            }
        } else {
            String::new()
        }
    };

    if script_text.trim().len() < 10 {
        return Err(AppError::Validation(
            "El package no tiene guion ni brief usable (mín. ~10 caracteres)".into(),
        ));
    }

    Ok(PackageDetail {
        summary,
        script_text,
        full_text,
        beats,
    })
}

/// Propose visual needs from package beats (preferred) or script chunks.
pub fn propose_needs_from_package(path: &Path, max_needs: Option<usize>) -> Result<ProposeNeedsResult, AppError> {
    let detail = load_package_detail(path)?;
    let max = max_needs.unwrap_or(12).clamp(1, 20);
    let provider = select_image_provider(None)?;
    let package_id = detail.summary.package_id.clone();
    let package_path = detail.summary.path.clone();

    let mut needs = Vec::new();
    if !detail.beats.is_empty() {
        for (i, beat) in detail.beats.iter().take(max).enumerate() {
            let concept_name = beat
                .concept_key
                .replace('-', " ")
                .replace('_', " ");
            let representation_name = if beat.representation_key.contains("diagram") {
                "Diagram"
            } else {
                "Lesson visual"
            };
            let orientation = "landscape".to_string();
            let style = "didactic".to_string();
            let intent = if beat.visual_intent.is_empty() {
                format!("Representar el beat {} ({})", beat.beat_id, beat.role)
            } else {
                beat.visual_intent.clone()
            };
            let prompt = build_prompt_template(
                &concept_name,
                representation_name,
                &beat.spoken_text,
                Some(&intent),
                &style,
                &orientation,
            );
            needs.push(ManualNeed {
                concept_key: beat.concept_key.clone(),
                concept_name: Some(concept_name),
                representation_key: beat.representation_key.clone(),
                representation_name: Some(representation_name.into()),
                prompt: Some(prompt),
                orientation: Some(orientation),
                style: Some(style),
                provider: Some(provider.id.clone()),
                script_excerpt: Some(beat.spoken_text.clone()),
                ai_instructions: Some(format!(
                    "Package {package_id} beat {} ({}). Intent: {}",
                    beat.beat_id, beat.role, intent
                )),
                pedagogical_intent: Some(intent),
                included: Some(true),
                variant_count: Some(2),
                also_generate_if_found: Some(false),
                package_id: Some(package_id.clone()),
                beat_id: Some(beat.beat_id.clone()),
                package_path: Some(package_path.clone()),
            });
            let _ = i;
        }
    } else {
        // fallback heuristic chunks
        let chunks = split_script_chunks(&detail.script_text, max);
        for (i, chunk) in chunks.into_iter().enumerate() {
            let (concept_key, concept_name) = concept_from_chunk_local(&chunk, i);
            let orientation = "landscape".to_string();
            let style = "didactic".to_string();
            let intent = format!("Tramo {} del package {package_id}", i + 1);
            let prompt = build_prompt_template(
                &concept_name,
                "Lesson visual",
                &chunk,
                Some(&intent),
                &style,
                &orientation,
            );
            needs.push(ManualNeed {
                concept_key,
                concept_name: Some(concept_name),
                representation_key: "lesson".into(),
                representation_name: Some("Lesson visual".into()),
                prompt: Some(prompt),
                orientation: Some(orientation),
                style: Some(style),
                provider: Some(provider.id.clone()),
                script_excerpt: Some(chunk),
                ai_instructions: Some(format!("Package {package_id} chunk {}", i + 1)),
                pedagogical_intent: Some(intent),
                included: Some(true),
                variant_count: Some(2),
                also_generate_if_found: Some(false),
                package_id: Some(package_id.clone()),
                beat_id: Some(format!("b{:02}", i + 1)),
                package_path: Some(package_path.clone()),
            });
        }
    }

    Ok(ProposeNeedsResult {
        needs,
        script_instructions: format!(
            "Package {} · {} · status script={}\nCopia assets approved a media/images/{{beat_id}} tras Review.",
            detail.summary.package_id,
            detail.summary.title,
            detail.summary.script_status
        ),
        method: "package_beats_v1".into(),
        notes: format!(
            "Needs desde package ({} beats visibles). Provider default: {}. Escribe al package con write_package_images.",
            detail.beats.len(),
            provider.id
        ),
    })
}

/// Copy approved/generated images into package media/images/{beat_id}.ext and update package.yaml.
pub fn write_package_images(
    package_path: &Path,
    media_root: &Path,
    items: &[WritePackageImageItem],
) -> Result<WritePackageImagesResult, AppError> {
    if items.is_empty() {
        return Err(AppError::Validation("no hay imágenes para escribir".into()));
    }
    let mut data = load_package_json(package_path)?;
    let package_id = data
        .get("package_id")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();
    let package_dir = package_path
        .parent()
        .ok_or_else(|| AppError::Validation("package path sin carpeta".into()))?;
    let images_dir = package_dir.join("media").join("images");
    fs::create_dir_all(&images_dir).map_err(|e| AppError::Storage(e.to_string()))?;

    let mut written = Vec::new();
    let mut new_assets: Vec<Value> = Vec::new();

    for item in items {
        let beat_id = sanitize_id(&item.beat_id);
        if beat_id.is_empty() {
            continue;
        }
        let source = resolve_source_path(&item.source_path, media_root)?;
        if !source.is_file() {
            return Err(AppError::NotFound(format!(
                "imagen no encontrada: {}",
                source.display()
            )));
        }
        let ext = source
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("png")
            .to_lowercase();
        let ext = if matches!(ext.as_str(), "png" | "jpg" | "jpeg" | "webp" | "bmp") {
            ext
        } else {
            "png".into()
        };
        let dest_name = format!("{beat_id}.{ext}");
        let dest = images_dir.join(&dest_name);
        fs::copy(&source, &dest).map_err(|e| AppError::Storage(e.to_string()))?;
        let rel = format!("media/images/{dest_name}");
        written.push(WrittenImage {
            beat_id: beat_id.clone(),
            dest_relative: rel.clone(),
            asset_id: item.asset_id.clone(),
        });
        new_assets.push(json!({
            "beat_id": beat_id,
            "path": rel,
            "asset_id": item.asset_id,
            "concept_key": item.concept_key,
            "source_app": "visualibrary",
        }));
    }

    // Merge with existing image_assets by beat_id (incremental approve write-back).
    let mut merged: Vec<Value> = data
        .get("image_assets")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    for new_item in new_assets {
        let beat = new_item
            .get("beat_id")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        if let Some(pos) = merged.iter().position(|x| {
            x.get("beat_id")
                .and_then(|v| v.as_str())
                .map(|b| b == beat)
                .unwrap_or(false)
        }) {
            merged[pos] = new_item;
        } else {
            merged.push(new_item);
        }
    }
    let total_assets = merged.len();
    data["image_assets"] = Value::Array(merged);

    if let Some(meta) = data.get_mut("meta").and_then(|m| m.as_object_mut()) {
        meta.insert("status".into(), json!("assets_partial"));
        meta.insert("stage".into(), json!("assets"));
        meta.insert("vl_write_at".into(), json!(now_iso()));
    } else {
        data["meta"] = json!({
            "status": "assets_partial",
            "stage": "assets",
            "vl_write_at": now_iso(),
        });
    }

    // If all beats have images, mark assets_ready
    let script_beats = data
        .get("script")
        .and_then(|s| s.get("beats"))
        .and_then(|b| b.as_array())
        .map(|a| a.len())
        .unwrap_or(0);
    if script_beats > 0 && total_assets >= script_beats {
        if let Some(meta) = data.get_mut("meta").and_then(|m| m.as_object_mut()) {
            meta.insert("status".into(), json!("assets_ready"));
        }
    }

    save_package_json(package_path, &data)?;

    // events.jsonl
    let events = package_dir.join("events.jsonl");
    let event = json!({
        "ts": now_iso(),
        "package_id": package_id,
        "station": "visualibrary",
        "action": "images_written",
        "payload": {
            "count": written.len(),
            "beats": written.iter().map(|w| w.beat_id.clone()).collect::<Vec<_>>(),
        }
    });
    use std::io::Write;
    let mut f = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&events)
        .map_err(|e| AppError::Storage(e.to_string()))?;
    writeln!(f, "{event}").map_err(|e| AppError::Storage(e.to_string()))?;

    Ok(WritePackageImagesResult {
        package_id,
        package_path: package_path.to_string_lossy().into_owned(),
        image_count: written.len(),
        notes: format!(
            "Escritas {} imagen(es) en esta pasada ({} total en package). FacelessCreator resuelve por beat_id.",
            written.len(),
            total_assets
        ),
        written,
    })
}

/// Write a single approved asset into its linked production package (if any).
/// Returns `Ok(None)` when the asset has no package handoff fields.
pub fn writeback_asset_to_package(
    asset: &crate::assets::AssetDto,
    media_root: &Path,
) -> Result<Option<WritePackageImagesResult>, AppError> {
    let package_path = asset
        .package_path
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty());
    let beat_id = asset
        .beat_id
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty());
    let (Some(pkg), Some(beat)) = (package_path, beat_id) else {
        return Ok(None);
    };
    if asset.storage_path.trim().is_empty() {
        return Err(AppError::Validation(
            "asset sin storage_path para write-back al package".into(),
        ));
    }
    let result = write_package_images(
        Path::new(pkg),
        media_root,
        &[WritePackageImageItem {
            beat_id: beat.to_string(),
            source_path: asset.storage_path.clone(),
            asset_id: Some(asset.id.clone()),
            concept_key: asset.package_concept_key.clone(),
        }],
    )?;
    Ok(Some(result))
}

fn resolve_source_path(source_path: &str, media_root: &Path) -> Result<PathBuf, AppError> {
    let raw = source_path.trim();
    if raw.is_empty() {
        return Err(AppError::Validation("source_path vacío".into()));
    }
    let p = PathBuf::from(raw);
    if p.is_file() {
        return Ok(p);
    }
    let under = media_root.join(raw.trim_start_matches(['/', '\\']));
    if under.is_file() {
        return Ok(under);
    }
    Err(AppError::NotFound(format!(
        "no se resolvió source_path={source_path:?} bajo media_root"
    )))
}

fn load_package_json(path: &Path) -> Result<Value, AppError> {
    let text = fs::read_to_string(path).map_err(|e| AppError::Storage(e.to_string()))?;
    let data: Value = serde_json::from_str(&text).map_err(|e| {
        AppError::Validation(format!(
            "package.yaml inválido (se espera JSON v0): {e}"
        ))
    })?;
    validate_package_shape(&data, "import")?;
    Ok(data)
}

/// Structural validation of FacelessStudio package schema 0.1 (import level).
/// Keeps VL aligned with YTM/FC validators without a JSON-Schema crate.
pub fn validate_package_shape(data: &Value, level: &str) -> Result<(), AppError> {
    let obj = data
        .as_object()
        .ok_or_else(|| AppError::Validation("package debe ser un objeto JSON".into()))?;
    let pid = obj
        .get("package_id")
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty());
    if pid.is_none() {
        return Err(AppError::Validation(
            "package inválido (schema 0.1): falta package_id".into(),
        ));
    }
    if let Some(script) = obj.get("script") {
        if !script.is_object() {
            return Err(AppError::Validation(
                "package inválido: script debe ser objeto".into(),
            ));
        }
        if let Some(beats) = script.get("beats") {
            let arr = beats.as_array().ok_or_else(|| {
                AppError::Validation("package inválido: script.beats debe ser array".into())
            })?;
            for (i, beat) in arr.iter().enumerate() {
                let b = beat.as_object().ok_or_else(|| {
                    AppError::Validation(format!("package inválido: beats[{i}] debe ser objeto"))
                })?;
                let bid = b
                    .get("beat_id")
                    .and_then(|v| v.as_str())
                    .map(str::trim)
                    .filter(|s| !s.is_empty());
                let spoken = b
                    .get("spoken_text")
                    .and_then(|v| v.as_str())
                    .map(str::trim)
                    .filter(|s| !s.is_empty());
                if bid.is_none() {
                    return Err(AppError::Validation(format!(
                        "package inválido: beats[{i}].beat_id requerido"
                    )));
                }
                if spoken.is_none() {
                    return Err(AppError::Validation(format!(
                        "package inválido: beats[{i}].spoken_text requerido"
                    )));
                }
            }
        }
    }
    if let Some(assets) = obj.get("image_assets") {
        let arr = assets.as_array().ok_or_else(|| {
            AppError::Validation("package inválido: image_assets debe ser array".into())
        })?;
        for (i, a) in arr.iter().enumerate() {
            let o = a.as_object().ok_or_else(|| {
                AppError::Validation(format!("package inválido: image_assets[{i}] debe ser objeto"))
            })?;
            if o
                .get("beat_id")
                .and_then(|v| v.as_str())
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .is_none()
            {
                return Err(AppError::Validation(format!(
                    "package inválido: image_assets[{i}].beat_id requerido"
                )));
            }
            if o
                .get("path")
                .and_then(|v| v.as_str())
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .is_none()
            {
                return Err(AppError::Validation(format!(
                    "package inválido: image_assets[{i}].path requerido"
                )));
            }
        }
    }
    if matches!(level, "export" | "strict") {
        let script = obj.get("script").and_then(|s| s.as_object()).ok_or_else(|| {
            AppError::Validation("package export: script requerido".into())
        })?;
        let beats = script
            .get("beats")
            .and_then(|b| b.as_array())
            .filter(|a| !a.is_empty());
        if beats.is_none() {
            return Err(AppError::Validation(
                "package export: script.beats debe tener al menos 1 beat".into(),
            ));
        }
    }
    let _ = level;
    Ok(())
}

fn save_package_json(path: &Path, data: &Value) -> Result<(), AppError> {
    let text = serde_json::to_string_pretty(data)
        .map_err(|e| AppError::Internal(format!("serialize package: {e}")))?;
    fs::write(path, text).map_err(|e| AppError::Storage(e.to_string()))
}

fn sanitize_id(raw: &str) -> String {
    raw.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect::<String>()
        .trim_matches('_')
        .to_string()
}

fn slug_concept(text: &str, index: usize) -> String {
    let cleaned: String = text
        .to_lowercase()
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c
            } else if c.is_whitespace() || c == '-' {
                '-'
            } else {
                ' '
            }
        })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join("-");
    let base = cleaned
        .trim_matches('-')
        .chars()
        .take(40)
        .collect::<String>();
    if base.is_empty() {
        format!("concept-{}", index + 1)
    } else {
        base
    }
}

fn concept_from_chunk_local(chunk: &str, index: usize) -> (String, String) {
    let name: String = chunk
        .split_whitespace()
        .take(6)
        .collect::<Vec<_>>()
        .join(" ");
    let name = if name.is_empty() {
        format!("Concepto {}", index + 1)
    } else {
        name
    };
    (slug_concept(&name, index), name)
}

fn now_iso() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    format!("unix:{secs}")
}

// Re-export split helper used by propose — make it pub in propose or duplicate.
// We need split_script_chunks to be public.

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_subdir(name: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let dir = env::temp_dir().join(format!("vl_pkg_{name}_{nanos}"));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn load_and_propose_from_package() {
        let dir = temp_subdir("load");
        let pkg_dir = dir.join("pp_t");
        fs::create_dir_all(pkg_dir.join("media/images")).unwrap();
        let path = pkg_dir.join("package.yaml");
        let body = json!({
            "package_id": "pp_t",
            "script": {
                "status": "approved",
                "title": "Demo",
                "full_text": "Uno dos tres. Cuatro cinco seis. Siete ocho nueve.",
                "beats": [
                    {
                        "beat_id": "b01",
                        "role": "hook",
                        "spoken_text": "Hoy hablamos de un tema importante para la lección.",
                        "visual_intent": "apertura",
                        "concept_key": "open-mood",
                        "representation_key": "lesson"
                    },
                    {
                        "beat_id": "b02",
                        "role": "method",
                        "spoken_text": "Tres pasos claros para aplicar la idea en la práctica.",
                        "visual_intent": "pasos",
                        "concept_key": "three-steps",
                        "representation_key": "diagram"
                    }
                ]
            },
            "meta": {"status": "script_approved", "smoke": true}
        });
        fs::write(&path, serde_json::to_string_pretty(&body).unwrap()).unwrap();

        let detail = load_package_detail(&path).unwrap();
        assert_eq!(detail.beats.len(), 2);
        let needs = propose_needs_from_package(&path, Some(4)).unwrap();
        assert_eq!(needs.needs.len(), 2);
        assert_eq!(needs.needs[0].beat_id.as_deref(), Some("b01"));
        assert_eq!(needs.method, "package_beats_v1");
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn write_images_to_package() {
        let dir = temp_subdir("write");
        let pkg_dir = dir.join("pp_w");
        fs::create_dir_all(pkg_dir.join("media/images")).unwrap();
        let path = pkg_dir.join("package.yaml");
        fs::write(
            &path,
            serde_json::to_string_pretty(&json!({
                "package_id": "pp_w",
                "script": {
                    "status": "approved",
                    "title": "W",
                    "full_text": "Texto suficiente para validar.",
                    "beats": [{"beat_id": "b01", "spoken_text": "Hola mundo de prueba extendida."}]
                },
                "meta": {}
            }))
            .unwrap(),
        )
        .unwrap();

        let media = dir.join("media");
        fs::create_dir_all(&media).unwrap();
        let src = media.join("tile.bmp");
        let mut f = fs::File::create(&src).unwrap();
        f.write_all(b"BM").unwrap();
        f.write_all(&[0u8; 64]).unwrap();

        let result = write_package_images(
            &path,
            &media,
            &[WritePackageImageItem {
                beat_id: "b01".into(),
                source_path: src.to_string_lossy().into(),
                asset_id: None,
                concept_key: Some("open".into()),
            }],
        )
        .unwrap();
        assert_eq!(result.image_count, 1);
        assert!(pkg_dir.join("media/images/b01.bmp").is_file());
        let data: Value = serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
        assert!(data.get("image_assets").is_some());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn write_images_merges_by_beat_id() {
        let dir = temp_subdir("merge");
        let pkg_dir = dir.join("pp_m");
        fs::create_dir_all(pkg_dir.join("media/images")).unwrap();
        let path = pkg_dir.join("package.yaml");
        fs::write(
            &path,
            serde_json::to_string_pretty(&json!({
                "package_id": "pp_m",
                "script": {
                    "status": "approved",
                    "title": "M",
                    "full_text": "Texto suficiente para validar merge.",
                    "beats": [
                        {"beat_id": "b01", "spoken_text": "Primera escena con texto largo."},
                        {"beat_id": "b02", "spoken_text": "Segunda escena con texto largo."}
                    ]
                },
                "meta": {},
                "image_assets": [{
                    "beat_id": "b01",
                    "path": "media/images/b01.png",
                    "asset_id": "old"
                }]
            }))
            .unwrap(),
        )
        .unwrap();

        let media = dir.join("media");
        fs::create_dir_all(&media).unwrap();
        let src = media.join("tile2.bmp");
        let mut f = fs::File::create(&src).unwrap();
        f.write_all(b"BM").unwrap();
        f.write_all(&[0u8; 64]).unwrap();

        write_package_images(
            &path,
            &media,
            &[WritePackageImageItem {
                beat_id: "b02".into(),
                source_path: src.to_string_lossy().into(),
                asset_id: Some("a2".into()),
                concept_key: Some("two".into()),
            }],
        )
        .unwrap();

        let data: Value = serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
        let assets = data["image_assets"].as_array().unwrap();
        assert_eq!(assets.len(), 2);
        assert_eq!(data["meta"]["status"], "assets_ready");
        let _ = fs::remove_dir_all(&dir);
    }
}
