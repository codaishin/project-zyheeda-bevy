use bevy::prelude::*;

#[derive(Resource, Debug, PartialEq, Default, Clone, Copy)]
pub(crate) struct WindowSize {
	pub(crate) height: f32,
	pub(crate) width: f32,
}

impl WindowSize {
	pub(crate) fn update(mut window_size: ResMut<WindowSize>, windows: Query<&Window>) {
		let Ok(window) = windows.single() else {
			return;
		};

		if window_size.matches(window) {
			return;
		}

		*window_size = WindowSize {
			width: window.width(),
			height: window.height(),
		}
	}

	fn matches(&self, window: &Window) -> bool {
		self.height == window.height() && self.width == window.width()
	}
}

#[cfg(test)]
mod test {
	#![allow(clippy::unwrap_used)]
	use super::*;
	use bevy::window::WindowResolution;
	use std::sync::{Arc, Mutex};
	use testing::{SingleThreadedApp, is_changed_resource};

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);
		app.init_resource::<WindowSize>();
		app.add_systems(Update, WindowSize::update);

		app
	}

	#[test]
	fn update_window_size() {
		let mut app = setup();
		app.world_mut().spawn(Window {
			resolution: WindowResolution::new(100., 20.),
			..default()
		});

		app.update();

		assert_eq!(
			&WindowSize {
				width: 100.,
				height: 20.
			},
			app.world().resource::<WindowSize>(),
		);
	}

	#[test]
	fn do_not_update_window_size_when_there_was_no_change() {
		let changed = Arc::new(Mutex::new(false));
		let mut app = setup();
		app.add_systems(Last, is_changed_resource!(WindowSize, &changed));
		app.world_mut().spawn(Window {
			resolution: WindowResolution::new(100., 20.),
			..default()
		});

		app.update();
		app.update();

		let changed = *changed.lock().unwrap();
		assert!(!changed)
	}
}
