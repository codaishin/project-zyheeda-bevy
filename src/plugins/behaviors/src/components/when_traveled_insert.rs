use bevy::prelude::*;
use common::{
	tools::Units,
	traits::{
		clamp_zero_positive::ClampZeroPositive,
		try_despawn::TryDespawn,
		try_insert_on::TryInsertOn,
	},
};
use std::marker::PhantomData;

#[derive(Debug, PartialEq, Clone, Copy)]
pub(crate) struct WhenTraveled<TTravel> {
	distance: Units,
	phantom_data: PhantomData<TTravel>,
}

impl WhenTraveled<()> {
	pub(crate) fn via<TTravel>() -> WhenTraveled<TTravel>
	where
		TTravel: Component,
	{
		WhenTraveled {
			distance: Units::new(0.),
			phantom_data: PhantomData,
		}
	}
}

impl<TTravel> WhenTraveled<TTravel>
where
	TTravel: Component,
{
	pub(crate) fn distance(mut self, distance: Units) -> Self {
		self.distance = distance;
		self
	}

	pub(crate) fn destroy(self) -> DestroyAfterDistanceTraveled<TTravel> {
		DestroyAfterDistanceTraveled {
			remaining_distance: self.distance,
			phantom_data: PhantomData,
		}
	}
}

#[derive(Component, Debug, PartialEq, Clone, Copy)]
pub(crate) struct DestroyAfterDistanceTraveled<TTravel> {
	remaining_distance: Units,
	phantom_data: PhantomData<TTravel>,
}

impl<TTravel> DestroyAfterDistanceTraveled<TTravel> {
	pub(crate) fn remaining_distance(&self) -> Units {
		self.remaining_distance
	}
}

impl<TTravel> DestroyAfterDistanceTraveled<TTravel>
where
	TTravel: Component,
{
	pub(crate) fn system(
		mut commands: Commands,
		mut transforms: Query<
			(Entity, &mut Self, &Transform, Option<&LastTranslation>),
			With<TTravel>,
		>,
	) {
		for (entity, mut travel, transform, last_translation) in &mut transforms {
			match last_translation {
				Some(last_translation) if travel.update(transform, last_translation).done() => {
					commands.try_despawn(entity);
				}
				_ => {
					commands.try_insert_on(entity, LastTranslation(transform.translation));
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

		*remaining_distance = Units::new(remaining);

		Done::when(remaining <= 0.)
	}
}

#[derive(Component, Debug, PartialEq, Clone, Copy)]
pub(crate) struct LastTranslation(Vec3);

pub(crate) struct Done(bool);

impl Done {
	fn when(done: bool) -> Self {
		Self(done)
	}

	fn done(self) -> bool {
		let Done(max_reached) = self;
		max_reached
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use common::traits::clamp_zero_positive::ClampZeroPositive;
	use testing::SingleThreadedApp;

	#[derive(Component, Debug, PartialEq, Default)]
	struct _Travel;

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);
		app.add_systems(Update, DestroyAfterDistanceTraveled::<_Travel>::system);

		app
	}

	#[test]
	fn destroy_when_exceeding_max_travel_distance() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((
				Transform::from_xyz(1., 2., 3.),
				WhenTraveled::via::<_Travel>()
					.distance(Units::new(10.))
					.destroy(),
				_Travel,
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
				WhenTraveled::via::<_Travel>()
					.distance(Units::new(5.))
					.destroy(),
				_Travel,
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
				WhenTraveled::via::<_Travel>()
					.distance(Units::new(2.))
					.destroy(),
				_Travel,
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
				WhenTraveled::via::<_Travel>()
					.distance(Units::new(10.))
					.destroy(),
				_Travel,
			))
			.id();

		app.update();
		app.world_mut()
			.entity_mut(entity)
			.insert(Transform::from_xyz(1., 12.0, 3.));
		app.update();

		assert!(app.world().get_entity(entity).is_err());
	}

	#[test]
	fn only_destroy_travel_component_present() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((
				Transform::from_xyz(1., 2., 3.),
				WhenTraveled::via::<_Travel>()
					.distance(Units::new(10.))
					.destroy(),
			))
			.id();

		app.update();
		app.world_mut()
			.entity_mut(entity)
			.insert(Transform::from_xyz(1., 12.1, 3.));
		app.update();

		assert!(app.world().get_entity(entity).is_ok());
	}
}
