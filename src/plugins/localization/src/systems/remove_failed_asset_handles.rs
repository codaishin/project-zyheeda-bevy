use crate::traits::current_locale::CurrentLocaleMut;
use bevy::{asset::LoadState, prelude::*};
use common::traits::get_asset_load_state::GetAssetLoadState;

impl<T> RemoveFailedAssetHandles for T where T: CurrentLocaleMut + Resource {}

pub(crate) trait RemoveFailedAssetHandles: CurrentLocaleMut + Resource + Sized {
	fn remove_failed_asset_handles(ftl_server: ResMut<Self>, asset_server: Res<AssetServer>) {
		remove_failed_asset_handles(ftl_server, asset_server)
	}
}

fn remove_failed_asset_handles<TFtlServer, TAssetServer>(
	mut ftl_server: ResMut<TFtlServer>,
	asset_server: Res<TAssetServer>,
) where
	TFtlServer: CurrentLocaleMut + Resource,
	TAssetServer: GetAssetLoadState + Resource,
{
	let locale = ftl_server.current_locale_mut();

	let file_state = get_state(asset_server.as_ref(), &locale.file);
	if matches!(file_state, Some(LoadState::Failed(_))) {
		locale.file = None;
	}

	let folder_state = get_state(asset_server.as_ref(), &locale.folder);
	if matches!(folder_state, Some(LoadState::Failed(_))) {
		locale.folder = None;
	}
}

fn get_state<TAssetServer, TAsset>(
	asset_server: &TAssetServer,
	handle: &Option<Handle<TAsset>>,
) -> Option<LoadState>
where
	TAssetServer: GetAssetLoadState,
	TAsset: Asset,
{
	let handle = handle.as_ref()?;
	asset_server.get_asset_load_state(handle.id().untyped())
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::resources::ftl_server::Locale;
	use bevy::asset::{AssetLoadError, LoadState, UntypedAssetId, io::AssetReaderError};
	use macros::NestedMocks;
	use mockall::{automock, predicate::eq};
	use std::{path::PathBuf, sync::Arc};
	use testing::{NestedMocks, SingleThreadedApp, new_handle};
	use unic_langid::langid;

	#[derive(Resource, NestedMocks)]
	struct _AssetServer {
		mock: Mock_AssetServer,
	}

	#[automock]
	impl GetAssetLoadState for _AssetServer {
		fn get_asset_load_state(&self, id: UntypedAssetId) -> Option<LoadState> {
			self.mock.get_asset_load_state(id)
		}
	}

	#[derive(Resource)]
	struct _FtlServer(Locale);

	impl CurrentLocaleMut for _FtlServer {
		fn current_locale_mut(&mut self) -> &mut Locale {
			&mut self.0
		}
	}

	fn setup(ftl_server: _FtlServer, asset_server: _AssetServer) -> App {
		let mut app = App::new().single_threaded(Update);

		app.insert_resource(ftl_server);
		app.insert_resource(asset_server);
		app.add_systems(
			Update,
			remove_failed_asset_handles::<_FtlServer, _AssetServer>,
		);

		app
	}

	fn path_error(path: &str) -> LoadState {
		LoadState::Failed(Arc::new(AssetLoadError::AssetReaderError(
			AssetReaderError::NotFound(PathBuf::from(path)),
		)))
	}

	#[test]
	fn remove_file_handle() {
		let file = new_handle();
		let folder = new_handle();
		let mut app = setup(
			_FtlServer(Locale {
				ln: langid!("en"),
				file: Some(file.clone()),
				folder: Some(folder.clone()),
				bundle: None,
			}),
			_AssetServer::new().with_mock(|mock| {
				mock.expect_get_asset_load_state()
					.with(eq(file.id().untyped()))
					.return_const(path_error("my/path.ftl"));
				mock.expect_get_asset_load_state()
					.with(eq(folder.id().untyped()))
					.return_const(None);
			}),
		);

		app.update();

		let ftl_server = app.world().resource::<_FtlServer>();
		assert_eq!(
			(&None, &Some(folder)),
			(&ftl_server.0.file, &ftl_server.0.folder)
		);
	}

	#[test]
	fn remove_folder_handle() {
		let file = new_handle();
		let folder = new_handle();
		let mut app = setup(
			_FtlServer(Locale {
				ln: langid!("en"),
				file: Some(file.clone()),
				folder: Some(folder.clone()),
				bundle: None,
			}),
			_AssetServer::new().with_mock(|mock| {
				mock.expect_get_asset_load_state()
					.with(eq(file.id().untyped()))
					.return_const(None);
				mock.expect_get_asset_load_state()
					.with(eq(folder.id().untyped()))
					.return_const(path_error("my/path.ftl"));
			}),
		);

		app.update();

		let ftl_server = app.world().resource::<_FtlServer>();
		assert_eq!(
			(&Some(file), &None),
			(&ftl_server.0.file, &ftl_server.0.folder)
		);
	}
}
