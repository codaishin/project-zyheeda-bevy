use crate::components::dropdown::Dropdown;
use bevy::{
	input::ButtonInput,
	prelude::{Entity, MouseButton, Query, Res},
	ui::Interaction,
};

pub(crate) fn dropdown_newly_active(
	dropdowns: Query<(Entity, &Dropdown, &Interaction)>,
	mouse: Res<ButtonInput<MouseButton>>,
) -> Vec<Entity> {
	if !mouse.just_pressed(MouseButton::Left) {
		return vec![];
	}

	dropdowns
		.iter()
		.filter(|(.., interaction)| interaction == &&Interaction::Pressed)
		.map(|(entity, ..)| entity)
		.collect()
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
	struct _Result(Vec<Entity>);

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);
		app.init_resource::<ButtonInput<MouseButton>>();
		app.add_systems(
			Update,
			dropdown_newly_active.pipe(|entities: In<Vec<Entity>>, mut commands: Commands| {
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

		assert_eq!(&_Result(vec![pressed]), app.world.resource::<_Result>());
	}

	#[test]
	fn return_empty_if_mouse_left_not_just_pressed() {
		let mut app = setup();
		let mut mouse = app.world.resource_mut::<ButtonInput<MouseButton>>();
		mouse.press(MouseButton::Left);
		mouse.clear_just_pressed(MouseButton::Left);

		app.world.spawn((Dropdown::default(), Interaction::Pressed));

		app.update();

		assert_eq!(&_Result(vec![]), app.world.resource::<_Result>());
	}

	#[test]
	fn return_empty_if_not_interaction_pressed() {
		let mut app = setup();
		app.world
			.resource_mut::<ButtonInput<MouseButton>>()
			.press(MouseButton::Left);

		app.world.spawn((Dropdown::default(), Interaction::None));

		app.update();

		assert_eq!(&_Result(vec![]), app.world.resource::<_Result>());
	}
}
