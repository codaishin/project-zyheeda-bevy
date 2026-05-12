use crate::components::light::Light;
use bevy::{camera::visibility::SetViewVisibility, prelude::*};

impl Light {
	pub(crate) fn set_visibility(entities: Query<&mut ViewVisibility, With<Self>>) {
		for mut visibility in entities {
			// FIXME: This is temporary. See https://github.com/codaishin/project-zyheeda-bevy/issues/758
			visibility.set_visible();
		}
	}
}

#[cfg(test)]
mod tests {
	#![allow(clippy::unwrap_used)]
	use super::*;
	use bevy::camera::visibility::SetViewVisibility;
	use testing::SingleThreadedApp;

	fn visible(app: &mut App) -> ViewVisibility {
		let mut entity = app.world_mut().spawn(ViewVisibility::HIDDEN);

		let mut visibility = entity.get_mut::<ViewVisibility>().unwrap();
		visibility.set_visible();
		let visibility = *visibility;

		entity.despawn();

		visibility
	}

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_systems(Update, Light::set_visibility);

		app
	}

	#[test]
	fn set_to_visible() {
		let mut app = setup();
		let entity = app.world_mut().spawn((Light, ViewVisibility::HIDDEN)).id();

		app.update();

		assert_eq!(
			Some(&visible(&mut app)),
			app.world().entity(entity).get::<ViewVisibility>(),
		);
	}

	#[test]
	fn do_nothing_if_light_missing() {
		let mut app = setup();
		let entity = app.world_mut().spawn(ViewVisibility::HIDDEN).id();

		app.update();

		assert_eq!(
			Some(&ViewVisibility::HIDDEN),
			app.world().entity(entity).get::<ViewVisibility>(),
		);
	}
}
