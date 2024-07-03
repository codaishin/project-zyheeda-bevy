use crate::components::dropdown::Dropdown;
use bevy::{
	input::ButtonInput,
	prelude::{Entity, MouseButton, Query, Res},
	ui::Interaction,
};
use common::tools::Focus;

pub(crate) fn dropdown_detect_focus_change(
	dropdowns: Query<(Entity, &Dropdown, &Interaction)>,
	mouse: Res<ButtonInput<MouseButton>>,
) -> Focus {
	if !mouse.just_pressed(MouseButton::Left) {
		return Focus::Unchanged;
	}

	dropdowns
		.iter()
		.filter(|(.., interaction)| interaction == &&Interaction::Pressed)
		.map(|(entity, ..)| entity)
		.collect::<Vec<_>>()
		.into()
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::{
		app::{App, Update},
		input::ButtonInput,
		prelude::{Commands, In, IntoSystem, MouseButton, Resource},
	};
	use common::test_tools::utils::SingleThreadedApp;

	#[derive(Resource, Debug, PartialEq)]
	struct _Result(Focus);

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);
		app.init_resource::<ButtonInput<MouseButton>>();
		app.add_systems(
			Update,
			dropdown_detect_focus_change.pipe(|entities: In<Focus>, mut commands: Commands| {
				commands.insert_resource(_Result(entities.0));
			}),
		);

		app
	}

	#[test]
	fn return_pressed() {
		let mut app = setup();
		app.world
			.resource_mut::<ButtonInput<MouseButton>>()
			.press(MouseButton::Left);

		let pressed = app
			.world
			.spawn((Dropdown::default(), Interaction::Pressed))
			.id();

		app.update();

		assert_eq!(
			&_Result(vec![pressed].into()),
			app.world.resource::<_Result>(),
		);
	}

	#[test]
	fn return_unchanged_if_mouse_left_not_just_pressed() {
		let mut app = setup();
		let mut mouse = app.world.resource_mut::<ButtonInput<MouseButton>>();
		mouse.press(MouseButton::Left);
		mouse.clear_just_pressed(MouseButton::Left);

		app.world.spawn((Dropdown::default(), Interaction::Pressed));

		app.update();

		assert_eq!(&_Result(Focus::Unchanged), app.world.resource::<_Result>());
	}

	#[test]
	fn return_empty_if_not_interaction_pressed() {
		let mut app = setup();
		app.world
			.resource_mut::<ButtonInput<MouseButton>>()
			.press(MouseButton::Left);

		app.world.spawn((Dropdown::default(), Interaction::None));

		app.update();

		assert_eq!(&_Result(vec![].into()), app.world.resource::<_Result>());
	}
}
