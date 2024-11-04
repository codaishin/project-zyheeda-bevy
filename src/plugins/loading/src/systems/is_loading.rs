use crate::resources::load_tracker::LoadTracker;
use bevy::prelude::*;

pub fn is_loading(load_tracker: Option<Res<LoadTracker>>) -> bool {
	load_tracker.is_some()
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::ecs::system::RunSystemOnce;

	fn setup(tracker: Option<LoadTracker>) -> App {
		let Some(tracker) = tracker else {
			return App::new();
		};

		let mut app = App::new();
		app.insert_resource(tracker);
		app
	}

	#[test]
	fn is_loading_true() {
		let mut app = setup(Some(LoadTracker::default()));

		assert!(app.world_mut().run_system_once(is_loading));
	}

	#[test]
	fn is_loading_false() {
		let mut app = setup(None);

		assert!(!app.world_mut().run_system_once(is_loading));
	}
}
