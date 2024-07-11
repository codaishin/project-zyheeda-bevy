use crate::{resources::SkillFolder, skills::Skill, traits::TryMap};
use bevy::{
	asset::{Asset, AssetEvent, AssetId, Assets, Handle},
	ecs::{system::Res, world::Ref},
	prelude::{Commands, Component, Entity, EventReader, Local, Query},
};
use common::{
	errors::{Error, Level},
	traits::{
		get_handle_from_path::GetHandelFromPath,
		load_asset::Path,
		try_insert_on::TryInsertOn,
		try_remove_from::TryRemoveFrom,
	},
};

pub(crate) fn skill_path_to_handle<
	TSource: Component + TryMap<Path, Handle<Skill>, TResult>,
	TResult: Component,
	TFolder: GetHandelFromPath<Skill> + Asset,
>(
	mut commands: Commands,
	mut folder_events: EventReader<AssetEvent<TFolder>>,
	skill_folder: Res<SkillFolder<TFolder>>,
	loaded_folders: Res<Assets<TFolder>>,
	sources: Query<(Entity, Ref<TSource>)>,
	mut folder_added: Local<bool>,
) -> Vec<Result<(), Error>> {
	let folder_id = skill_folder.0.id();
	*folder_added = *folder_added || folder_events.read().any(added(folder_id));

	if !*folder_added {
		return vec![];
	}

	let Some(folder) = loaded_folders.get(folder_id) else {
		return vec![Err(no_skill_folder_error())];
	};

	let mut errors = vec![];

	for (entity, source) in &sources {
		commands.try_insert_on(entity, source.try_map(get_handle(folder, &mut errors)));
		commands.try_remove_from::<TSource>(entity);
	}

	errors
}

fn added<TFolder: Asset>(id: AssetId<TFolder>) -> impl FnMut(&AssetEvent<TFolder>) -> bool {
	move |event| event.is_added(id)
}

fn get_handle<'a, TFolder: GetHandelFromPath<Skill>>(
	folder: &'a TFolder,
	errors: &'a mut Vec<Result<(), Error>>,
) -> impl FnMut(&Path) -> Option<Handle<Skill>> + 'a {
	move |path| {
		let handle = folder.handle_from_path(path);
		if handle.is_none() {
			errors.push(Err(no_matching_handle_error(path)));
		}
		handle
	}
}

fn no_skill_folder_error() -> Error {
	Error {
		msg: "Skill folder asset not found".to_owned(),
		lvl: Level::Error,
	}
}

fn no_matching_handle_error(path: &Path) -> Error {
	Error {
		msg: format!("No skill file found at {path:?}"),
		lvl: Level::Error,
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::{
		app::{App, Update},
		asset::{AssetId, Handle},
		prelude::IntoSystem,
		reflect::TypePath,
	};
	use common::{
		systems::log::test_tools::{fake_log_error_many_recourse, FakeErrorLogManyResource},
		test_tools::utils::SingleThreadedApp,
		traits::load_asset::Path,
	};
	use mockall::{automock, predicate::eq};
	use uuid::Uuid;

	#[derive(Default, TypePath, Asset)]
	struct _Folder {
		mock: Mock_Folder,
	}

	#[automock]
	impl GetHandelFromPath<Skill> for _Folder {
		fn handle_from_path(&self, path: &Path) -> Option<Handle<Skill>> {
			self.mock.handle_from_path(path)
		}
	}

	#[derive(Component)]
	struct _Source(Vec<Path>);

	impl TryMap<Path, Handle<Skill>, _Result> for _Source {
		fn try_map(&self, map_fn: impl FnMut(&Path) -> Option<Handle<Skill>>) -> _Result {
			_Result(self.0.iter().filter_map(map_fn).collect())
		}
	}

	#[derive(Component, Debug, PartialEq)]
	struct _Result(Vec<Handle<Skill>>);

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.init_resource::<SkillFolder<_Folder>>();
		app.init_resource::<Assets<_Folder>>();
		app.add_event::<AssetEvent<_Folder>>();
		app.add_systems(
			Update,
			skill_path_to_handle::<_Source, _Result, _Folder>.pipe(fake_log_error_many_recourse),
		);

		app
	}

	fn set_folder(app: &mut App, folder: _Folder) -> AssetId<_Folder> {
		let folder = app
			.world_mut()
			.resource_mut::<Assets<_Folder>>()
			.add(folder);
		app.world_mut().insert_resource(SkillFolder(folder.clone()));

		folder.id()
	}

	#[test]
	fn get_handle_from_item_skill_path() {
		let mut app = setup();

		let mut folder = _Folder::default();
		folder
			.mock
			.expect_handle_from_path()
			.times(1)
			.with(eq(Path::from("my/skill/path")))
			.return_const(None);
		let id = set_folder(&mut app, folder);
		app.world_mut().send_event(AssetEvent::Added { id });

		app.world_mut()
			.spawn(_Source(vec![Path::from("my/skill/path")]));

		app.update();
	}

	#[test]
	fn set_skill_handle() {
		let handle = Handle::Weak(AssetId::Uuid {
			uuid: Uuid::new_v4(),
		});

		let mut app = setup();

		let mut folder = _Folder::default();
		folder
			.mock
			.expect_handle_from_path()
			.with(eq(Path::from("my/path")))
			.return_const(handle.clone());
		folder
			.mock
			.expect_handle_from_path()
			.return_const(Handle::default());
		let id = set_folder(&mut app, folder);
		app.world_mut().send_event(AssetEvent::Added { id });

		let source = app
			.world_mut()
			.spawn(_Source(vec![Path::from("my/path")]))
			.id();

		app.update();

		let source = app.world().entity(source);

		assert_eq!(Some(&_Result(vec![handle])), source.get::<_Result>());
	}

	#[test]
	fn do_not_update_when_no_added_event_present() {
		let mut app = setup();

		let mut folder = _Folder::default();
		folder
			.mock
			.expect_handle_from_path()
			.never()
			.return_const(None);
		set_folder(&mut app, folder);

		app.world_mut()
			.spawn(_Source(vec![Path::from("my/skill/path")]));

		app.update();
	}

	#[test]
	fn get_handle_from_item_skill_path_after_added_event_already_fired() {
		let mut app = setup();

		let mut folder = _Folder::default();
		folder
			.mock
			.expect_handle_from_path()
			.times(1)
			.with(eq(Path::from("my/skill/path")))
			.return_const(None);
		let id = set_folder(&mut app, folder);
		app.world_mut().send_event(AssetEvent::Added { id });

		app.update();

		app.world_mut()
			.spawn(_Source(vec![Path::from("my/skill/path")]));

		app.update();
	}

	#[test]
	fn remove_source() {
		let mut app = setup();

		let mut folder = _Folder::default();
		folder.mock.expect_handle_from_path().return_const(None);
		let id = set_folder(&mut app, folder);
		app.world_mut().send_event(AssetEvent::Added { id });

		let source = app
			.world_mut()
			.spawn(_Source(vec![Path::from("my/skill/path")]))
			.id();

		app.update();

		let source = app.world().entity(source);

		assert!(!source.contains::<_Source>());
	}

	#[test]
	fn do_not_remove_source_when_no_added_event_present() {
		let mut app = setup();

		let mut folder = _Folder::default();
		folder.mock.expect_handle_from_path().return_const(None);
		set_folder(&mut app, folder);

		let source = app
			.world_mut()
			.spawn(_Source(vec![Path::from("my/skill/path")]))
			.id();

		app.update();

		let source = app.world().entity(source);

		assert!(source.contains::<_Source>());
	}

	#[test]
	fn log_error_when_no_folder() {
		let mut app = setup();

		let folder = Handle::<_Folder>::default();
		let id = folder.id();
		app.world_mut().insert_resource(SkillFolder(folder));
		app.world_mut().send_event(AssetEvent::Added { id });

		app.world_mut()
			.spawn(_Source(vec![Path::from("my/skill/path")]));

		app.update();
		app.update();

		assert_eq!(
			Some(&FakeErrorLogManyResource(vec![no_skill_folder_error()])),
			app.world().get_resource::<FakeErrorLogManyResource>()
		);
	}

	#[test]
	fn log_error_when_skill_path_not_found() {
		let mut app = setup();

		let mut folder = _Folder::default();
		folder.mock.expect_handle_from_path().return_const(None);
		let id = set_folder(&mut app, folder);
		app.world_mut().send_event(AssetEvent::Added { id });

		app.world_mut().spawn(_Source(vec![Path::from("my/path")]));

		app.update();

		assert_eq!(
			Some(&FakeErrorLogManyResource(vec![no_matching_handle_error(
				&Path::from("my/path")
			)])),
			app.world().get_resource::<FakeErrorLogManyResource>()
		);
	}
}
