use crate::{
	folder_asset_loader::LoadResult,
	resources::AssetFolder,
	states::{AssetLoadState, LoadState},
};
use bevy::{
	asset::{Asset, AssetEvent, Assets, LoadedFolder},
	prelude::{EventReader, Local, NextState, Res, ResMut, State},
};
use std::fmt::Debug;

pub(crate) fn set_assets_to_loaded<TAsset>(
	current_state: Res<State<AssetLoadState<TAsset>>>,
	mut next_state: ResMut<NextState<AssetLoadState<TAsset>>>,
	mut folder_events: EventReader<AssetEvent<LoadedFolder>>,
	folder: Res<AssetFolder<TAsset>>,
	results: Res<Assets<LoadResult<TAsset>>>,
	mut folder_loaded: Local<bool>,
) where
	TAsset: Asset + Debug + Send + Sync + 'static,
{
	let folder_events = consume_events(&mut folder_events);

	if **current_state.get() == LoadState::Loaded {
		return;
	}

	if folder_events.into_iter().any(is_loaded(folder)) {
		*folder_loaded = true;
	}

	if !*folder_loaded {
		return;
	}

	if !results.is_empty() {
		return;
	}

	next_state.set(AssetLoadState::new(LoadState::Loaded));
}

fn consume_events<'a>(
	folder_events: &'a mut EventReader<AssetEvent<LoadedFolder>>,
) -> Vec<&'a AssetEvent<LoadedFolder>> {
	folder_events.read().collect()
}

fn is_loaded<TAsset: Asset>(
	folder: Res<AssetFolder<TAsset>>,
) -> impl Fn(&AssetEvent<LoadedFolder>) -> bool + '_ {
	move |event| {
		let AssetEvent::LoadedWithDependencies { id } = event else {
			return false;
		};

		id == &folder.folder.id()
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{folder_asset_loader::LoadResult, states::AssetLoadState};
	use bevy::{
		app::{App, Update},
		asset::{Asset, AssetEvent, AssetId, Assets, Handle, LoadedFolder},
		prelude::{AppExtStates, State},
		reflect::TypePath,
		state::app::StatesPlugin,
	};
	use common::test_tools::utils::SingleThreadedApp;
	use uuid::Uuid;

	#[derive(Asset, TypePath, Debug, PartialEq, Eq, Hash, Clone)]
	struct _Asset;

	fn new_handle<TAsset: Asset>() -> Handle<TAsset> {
		Handle::Weak(AssetId::Uuid {
			uuid: Uuid::new_v4(),
		})
	}

	fn setup(folder: Handle<LoadedFolder>) -> App {
		let mut app = App::new().single_threaded(Update);
		app.add_plugins(StatesPlugin);
		app.insert_state(AssetLoadState::<_Asset>::new(LoadState::Loading));
		app.insert_resource(AssetFolder::<_Asset>::new(folder));
		app.add_event::<AssetEvent<LoadedFolder>>();
		app.init_resource::<Assets<LoadResult<_Asset>>>();

		app.add_systems(Update, set_assets_to_loaded::<_Asset>);

		app
	}

	#[test]
	fn set_to_loaded_when_load_results_empty_and_asset_folder_loaded() {
		let asset_folder = new_handle();
		let mut app = setup(asset_folder.clone());

		app.world_mut()
			.send_event(AssetEvent::LoadedWithDependencies {
				id: asset_folder.id(),
			});
		app.update();
		app.update();

		assert_eq!(
			&AssetLoadState::new(LoadState::Loaded),
			app.world()
				.resource::<State<AssetLoadState<_Asset>>>()
				.get()
		);
	}

	#[test]
	fn do_not_set_to_loaded_when_asset_folder_not_loaded() {
		let asset_folder = new_handle();
		let mut app = setup(asset_folder.clone());

		app.update();
		app.update();

		assert_eq!(
			&AssetLoadState::new(LoadState::Loading),
			app.world()
				.resource::<State<AssetLoadState::<_Asset>>>()
				.get()
		);
	}

	#[test]
	fn do_not_set_to_loaded_when_unrelated_folder_loaded() {
		let asset_folder = new_handle();
		let unrelated_folder = new_handle::<LoadedFolder>();
		let mut app = setup(asset_folder.clone());

		app.world_mut()
			.send_event(AssetEvent::LoadedWithDependencies {
				id: unrelated_folder.id(),
			});
		app.update();
		app.update();

		assert_eq!(
			&AssetLoadState::new(LoadState::Loading),
			app.world()
				.resource::<State<AssetLoadState::<_Asset>>>()
				.get()
		);
	}

	#[test]
	fn do_not_set_to_loaded_when_load_results_not_empty_and_asset_folder_loaded() {
		let asset_folder = new_handle();
		let mut app = setup(asset_folder.clone());

		app.world_mut()
			.send_event(AssetEvent::LoadedWithDependencies {
				id: asset_folder.id(),
			});
		app.world_mut()
			.resource_mut::<Assets<LoadResult<_Asset>>>()
			.add(LoadResult::Ok(_Asset));
		app.update();
		app.update();

		assert_eq!(
			&AssetLoadState::new(LoadState::Loading),
			app.world()
				.resource::<State<AssetLoadState::<_Asset>>>()
				.get()
		);
	}

	#[test]
	fn set_to_loaded_when_load_results_empty_after_asset_folder_loaded() {
		let asset_folder = new_handle();
		let mut app = setup(asset_folder.clone());

		app.world_mut()
			.send_event(AssetEvent::LoadedWithDependencies {
				id: asset_folder.id(),
			});
		app.world_mut()
			.resource_mut::<Assets<LoadResult<_Asset>>>()
			.add(LoadResult::Ok(_Asset));
		app.update();
		app.update();

		app.world_mut()
			.insert_resource(Assets::<LoadResult<_Asset>>::default());
		app.update();
		app.update();

		assert_eq!(
			&AssetLoadState::new(LoadState::Loaded),
			app.world()
				.resource::<State<AssetLoadState<_Asset>>>()
				.get()
		);
	}

	#[test]
	fn do_not_set_to_loaded_multiple_times() {
		let asset_folder = new_handle();
		let mut app = setup(asset_folder.clone());

		app.world_mut()
			.send_event(AssetEvent::LoadedWithDependencies {
				id: asset_folder.id(),
			});
		app.update();
		app.update();
		app.update();

		assert!(
			match app.world().resource::<NextState<AssetLoadState<_Asset>>>() {
				NextState::Unchanged => true,
				NextState::Pending(_) => false,
			}
		);
	}
}
