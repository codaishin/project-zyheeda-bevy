use crate::components::dropdown::Dropdown;
use bevy::prelude::*;
use common::tools::Focus;

pub(crate) fn dropdown_detect_focus_change<TItem: Sync + Send + 'static>(
	dropdowns: Query<(Entity, &Dropdown<TItem>, &Interaction)>,
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
	use testing::{SingleThreadedApp, set_input};

	#[derive(Resource, Debug, PartialEq)]
	struct _Result(Focus);

	const MOUSE_LEFT: MouseButton = MouseButton::Left;

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);
		app.init_resource::<ButtonInput<MouseButton>>();
		app.add_systems(
			Update,
			dropdown_detect_focus_change::<()>.pipe(
				|entities: In<Focus>, mut commands: Commands| {
					commands.insert_resource(_Result(entities.0));
				},
			),
		);

		app
	}

	#[test]
	fn return_pressed() {
		let mut app = setup();
		let pressed = app
			.world_mut()
			.spawn((Dropdown::<()>::default(), Interaction::Pressed))
			.id();
		set_input!(app, just_pressed(MOUSE_LEFT));

		app.update();

		assert_eq!(
			&_Result(vec![pressed].into()),
			app.world().resource::<_Result>(),
		);
	}

	#[test]
	fn return_unchanged_if_mouse_left_not_just_pressed() {
		let mut app = setup();
		app.world_mut()
			.spawn((Dropdown::<()>::default(), Interaction::Pressed));
		set_input!(app, pressed(MOUSE_LEFT));

		app.update();

		assert_eq!(
			&_Result(Focus::Unchanged),
			app.world().resource::<_Result>()
		);
	}

	#[test]
	fn return_empty_if_not_interaction_pressed() {
		let mut app = setup();
		app.world_mut()
			.spawn((Dropdown::<()>::default(), Interaction::None));
		set_input!(app, just_pressed(MOUSE_LEFT));

		app.update();

		assert_eq!(&_Result(vec![].into()), app.world().resource::<_Result>());
	}
}
