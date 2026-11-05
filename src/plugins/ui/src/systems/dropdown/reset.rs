use crate::resources::dropdown_change::DropdownChanged;
use bevy::prelude::*;

impl DropdownChanged {
	pub(crate) fn reset(mut changed: ResMut<Self>) {
		**changed = false;
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use testing::SingleThreadedApp;

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.init_resource::<DropdownChanged>();
		app.add_systems(Update, DropdownChanged::reset);

		app
	}

	#[test]
	fn reset() {
		let mut app = setup();
		app.insert_resource(DropdownChanged(true));

		app.update();

		assert!(!app.world().resource::<DropdownChanged>());
	}
}
