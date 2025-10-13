use crate::system_params::input::Input;
use bevy::prelude::*;
use common::traits::handles_input::InputSetupChanged;

impl<'w, 's, TKeyMap> InputSetupChanged for Input<'w, 's, Res<'static, TKeyMap>>
where
	TKeyMap: Resource,
{
	fn input_setup_changed(&self) -> bool {
		self.key_map.is_changed()
	}
}

impl<'w, 's, TKeyMap> InputSetupChanged for Input<'w, 's, ResMut<'static, TKeyMap>>
where
	TKeyMap: Resource,
{
	fn input_setup_changed(&self) -> bool {
		self.key_map.is_changed()
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use testing::SingleThreadedApp;

	#[derive(Resource)]
	struct _Map;

	#[derive(Resource, Debug, PartialEq, Default)]
	struct _Changed(bool);

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.init_resource::<_Changed>();
		app.insert_resource(_Map);
		app.init_resource::<ButtonInput<KeyCode>>();
		app.init_resource::<ButtonInput<MouseButton>>();

		app
	}

	mod res {
		use super::*;
		use std::ops::DerefMut;

		type _Input<'w, 's> = Input<'w, 's, Res<'static, _Map>>;

		fn detect_change(input: _Input, mut changed: ResMut<_Changed>) {
			*changed = _Changed(input.input_setup_changed())
		}

		#[test]
		fn changed() {
			let mut app = setup();
			app.add_systems(Update, detect_change);
			app.update();

			app.world_mut().resource_mut::<_Map>().deref_mut();
			app.update();

			assert_eq!(&_Changed(true), app.world().resource::<_Changed>());
		}

		#[test]
		fn unchanged() {
			let mut app = setup();
			app.add_systems(Update, detect_change);
			app.update();

			app.update();

			assert_eq!(&_Changed(false), app.world().resource::<_Changed>());
		}
	}

	mod res_mut {
		use super::*;
		use std::ops::DerefMut;

		type _Input<'w, 's> = Input<'w, 's, ResMut<'static, _Map>>;

		fn detect_change(input: _Input, mut changed: ResMut<_Changed>) {
			*changed = _Changed(input.input_setup_changed())
		}

		#[test]
		fn changed() {
			let mut app = setup();
			app.add_systems(Update, detect_change);
			app.update();

			app.world_mut().resource_mut::<_Map>().deref_mut();
			app.update();

			assert_eq!(&_Changed(true), app.world().resource::<_Changed>());
		}

		#[test]
		fn unchanged() {
			let mut app = setup();
			app.add_systems(Update, detect_change);
			app.update();

			app.update();

			assert_eq!(&_Changed(false), app.world().resource::<_Changed>());
		}
	}
}
