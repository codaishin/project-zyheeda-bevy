use crate::resources::track::Track;
use bevy::prelude::*;

pub fn is_processing<TProgress>(track: Option<Res<Track<TProgress>>>) -> bool
where
	TProgress: Sync + Send + 'static,
{
	track.is_some()
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::ecs::system::RunSystemOnce;

	#[derive(Default, Debug, PartialEq)]
	struct _Progress;

	fn setup(track: Option<Track<_Progress>>) -> App {
		let Some(track) = track else {
			return App::new();
		};

		let mut app = App::new();
		app.insert_resource(track);
		app
	}

	#[test]
	fn is_loading_true() {
		let mut app = setup(Some(Track::default()));

		assert!(app.world_mut().run_system_once(is_processing::<_Progress>));
	}

	#[test]
	fn is_loading_false() {
		let mut app = setup(None);

		assert!(!app.world_mut().run_system_once(is_processing::<_Progress>));
	}
}
