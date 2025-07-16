use crate::components::map_image::MapImage;
use bevy::{asset::LoadState, prelude::*};
use common::traits::{
	get_asset_load_state::GetAssetLoadState,
	handles_load_tracking::Loaded,
	thread_safe::ThreadSafe,
};

impl<TCell> MapImage<TCell>
where
	TCell: ThreadSafe,
{
	pub(crate) fn all_loaded(asset_server: Res<AssetServer>, images: Query<&Self>) -> Loaded {
		all_loaded(asset_server, images)
	}
}

fn all_loaded<TCell, TAssets>(assets: Res<TAssets>, images: Query<&MapImage<TCell>>) -> Loaded
where
	TCell: ThreadSafe,
	TAssets: Resource + GetAssetLoadState,
{
	let is_loaded = |MapImage { image, .. }: &MapImage<TCell>| {
		matches!(
			assets.get_asset_load_state(image.id().untyped()),
			Some(LoadState::Loaded),
		)
	};

	Loaded(images.iter().all(is_loaded))
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::{
		asset::{LoadState, UntypedAssetId},
		ecs::system::{RunSystemError, RunSystemOnce},
	};
	use macros::NestedMocks;
	use mockall::{automock, predicate::eq};
	use testing::{NestedMocks, SingleThreadedApp, new_handle};

	struct _Cell;

	#[derive(Resource, NestedMocks)]
	struct _Assets {
		mock: Mock_Assets,
	}

	#[automock]
	impl GetAssetLoadState for _Assets {
		fn get_asset_load_state(&self, id: UntypedAssetId) -> Option<LoadState> {
			self.mock.get_asset_load_state(id)
		}
	}

	fn setup(assets: _Assets) -> App {
		let mut app = App::new().single_threaded(Update);

		app.insert_resource(assets);

		app
	}

	#[test]
	fn all_loaded_true() -> Result<(), RunSystemError> {
		let mut app = setup(_Assets::new().with_mock(|mock| {
			mock.expect_get_asset_load_state()
				.return_const(LoadState::Loaded);
		}));
		app.world_mut()
			.spawn(MapImage::<_Cell>::from(new_handle::<Image>()));

		let loaded = app
			.world_mut()
			.run_system_once(all_loaded::<_Cell, _Assets>)?;

		assert_eq!(Loaded(true), loaded);
		Ok(())
	}

	#[test]
	fn not_all_loaded() -> Result<(), RunSystemError> {
		let mut app = setup(_Assets::new().with_mock(|mock| {
			mock.expect_get_asset_load_state()
				.return_const(LoadState::Loading);
		}));
		app.world_mut()
			.spawn(MapImage::<_Cell>::from(new_handle::<Image>()));

		let loaded = app
			.world_mut()
			.run_system_once(all_loaded::<_Cell, _Assets>)?;

		assert_eq!(Loaded(false), loaded);
		Ok(())
	}

	#[test]
	fn use_proper_arguments() -> Result<(), RunSystemError> {
		let handle = new_handle::<Image>();
		let id = handle.id().untyped();
		let mut app = setup(_Assets::new().with_mock(|mock| {
			mock.expect_get_asset_load_state()
				.times(1)
				.with(eq(id))
				.return_const(LoadState::Loaded);
		}));
		app.world_mut().spawn(MapImage::<_Cell>::from(handle));

		_ = app
			.world_mut()
			.run_system_once(all_loaded::<_Cell, _Assets>)?;

		Ok(())
	}
}
