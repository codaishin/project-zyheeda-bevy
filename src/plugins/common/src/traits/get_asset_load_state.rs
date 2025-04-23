mod asset_server;

use bevy::asset::{LoadState, UntypedAssetId};

pub trait GetAssetLoadState {
	fn get_asset_load_state(&self, id: UntypedAssetId) -> Option<LoadState>;
}
