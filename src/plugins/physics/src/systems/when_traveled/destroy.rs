use crate::components::when_traveled::DestroyAfterDistanceTraveled;
use bevy::prelude::*;
use common::{
	tools::{Done, Units},
	traits::accessors::get::TryApplyOn,
	zyheeda_commands::ZyheedaCommands,
};

impl DestroyAfterDistanceTraveled {
	pub(crate) fn system(
		mut commands: ZyheedaCommands,
		mut transforms: Query<(Entity, &mut Self, &Transform, Option<&LastTranslation>)>,
	) {
		for (entity, mut travel, transform, last_translation) in &mut transforms {
			match last_translation {
				Some(last_translation) if travel.update(transform, last_translation).is_done() => {
					commands.try_apply_on(&entity, |e| e.try_despawn());
				}
				_ => {
					commands.try_apply_on(&entity, |mut e| {
						e.try_insert(LastTranslation(transform.translation));
					});
				}
			}
		}
	}

	fn update(&mut self, transform: &Transform, LastTranslation(last): &LastTranslation) -> Done {
		let Self {
			remaining_distance, ..
		} = self;
		let direction = transform.translation - *last;
		let remaining = **remaining_distance - direction.length();

		*remaining_distance = Units::from(remaining);

		Done::when(remaining <= 0.)
	}
}

#[derive(Component, Debug, PartialEq, Clone, Copy)]
pub(crate) struct LastTranslation(Vec3);

#[cfg(test)]
mod tests {
	use crate::components::when_traveled::WhenTraveled;

	use super::*;
	use testing::SingleThreadedApp;

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);
		app.add_systems(Update, DestroyAfterDistanceTraveled::system);

		app
	}

	#[test]
	fn destroy_when_exceeding_max_travel_distance() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((
				Transform::from_xyz(1., 2., 3.),
				WhenTraveled::distance(Units::from(10.)).destroy(),
			))
			.id();

		app.update();
		app.world_mut()
			.entity_mut(entity)
			.insert(Transform::from_xyz(1., 12.1, 3.));
		app.update();

		assert!(app.world().get_entity(entity).is_err());
	}

	#[test]
	fn do_not_destroy_when_not_exceeding_max_travel_distance() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((
				Transform::from_xyz(1., 2., 3.),
				WhenTraveled::distance(Units::from(5.)).destroy(),
			))
			.id();

		app.update();
		app.world_mut()
			.entity_mut(entity)
			.insert(Transform::from_xyz(2., 2., 3.));
		app.update();

		assert!(app.world().get_entity(entity).is_ok());
	}

	#[test]
	fn destroy_when_exceeding_max_travel_distance_back_and_forth() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((
				Transform::from_xyz(1., 2., 3.),
				WhenTraveled::distance(Units::from(2.)).destroy(),
			))
			.id();

		app.update();
		app.world_mut()
			.entity_mut(entity)
			.insert(Transform::from_xyz(2.1, 2., 3.));
		app.update();
		app.world_mut()
			.entity_mut(entity)
			.insert(Transform::from_xyz(1., 2., 3.));
		app.update();

		assert!(app.world().get_entity(entity).is_err());
	}

	#[test]
	fn destroy_when_matching_max_travel_distance() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((
				Transform::from_xyz(1., 2., 3.),
				WhenTraveled::distance(Units::from(10.)).destroy(),
			))
			.id();

		app.update();
		app.world_mut()
			.entity_mut(entity)
			.insert(Transform::from_xyz(1., 12.0, 3.));
		app.update();

		assert!(app.world().get_entity(entity).is_err());
	}
}
