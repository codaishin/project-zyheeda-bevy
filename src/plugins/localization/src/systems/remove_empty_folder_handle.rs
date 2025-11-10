use crate::traits::current_locale::CurrentLocaleMut;
use bevy::{asset::LoadedFolder, prelude::*};

impl<T> RemoveEmptyFolderHandle for T where T: CurrentLocaleMut + Resource {}

pub(crate) trait RemoveEmptyFolderHandle: CurrentLocaleMut + Resource + Sized {
	fn remove_empty_folder_handle(ftl_server: ResMut<Self>, folders: Res<Assets<LoadedFolder>>) {
		remove_empty_folder(ftl_server, folders)
	}
}

fn remove_empty_folder<TFtlServer>(
	mut ftl_server: ResMut<TFtlServer>,
	folders: Res<Assets<LoadedFolder>>,
) where
	TFtlServer: CurrentLocaleMut + Resource,
{
	let locale = ftl_server.current_locale_mut();

	let Some(LoadedFolder { handles }) = locale.folder.as_ref().and_then(|f| folders.get(f)) else {
		return;
	};

	if !handles.is_empty() {
		return;
	}

	locale.folder = None;
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{assets::ftl::Ftl, resources::ftl_server::Locale};
	use testing::{SingleThreadedApp, new_handle};
	use unic_langid::langid;

	#[derive(Resource)]
	struct _FtlServer(Locale);

	impl CurrentLocaleMut for _FtlServer {
		fn current_locale_mut(&mut self) -> &mut Locale {
			&mut self.0
		}
	}

	fn setup<const N: usize>(
		folders: [(&Handle<LoadedFolder>, LoadedFolder); N],
		ftl_server: _FtlServer,
	) -> App {
		let mut app = App::new().single_threaded(Update);
		let mut folder_assets = Assets::default();

		for (id, asset) in folders {
			folder_assets.insert(id, asset);
		}

		app.insert_resource(folder_assets);
		app.insert_resource(ftl_server);
		app.add_systems(Update, remove_empty_folder::<_FtlServer>);

		app
	}

	#[test]
	fn remove_folder_handle() {
		let handle = new_handle();
		let folder = LoadedFolder { handles: vec![] };
		let mut app = setup(
			[(&handle, folder)],
			_FtlServer(Locale {
				ln: langid!("en"),
				file: None,
				folder: Some(handle.clone()),
				bundle: None,
			}),
		);

		app.update();

		assert_eq!(None, app.world().resource::<_FtlServer>().0.folder);
	}

	#[test]
	fn do_not_remove_folder_handle_when_some_items_in_folder() {
		let handle = new_handle();
		let folder = LoadedFolder {
			handles: vec![new_handle::<Ftl>().untyped(), new_handle::<Ftl>().untyped()],
		};
		let mut app = setup(
			[(&handle, folder)],
			_FtlServer(Locale {
				ln: langid!("en"),
				file: None,
				folder: Some(handle.clone()),
				bundle: None,
			}),
		);

		app.update();

		assert_eq!(Some(handle), app.world().resource::<_FtlServer>().0.folder);
	}
}
