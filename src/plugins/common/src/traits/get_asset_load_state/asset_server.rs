use super::GetAssetLoadState;
use bevy::{
	asset::{LoadState, UntypedAssetId},
	prelude::*,
};

impl GetAssetLoadState for AssetServer {
	fn get_asset_load_state(&self, id: UntypedAssetId) -> Option<LoadState> {
		self.get_load_state(id)
	}
}
