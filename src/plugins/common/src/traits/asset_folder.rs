use super::load_asset::Path;

pub trait AssetFolderPath {
	fn asset_folder_path() -> Path;
}
