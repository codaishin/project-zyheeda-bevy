use super::GetRef;
use bevy::prelude::*;

impl<TAsset> GetRef<Handle<TAsset>, TAsset> for Assets<TAsset>
where
	TAsset: Asset,
{
	fn get(&self, key: &Handle<TAsset>) -> Option<&TAsset> {
		self.get(key.id())
	}
}
