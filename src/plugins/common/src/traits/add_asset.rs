pub mod assets;

use bevy::asset::{Asset, Handle};

pub trait AddAsset<TAsset: Asset> {
	fn add_asset(&mut self, asset: TAsset) -> Handle<TAsset>;
}
