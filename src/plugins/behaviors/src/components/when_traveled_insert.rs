use bevy::prelude::*;
use common::{
	tools::Units,
	traits::{
		clamp_zero_positive::ClampZeroPositive,
		try_insert_on::TryInsertOn,
		try_remove_from::TryRemoveFrom,
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

	pub(crate) fn insert<TInsert>(self) -> InsertAfterDistanceTraveled<TInsert, TTravel> {
		InsertAfterDistanceTraveled {
			remaining_distance: self.distance,
			phantom_data: PhantomData,
		}
	}
}

#[derive(Component, Debug, PartialEq, Clone, Copy)]
pub(crate) struct InsertAfterDistanceTraveled<TInsert, TTravel> {
	remaining_distance: Units,
	phantom_data: PhantomData<(TInsert, TTravel)>,
}

impl<TInsert, TTravel> InsertAfterDistanceTraveled<TInsert, TTravel> {
	pub(crate) fn remaining_distance(&self) -> Units {
		self.remaining_distance
	}
}

impl<TComponent, TTravel> InsertAfterDistanceTraveled<TComponent, TTravel>
where
	TComponent: Component + Default,
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
					commands.try_insert_on(entity, TComponent::default());
					commands.try_remove_from::<Self>(entity);
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
	use common::{
		test_tools::utils::SingleThreadedApp,
		traits::clamp_zero_positive::ClampZeroPositive,
	};

	#[derive(Component, Debug, PartialEq, Default)]
	struct _Travel;

	#[derive(Component, Debug, PartialEq, Default)]
	struct _Component;

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);
		app.add_systems(
			Update,
			InsertAfterDistanceTraveled::<_Component, _Travel>::system,
		);

		app
	}

	#[test]
	fn insert_component_when_exceeding_max_travel_distance() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((
				Transform::from_xyz(1., 2., 3.),
				WhenTraveled::via::<_Travel>()
					.distance(Units::new(10.))
					.insert::<_Component>(),
				_Travel,
			))
			.id();

		app.update();
		app.world_mut()
			.entity_mut(entity)
			.insert(Transform::from_xyz(1., 12.1, 3.));
		app.update();

		assert_eq!(
			Some(&_Component),
			app.world().entity(entity).get::<_Component>()
		);
	}

	#[test]
	fn do_not_insert_component_when_not_exceeding_max_travel_distance() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((
				Transform::from_xyz(1., 2., 3.),
				WhenTraveled::via::<_Travel>()
					.distance(Units::new(5.))
					.insert::<_Component>(),
				_Travel,
			))
			.id();

		app.update();
		app.world_mut()
			.entity_mut(entity)
			.insert(Transform::from_xyz(2., 2., 3.));
		app.update();

		assert_eq!(None, app.world().entity(entity).get::<_Component>());
	}

	#[test]
	fn insert_component_when_exceeding_max_travel_distance_back_and_forth() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((
				Transform::from_xyz(1., 2., 3.),
				WhenTraveled::via::<_Travel>()
					.distance(Units::new(2.))
					.insert::<_Component>(),
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

		assert_eq!(
			Some(&_Component),
			app.world().entity(entity).get::<_Component>()
		);
	}

	#[test]
	fn insert_component_when_matching_max_travel_distance() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((
				Transform::from_xyz(1., 2., 3.),
				WhenTraveled::via::<_Travel>()
					.distance(Units::new(10.))
					.insert::<_Component>(),
				_Travel,
			))
			.id();

		app.update();
		app.world_mut()
			.entity_mut(entity)
			.insert(Transform::from_xyz(1., 12.0, 3.));
		app.update();

		assert_eq!(
			Some(&_Component),
			app.world().entity(entity).get::<_Component>()
		);
	}

	#[test]
	fn remove_driving_component_when_travel_distance_reached() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((
				Transform::from_xyz(1., 2., 3.),
				WhenTraveled::via::<_Travel>()
					.distance(Units::new(10.))
					.insert::<_Component>(),
				_Travel,
			))
			.id();

		app.update();
		app.world_mut()
			.entity_mut(entity)
			.insert(Transform::from_xyz(1., 12.0, 3.));
		app.update();

		assert_eq!(
			None,
			app.world()
				.entity(entity)
				.get::<InsertAfterDistanceTraveled::<_Component, _Travel>>()
		);
	}

	#[test]
	fn only_apply_when_travel_component_present() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((
				Transform::from_xyz(1., 2., 3.),
				WhenTraveled::via::<_Travel>()
					.distance(Units::new(10.))
					.insert::<_Component>(),
			))
			.id();

		app.update();
		app.world_mut()
			.entity_mut(entity)
			.insert(Transform::from_xyz(1., 12.1, 3.));
		app.update();

		assert_eq!(None, app.world().entity(entity).get::<_Component>());
	}
}
