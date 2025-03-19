use bevy::prelude::*;

pub const fn default_handle<TAsset>() -> Handle<TAsset>
where
	TAsset: Asset,
{
	const {
		Handle::Weak(AssetId::Uuid {
			uuid: AssetId::<TAsset>::DEFAULT_UUID,
		})
	}
}
