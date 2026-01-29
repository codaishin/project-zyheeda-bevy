use bevy::prelude::*;
use std::marker::PhantomData;

pub const fn default_handle<TAsset>() -> Handle<TAsset>
where
	TAsset: Asset,
{
	const { Handle::Uuid(AssetId::<TAsset>::DEFAULT_UUID, PhantomData) }
}
