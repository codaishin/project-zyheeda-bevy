use super::SharedAssetHandleProcedural;
use crate::traits::{add_asset::AddAsset, cache::Cache};
use bevy::asset::{Asset, Handle};

impl<TCache, TAssets, TKey, TAsset> SharedAssetHandleProcedural<TCache, TKey, TAsset> for TAssets
where
	TCache: Cache<TKey, Handle<TAsset>>,
	TAssets: AddAsset<TAsset>,
	TAsset: Asset,
{
	fn handle(
		&mut self,
		cache: &mut TCache,
		key: TKey,
		new: impl FnOnce() -> TAsset,
	) -> Handle<TAsset> {
		cache.cached(key, || self.add_asset(new()))
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::{asset::AssetId, pbr::StandardMaterial, utils::Uuid};

	struct _Cache {
		args: Vec<(u32, Handle<StandardMaterial>)>,
		returns: Handle<StandardMaterial>,
	}

	impl _Cache {
		fn returns(mat: Handle<StandardMaterial>) -> Self {
			Self {
				args: vec![],
				returns: mat,
			}
		}
	}

	impl Cache<u32, Handle<StandardMaterial>> for _Cache {
		fn cached(
			&mut self,
			key: u32,
			new: impl FnOnce() -> Handle<StandardMaterial>,
		) -> Handle<StandardMaterial> {
			self.args.push((key, new()));
			self.returns.clone()
		}
	}

	struct _AddAsset {
		args: Vec<StandardMaterial>,
		returns: Handle<StandardMaterial>,
	}

	impl _AddAsset {
		fn returns(handle: Handle<StandardMaterial>) -> Self {
			Self {
				args: vec![],
				returns: handle,
			}
		}
	}

	impl AddAsset<StandardMaterial> for _AddAsset {
		fn add_asset(&mut self, asset: StandardMaterial) -> Handle<StandardMaterial> {
			self.args.push(asset);
			self.returns.clone()
		}
	}

	#[test]
	fn return_cached_handle() {
		let expected = Handle::Weak(AssetId::Uuid {
			uuid: Uuid::new_v4(),
		});
		let mut cache = _Cache::returns(expected.clone());
		let mut add_asset = _AddAsset::returns(Handle::default());

		let got = add_asset.handle(&mut cache, 32, StandardMaterial::default);

		assert_eq!(expected, got);
	}

	#[test]
	fn call_cache_with_proper_arguments() {
		let expected = (
			42,
			Handle::Weak(AssetId::Uuid {
				uuid: Uuid::new_v4(),
			}),
		);
		let mut cache = _Cache::returns(Handle::default());
		let mut load_asset = _AddAsset::returns(expected.1.clone());

		load_asset.handle(&mut cache, expected.0, StandardMaterial::default);

		assert_eq!(vec![expected], cache.args);
	}
}
