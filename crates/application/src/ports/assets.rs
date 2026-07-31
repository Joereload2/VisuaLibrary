use crate::assets::AssetDto;
use crate::error::AppError;

pub trait AssetStore {
    fn insert(&self, asset: &AssetDto) -> Result<(), AppError>;
    fn get(&self, id: &str) -> Result<Option<AssetDto>, AppError>;
    fn list_by_status(&self, status: &str) -> Result<Vec<AssetDto>, AppError>;
    fn update_status(
        &self,
        id: &str,
        status: &str,
        approved_at: Option<&str>,
        rejected_at: Option<&str>,
        reject_reason: Option<&str>,
    ) -> Result<(), AppError>;

    /// Best approved candidate for FOUND matching (MVP: representation + orientation/style).
    fn find_approved_match(
        &self,
        representation_id: &str,
        orientation: &str,
        style: &str,
    ) -> Result<Option<AssetDto>, AppError>;

    fn update_metadata(
        &self,
        id: &str,
        review_notes: Option<&str>,
        orientation: Option<&str>,
        style: Option<&str>,
        prompt: Option<&str>,
    ) -> Result<(), AppError>;

    fn set_duplicate_of(&self, id: &str, of_asset_id: &str) -> Result<(), AppError>;
}
