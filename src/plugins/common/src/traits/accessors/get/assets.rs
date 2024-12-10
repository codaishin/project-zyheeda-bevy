use super::GetRefOption;
use bevy::prelude::*;

impl<TAsset> GetRefOption<Handle<TAsset>, TAsset> for Assets<TAsset>
where
	TAsset: Asset,
{
	fn get(&self, key: &Handle<TAsset>) -> Option<&TAsset> {
		self.get(key.id())
	}
}
