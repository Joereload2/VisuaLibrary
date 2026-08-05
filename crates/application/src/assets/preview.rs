use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::error::AppError;
use crate::jobs::resolve_under_media_root;
use crate::ports::assets::AssetStore;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetPreviewDto {
    pub asset_id: String,
    pub mime: String,
    pub data_url: String,
    pub storage_path: String,
    pub width: Option<i64>,
    pub height: Option<i64>,
}

/// Read asset bytes under media_root and return a browser-ready data URL.
pub fn get_asset_preview(
    assets: &impl AssetStore,
    media_root: &Path,
    asset_id: &str,
) -> Result<AssetPreviewDto, AppError> {
    let asset = assets
        .get(asset_id)?
        .ok_or_else(|| AppError::NotFound(format!("asset {asset_id}")))?;
    let full = resolve_under_media_root(media_root, &asset.storage_path)?;
    let bytes = std::fs::read(&full)
        .map_err(|e| AppError::Storage(format!("no se pudo leer {}: {e}", full.display())))?;
    let mime = asset
        .mime
        .clone()
        .unwrap_or_else(|| guess_mime(&asset.storage_path));
    let b64 = base64_encode(&bytes);
    Ok(AssetPreviewDto {
        asset_id: asset.id,
        mime: mime.clone(),
        data_url: format!("data:{mime};base64,{b64}"),
        storage_path: asset.storage_path,
        width: asset.width,
        height: asset.height,
    })
}

fn guess_mime(path: &str) -> String {
    let lower = path.to_ascii_lowercase();
    if lower.ends_with(".png") {
        "image/png".into()
    } else if lower.ends_with(".jpg") || lower.ends_with(".jpeg") {
        "image/jpeg".into()
    } else if lower.ends_with(".bmp") {
        "image/bmp".into()
    } else if lower.ends_with(".webp") {
        "image/webp".into()
    } else {
        "application/octet-stream".into()
    }
}

/// Minimal base64 (no external dep) for local previews.
fn base64_encode(data: &[u8]) -> String {
    const TABLE: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity((data.len() + 2) / 3 * 4);
    let mut i = 0;
    while i + 3 <= data.len() {
        let n = ((data[i] as u32) << 16) | ((data[i + 1] as u32) << 8) | (data[i + 2] as u32);
        out.push(TABLE[((n >> 18) & 63) as usize] as char);
        out.push(TABLE[((n >> 12) & 63) as usize] as char);
        out.push(TABLE[((n >> 6) & 63) as usize] as char);
        out.push(TABLE[(n & 63) as usize] as char);
        i += 3;
    }
    let rem = data.len() - i;
    if rem == 1 {
        let n = (data[i] as u32) << 16;
        out.push(TABLE[((n >> 18) & 63) as usize] as char);
        out.push(TABLE[((n >> 12) & 63) as usize] as char);
        out.push('=');
        out.push('=');
    } else if rem == 2 {
        let n = ((data[i] as u32) << 16) | ((data[i + 1] as u32) << 8);
        out.push(TABLE[((n >> 18) & 63) as usize] as char);
        out.push(TABLE[((n >> 12) & 63) as usize] as char);
        out.push(TABLE[((n >> 6) & 63) as usize] as char);
        out.push('=');
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base64_known_vector() {
        assert_eq!(base64_encode(b"hello"), "aGVsbG8=");
    }
}
