use super::GetRef;
use bevy::prelude::*;

impl<TAsset> GetRef<Handle<TAsset>> for Assets<TAsset>
where
	TAsset: Asset,
{
	type TValue<'a>
		= &'a TAsset
	where
		Self: 'a;

	fn get_ref(&self, key: &Handle<TAsset>) -> Option<&TAsset> {
		self.get(key.id())
	}
}
