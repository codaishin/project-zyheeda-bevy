use bevy::{prelude::*, state::state::FreelyMutableState};
use std::{any::TypeId, collections::HashMap};

#[derive(Resource, Default, Debug, PartialEq)]
pub struct LoadTracker(HashMap<TypeId, Loaded>);

#[derive(Debug, PartialEq)]
pub struct Loaded(pub bool);

impl LoadTracker {
	pub(crate) fn track<T>(In(loaded): In<Loaded>, mut tracker: ResMut<LoadTracker>)
	where
		T: 'static,
	{
		tracker.0.insert(TypeId::of::<T>(), loaded);
	}

	pub fn when_all_loaded_set<TState>(
		state: TState,
	) -> impl Fn(Option<Res<Self>>, ResMut<NextState<TState>>)
	where
		TState: FreelyMutableState + Copy,
	{
		move |load_tracker: Option<Res<Self>>, mut next_state: ResMut<NextState<TState>>| {
			let Some(load_tracker) = load_tracker else {
				return;
			};

			let not_all_loaded = load_tracker.0.values().any(|Loaded(loaded)| !*loaded);

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

	fn setup(load_tracker: Option<LoadTracker>) -> App {
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
		let mut app = setup(Some(LoadTracker::default()));

		app.world_mut()
			.run_system_once_with(Loaded(true), LoadTracker::track::<f32>);
		app.world_mut()
			.run_system_once_with(Loaded(false), LoadTracker::track::<u32>);

		assert_eq!(
			&LoadTracker(HashMap::from([
				(TypeId::of::<f32>(), Loaded(true)),
				(TypeId::of::<u32>(), Loaded(false)),
			])),
			app.world().resource::<LoadTracker>(),
		);
	}

	#[test]
	fn set_state_when_all_loaded() {
		let mut app = setup(Some(LoadTracker(HashMap::from([
			(TypeId::of::<f32>(), Loaded(true)),
			(TypeId::of::<u32>(), Loaded(true)),
		]))));

		app.world_mut()
			.run_system_once(LoadTracker::when_all_loaded_set(_State));

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
		let mut app = setup(Some(LoadTracker(HashMap::from([
			(TypeId::of::<f32>(), Loaded(true)),
			(TypeId::of::<u32>(), Loaded(false)),
		]))));

		app.world_mut()
			.run_system_once(LoadTracker::when_all_loaded_set(_State));

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
			.run_system_once(LoadTracker::when_all_loaded_set(_State));
	}
}
