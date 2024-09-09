use crate::{
	errors::{Error, Level},
	folder_asset_loader::LoadResult,
	resources::AliveAssets,
	traits::get_asset_path::GetAssetPath,
};
use bevy::{
	asset::{Asset, AssetPath, Assets},
	prelude::{Res, ResMut, Resource},
	reflect::TypePath,
};
use std::fmt::Debug;

pub fn map_load_results<
	TAsset: Asset + Clone,
	TError: Debug + Sync + Send + TypePath + 'static,
	TGetAssetPath: Resource + GetAssetPath,
>(
	mut assets: ResMut<Assets<TAsset>>,
	mut load_results: ResMut<Assets<LoadResult<TAsset, TError>>>,
	mut alive_assets: ResMut<AliveAssets<TAsset>>,
	asset_server: Res<TGetAssetPath>,
) -> Vec<Result<(), Error>> {
	if load_results.is_empty() {
		return vec![];
	}

	let mut errors = vec![];

	for (asset_id, result) in load_results.iter() {
		match result {
			LoadResult::Ok(asset) => {
				alive_assets.insert(assets.add(asset.clone()));
			}
			LoadResult::Err(err) => {
				errors.push(Err(error(err, asset_server.get_asset_path(asset_id))));
			}
		}
	}

	*load_results = Assets::<LoadResult<TAsset, TError>>::default();

	errors
}

fn error<TError: Debug>(error: &TError, path: Option<AssetPath>) -> Error {
	match path {
		Some(path) => Error {
			msg: format!("{:?}: {:?}", path, error),
			lvl: Level::Error,
		},
		None => Error {
			msg: format!("Unknown Path: {:?}", error),
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
		ecs::system::RunSystemOnce,
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
		fn get_asset_path<T: Into<UntypedAssetId>>(&self, id: T) -> Option<AssetPath> {
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
	fn add_loaded_asset() {
		let mut app = setup([(LoadResult::Ok(_Asset), None)]);

		app.world_mut()
			.run_system_once(map_load_results::<_Asset, _Error, _Server>);

		let assets = app.world().resource::<Assets<_Asset>>();
		assert_eq!(
			vec![&_Asset],
			assets.iter().map(|(_, a)| a).collect::<Vec<_>>()
		);
	}

	#[test]
	fn empty_load_results() {
		let mut app = setup([
			(LoadResult::Ok(_Asset), None),
			(LoadResult::Err(_Error), None),
		]);

		app.world_mut()
			.run_system_once(map_load_results::<_Asset, _Error, _Server>);

		let load_results = app.world().resource::<Assets<LoadResult<_Asset, _Error>>>();
		assert_eq!(
			vec![] as Vec<&LoadResult<_Asset, _Error>>,
			load_results.iter().map(|(_, a)| a).collect::<Vec<_>>()
		);
	}

	#[test]
	fn return_error() {
		let mut app = setup([(LoadResult::Err(_Error), Some(AssetPath::from("my/path")))]);

		let results = app
			.world_mut()
			.run_system_once(map_load_results::<_Asset, _Error, _Server>);

		assert_eq!(
			vec![Err(error(&_Error, Some(AssetPath::from("my/path"))))],
			results
		);
	}

	#[test]
	fn store_asset_handle_so_it_is_not_unloaded() {
		let mut app = setup([(LoadResult::Ok(_Asset), None)]);

		app.world_mut()
			.run_system_once(map_load_results::<_Asset, _Error, _Server>);

		let assets = app.world().resource::<Assets<_Asset>>();
		let (id, _) = assets.iter().next().unwrap();
		let alive_assets = app
			.world()
			.resource::<AliveAssets<_Asset>>()
			.iter()
			.map(|h| h.id())
			.collect::<Vec<_>>();
		assert_eq!(vec![id], alive_assets);
	}
}
