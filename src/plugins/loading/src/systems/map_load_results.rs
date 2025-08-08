use crate::{folder_asset_loader::LoadResult, resources::alive_assets::AliveAssets};
use bevy::{
	asset::{Asset, AssetPath, Assets},
	prelude::{Res, ResMut, Resource},
	reflect::TypePath,
};
use common::{
	errors::{Error, Level},
	traits::{get_asset_path::GetAssetPath, or_ok::OrOk},
};
use std::fmt::Debug;

pub(crate) fn map_load_results<
	TAsset: Asset + Clone,
	TError: Debug + Sync + Send + TypePath + 'static,
	TGetAssetPath: Resource + GetAssetPath,
>(
	mut assets: ResMut<Assets<TAsset>>,
	mut load_results: ResMut<Assets<LoadResult<TAsset, TError>>>,
	mut alive_assets: ResMut<AliveAssets<TAsset>>,
	asset_server: Res<TGetAssetPath>,
) -> Result<(), Vec<Error>> {
	if load_results.is_empty() {
		return Ok(());
	}

	let mut errors = vec![];

	for (asset_id, result) in load_results.iter() {
		match result {
			LoadResult::Ok(asset) => {
				alive_assets.insert(assets.add(asset.clone()));
			}
			LoadResult::Err(err) => {
				errors.push(error(err, asset_server.get_asset_path(asset_id)));
			}
		}
	}

	*load_results = Assets::<LoadResult<TAsset, TError>>::default();

	errors.or_ok(|| ())
}

fn error<TError: Debug>(error: &TError, path: Option<AssetPath>) -> Error {
	match path {
		Some(path) => Error::Single {
			msg: format!("{path:?}: {error:?}"),
			lvl: Level::Error,
		},
		None => Error::Single {
			msg: format!("Unknown Path: {error:?}"),
			lvl: Level::Error,
		},
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::folder_asset_loader::LoadResult;
	use bevy::{
		app::App,
		asset::{AssetPath, Assets, UntypedAssetId},
		ecs::system::{RunSystemError, RunSystemOnce},
		reflect::TypePath,
	};
	use std::collections::HashMap;

	#[derive(Asset, TypePath, Debug, PartialEq, Clone)]
	struct _Asset;

	#[derive(Asset, TypePath, Debug, PartialEq, Clone)]
	struct _Error;

	#[derive(Resource)]
	struct _Server(HashMap<UntypedAssetId, Option<AssetPath<'static>>>);

	impl GetAssetPath for _Server {
		fn get_asset_path<T: Into<UntypedAssetId>>(&self, id: T) -> Option<AssetPath<'_>> {
			let asset_id = id.into();
			self.0.get(&asset_id).cloned()?
		}
	}

	fn setup<const N: usize>(
		load_results: [(LoadResult<_Asset, _Error>, Option<AssetPath<'static>>); N],
	) -> App {
		let mut app = App::new();
		let mut server = _Server(HashMap::default());
		let mut results = Assets::<LoadResult<_Asset, _Error>>::default();
		for (result, path) in load_results {
			let handle = results.add(result);
			server.0.insert(handle.untyped().id(), path);
		}
		app.init_resource::<Assets<_Asset>>();
		app.init_resource::<AliveAssets<_Asset>>();
		app.insert_resource(results);
		app.insert_resource(server);

		app
	}

	#[test]
	fn add_loaded_asset() -> Result<(), RunSystemError> {
		let mut app = setup([(LoadResult::Ok(_Asset), None)]);

		_ = app
			.world_mut()
			.run_system_once(map_load_results::<_Asset, _Error, _Server>)?;

		let assets = app.world().resource::<Assets<_Asset>>();
		assert_eq!(
			vec![&_Asset],
			assets.iter().map(|(_, a)| a).collect::<Vec<_>>()
		);
		Ok(())
	}

	#[test]
	fn empty_load_results() -> Result<(), RunSystemError> {
		let mut app = setup([
			(LoadResult::Ok(_Asset), None),
			(LoadResult::Err(_Error), None),
		]);
		_ = app
			.world_mut()
			.run_system_once(map_load_results::<_Asset, _Error, _Server>)?;

		let load_results = app.world().resource::<Assets<LoadResult<_Asset, _Error>>>();
		assert_eq!(
			vec![] as Vec<&LoadResult<_Asset, _Error>>,
			load_results.iter().map(|(_, a)| a).collect::<Vec<_>>()
		);
		Ok(())
	}

	#[test]
	fn return_error() -> Result<(), RunSystemError> {
		let mut app = setup([(LoadResult::Err(_Error), Some(AssetPath::from("my/path")))]);

		let results = app
			.world_mut()
			.run_system_once(map_load_results::<_Asset, _Error, _Server>)?;

		assert_eq!(
			Err(vec![error(&_Error, Some(AssetPath::from("my/path")))]),
			results
		);
		Ok(())
	}

	#[test]
	fn store_asset_handle_so_it_is_not_unloaded() -> Result<(), RunSystemError> {
		let mut app = setup([(LoadResult::Ok(_Asset), None)]);

		_ = app
			.world_mut()
			.run_system_once(map_load_results::<_Asset, _Error, _Server>)?;

		let assets = app.world().resource::<Assets<_Asset>>();
		let (id, _) = assets.iter().next().unwrap();
		let alive_assets = app
			.world()
			.resource::<AliveAssets<_Asset>>()
			.iter()
			.map(|h| h.id())
			.collect::<Vec<_>>();
		assert_eq!(vec![id], alive_assets);
		Ok(())
	}
}
