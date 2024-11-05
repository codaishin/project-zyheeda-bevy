use bevy::{prelude::*, render::MainWorld, state::state::FreelyMutableState};
use std::{
	any::{type_name, TypeId},
	collections::HashMap,
	marker::PhantomData,
};

#[derive(Resource, Default, Debug, PartialEq)]
pub struct Track<T> {
	items: HashMap<TypeId, LoadData>,
	phantom_data: PhantomData<T>,
}

#[derive(Debug, PartialEq)]
pub(crate) struct LoadData {
	type_name: &'static str,
	loaded: Loaded,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Loaded(pub bool);

impl<TProgress> Track<TProgress>
where
	TProgress: Send + Sync + 'static,
{
	#[cfg(test)]
	fn new<const N: usize>(items: [(TypeId, LoadData); N]) -> Self {
		Self {
			items: HashMap::from(items),
			phantom_data: PhantomData,
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

	pub(crate) fn track<T>(In(loaded): In<Loaded>, mut tracker: ResMut<Track<TProgress>>)
	where
		T: 'static,
	{
		tracker.insert::<T>(loaded);
	}

	pub(crate) fn track_in_main_world<T>(In(loaded): In<Loaded>, mut main_world: ResMut<MainWorld>)
	where
		T: 'static,
	{
		let Some(mut tracker) = main_world.get_resource_mut::<Track<TProgress>>() else {
			return;
		};

		tracker.insert::<T>(loaded);
	}

	pub(crate) fn main_world_is_processing(main_world: Res<MainWorld>) -> bool {
		main_world.get_resource::<Track<TProgress>>().is_some()
	}

	pub fn when_all_done_set<TState>(
		state: TState,
	) -> impl Fn(Option<Res<Self>>, ResMut<NextState<TState>>)
	where
		TState: FreelyMutableState + Copy,
	{
		move |load_tracker: Option<Res<Self>>, mut next_state: ResMut<NextState<TState>>| {
			let Some(load_tracker) = load_tracker else {
				return;
			};

			let not_all_loaded = load_tracker
				.items
				.values()
				.map(|l| l.loaded)
				.any(|Loaded(loaded)| !loaded);

			if not_all_loaded {
				return;
			}

			next_state.set(state);
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::{ecs::system::RunSystemOnce, state::app::StatesPlugin};
	use common::test_tools::utils::SingleThreadedApp;

	#[derive(States, Default, Debug, PartialEq, Eq, Hash, Clone, Copy)]
	struct _State;

	#[derive(Default, Debug, PartialEq)]
	struct _Progress;

	fn setup(load_tracker: Option<Track<_Progress>>) -> App {
		let mut app = App::new().single_threaded(Update);
		app.add_plugins(StatesPlugin);
		app.init_state::<_State>();

		if let Some(load_tracker) = load_tracker {
			app.insert_resource(load_tracker);
		}

		app
	}

	#[test]
	fn track_load_status() {
		let mut app = setup(Some(Track::<_Progress>::default()));

		app.world_mut()
			.run_system_once_with(Loaded(true), Track::<_Progress>::track::<f32>);
		app.world_mut()
			.run_system_once_with(Loaded(false), Track::<_Progress>::track::<u32>);

		assert_eq!(
			&Track::<_Progress>::new([
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
			app.world().resource::<Track<_Progress>>(),
		);
	}

	#[test]
	fn set_state_when_all_loaded() {
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

		app.world_mut()
			.run_system_once(Track::<_Progress>::when_all_done_set(_State));

		let state = app.world().resource::<NextState<_State>>();
		assert!(
			matches!(state, NextState::Pending(_State)),
			"expected: {:?}\n     got: {:?}",
			NextState::Pending(_State),
			state,
		);
	}

	#[test]
	fn do_not_set_state_when_not_all_loaded() {
		let mut app = setup(Some(Track::<_Progress>::new([
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

		app.world_mut()
			.run_system_once(Track::<_Progress>::when_all_done_set(_State));

		let state = app.world().resource::<NextState<_State>>();
		assert!(
			matches!(state, NextState::Unchanged),
			"expected: {:?}\n     got: {:?}",
			NextState::<_State>::Unchanged,
			state,
		);
	}

	#[test]
	fn no_panic_when_tracker_does_not_exist() {
		let mut app = setup(None);

		app.world_mut()
			.run_system_once(Track::<_Progress>::when_all_done_set(_State));
	}
}
