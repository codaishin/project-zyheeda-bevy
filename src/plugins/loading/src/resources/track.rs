use bevy::{prelude::*, render::MainWorld};
use common::prelude::*;
use std::{
	any::{TypeId, type_name},
	collections::HashMap,
	marker::PhantomData,
};

#[derive(Resource, Debug, PartialEq)]
pub(crate) struct Track<TLoadGroup, TProgress> {
	items: HashMap<TypeId, LoadData>,
	_p: PhantomData<(TLoadGroup, TProgress)>,
}

impl<TLoadGroup, TProgress> Default for Track<TLoadGroup, TProgress> {
	fn default() -> Self {
		Self {
			items: HashMap::default(),
			_p: PhantomData,
		}
	}
}

#[derive(Debug, PartialEq)]
pub(crate) struct LoadData {
	type_name: &'static str,
	loaded: Loaded,
}

impl<TLoadGroup, TProgress> Track<TLoadGroup, TProgress>
where
	TProgress: ThreadSafe,
	TLoadGroup: ThreadSafe,
{
	#[cfg(test)]
	fn new<const N: usize>(items: [(TypeId, LoadData); N]) -> Self {
		Self {
			items: HashMap::from(items),
			_p: PhantomData,
		}
	}

	fn insert<T>(&mut self, loaded: Loaded)
	where
		T: 'static,
	{
		self.items.insert(
			TypeId::of::<T>(),
			LoadData {
				type_name: type_name::<T>(),
				loaded,
			},
		);
	}

	pub(crate) fn track<T, TLoaded>(In(loaded): In<TLoaded>, mut tracker: ResMut<Self>)
	where
		T: 'static,
		TLoaded: Into<Loaded>,
	{
		tracker.insert::<T>(loaded.into());
	}

	pub(crate) fn track_in_main_world<T>(In(loaded): In<Loaded>, mut main_world: ResMut<MainWorld>)
	where
		T: 'static,
	{
		let Some(mut tracker) = main_world.get_resource_mut::<Self>() else {
			return;
		};

		tracker.insert::<T>(loaded);
	}

	pub(crate) fn main_world_is_processing(main_world: Res<MainWorld>) -> bool {
		main_world
			.get_resource::<Track<TLoadGroup, TProgress>>()
			.is_some()
	}

	pub fn is_done(load_tracker: Option<Res<Self>>) -> Option<IsDone> {
		let all_done = load_tracker?
			.items
			.values()
			.map(|LoadData { loaded, .. }| *loaded)
			.all(|Loaded(loaded)| loaded);

		if !all_done {
			return None;
		}

		Some(IsDone)
	}
}

#[derive(Debug, PartialEq, Eq, Hash, Default, Clone, Copy)]
pub(crate) struct IsDone;

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::{
		ecs::system::{RunSystemError, RunSystemOnce},
		state::app::StatesPlugin,
	};
	use testing::SingleThreadedApp;

	#[derive(Default, Debug, PartialEq)]
	struct _LoadGroup;

	#[derive(Default, Debug, PartialEq)]
	struct _Progress;

	fn setup(load_tracker: Option<Track<_LoadGroup, _Progress>>) -> App {
		let mut app = App::new().single_threaded(Update);
		app.add_plugins(StatesPlugin);

		if let Some(load_tracker) = load_tracker {
			app.insert_resource(load_tracker);
		}

		app
	}

	#[test]
	fn track_load_status() -> Result<(), RunSystemError> {
		let mut app = setup(Some(Track::<_LoadGroup, _Progress>::default()));

		app.world_mut().run_system_once_with(
			Track::<_LoadGroup, _Progress>::track::<f32, Loaded>,
			Loaded(true),
		)?;
		app.world_mut().run_system_once_with(
			Track::<_LoadGroup, _Progress>::track::<u32, Loaded>,
			Loaded(false),
		)?;

		assert_eq!(
			&Track::<_LoadGroup, _Progress>::new([
				(
					TypeId::of::<f32>(),
					LoadData {
						type_name: type_name::<f32>(),
						loaded: Loaded(true)
					}
				),
				(
					TypeId::of::<u32>(),
					LoadData {
						type_name: type_name::<u32>(),
						loaded: Loaded(false)
					}
				),
			]),
			app.world().resource::<Track<_LoadGroup, _Progress>>(),
		);
		Ok(())
	}

	#[test]
	fn all_loaded() -> Result<(), RunSystemError> {
		let mut app = setup(Some(Track::new([
			(
				TypeId::of::<f32>(),
				LoadData {
					type_name: type_name::<f32>(),
					loaded: Loaded(true),
				},
			),
			(
				TypeId::of::<u32>(),
				LoadData {
					type_name: type_name::<u32>(),
					loaded: Loaded(true),
				},
			),
		])));

		let done = app
			.world_mut()
			.run_system_once(Track::<_LoadGroup, _Progress>::is_done)?;

		assert_eq!(Some(IsDone), done);
		Ok(())
	}

	#[test]
	fn not_all_loaded() -> Result<(), RunSystemError> {
		let mut app = setup(Some(Track::<_LoadGroup, _Progress>::new([
			(
				TypeId::of::<f32>(),
				LoadData {
					type_name: type_name::<f32>(),
					loaded: Loaded(true),
				},
			),
			(
				TypeId::of::<u32>(),
				LoadData {
					type_name: type_name::<u32>(),
					loaded: Loaded(false),
				},
			),
		])));

		let done = app
			.world_mut()
			.run_system_once(Track::<_LoadGroup, _Progress>::is_done)?;

		assert_eq!(None, done);
		Ok(())
	}

	#[test]
	fn no_panic_when_tracker_does_not_exist() -> Result<(), RunSystemError> {
		let mut app = setup(None);

		_ = app
			.world_mut()
			.run_system_once(Track::<_LoadGroup, _Progress>::is_done);

		Ok(())
	}
}
