use crate::traits::{
	cache::Cache,
	load_asset::{LoadAsset, Path},
};
use bevy::asset::{Asset, Handle};

use super::SharedAssetHandle;

impl<TCache, TAssets, TAsset> SharedAssetHandle<TCache, Path, TAsset> for TAssets
where
	TCache: Cache<Path, Handle<TAsset>>,
	TAssets: LoadAsset<TAsset>,
	TAsset: Asset,
{
	fn handle(&mut self, cache: &mut TCache, key: Path) -> Handle<TAsset> {
		cache.cached(key.clone(), || self.load_asset(key))
	}
}

#[cfg(test)]
mod test_caching_of_pre_made_assets {
	use super::*;
	use crate::traits::load_asset::Path;
	use bevy::{asset::AssetId, render::texture::Image, utils::Uuid};
	use std::cell::RefCell;

	struct _Cache {
		args: Vec<(Path, Handle<Image>)>,
		returns: Handle<Image>,
	}

	impl _Cache {
		fn returns(handle: Handle<Image>) -> Self {
			Self {
				args: vec![],
				returns: handle,
			}
		}
	}

	impl Cache<Path, Handle<Image>> for _Cache {
		fn cached(&mut self, key: Path, new: impl FnOnce() -> Handle<Image>) -> Handle<Image> {
			self.args.push((key, new()));
			self.returns.clone()
		}
	}

	struct _LoadAsset {
		args: RefCell<Vec<Path>>,
		returns: Handle<Image>,
	}

	impl _LoadAsset {
		fn returns(handle: Handle<Image>) -> Self {
			Self {
				args: RefCell::from(vec![]),
				returns: handle,
			}
		}
	}

	impl LoadAsset<Image> for _LoadAsset {
		fn load_asset(&self, path: Path) -> Handle<Image> {
			self.args.borrow_mut().push(path);
			self.returns.clone()
		}
	}

	#[test]
	fn return_cached_handle() {
		let expected = Handle::Weak(AssetId::Uuid {
			uuid: Uuid::new_v4(),
		});
		let mut cache = _Cache::returns(expected.clone());
		let mut load_asset = _LoadAsset::returns(Handle::default());

		let got = load_asset.handle(&mut cache, Path::from("a/path"));

		assert_eq!(expected, got);
	}

	#[test]
	fn call_cache_with_proper_arguments() {
		let expected = (
			Path::from("asset/path"),
			Handle::Weak(AssetId::Uuid {
				uuid: Uuid::new_v4(),
			}),
		);
		let mut cache = _Cache::returns(Handle::default());
		let mut load_asset = _LoadAsset::returns(expected.1.clone());

		load_asset.handle(&mut cache, expected.0.clone());

		assert_eq!(vec![expected], cache.args);
	}

	#[test]
	fn call_load_asset_with_proper_arguments() {
		let expected = Path::from("asset/path");
		let mut cache = _Cache::returns(Handle::default());
		let mut load_asset = _LoadAsset::returns(Handle::default());

		load_asset.handle(&mut cache, expected.clone());

		assert_eq!(vec![expected], load_asset.args.into_inner());
	}
}
