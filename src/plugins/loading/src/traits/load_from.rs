use common::traits::load_asset::LoadAsset;

pub trait LoadFrom<TFrom> {
	fn load_from<TLoadAsset: LoadAsset>(from: TFrom, asset_server: &mut TLoadAsset) -> Self;
}
