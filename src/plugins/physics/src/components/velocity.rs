use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use common::{
	tools::speed::Speed,
	traits::{
		handles_physics::LinearMotion,
		register_derived_component::{DerivableFrom, InsertDerivedComponent},
	},
};
use macros::SavableComponent;
use serde::{Deserialize, Serialize};

#[derive(Component, SavableComponent, Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
#[component(immutable)]
pub struct Motion {
	target: Vec3,
	speed: Speed,
}

impl From<LinearMotion> for Motion {
	fn from(LinearMotion { target, speed }: LinearMotion) -> Self {
		Self { target, speed }
	}
}

impl<'w, 's> DerivableFrom<'w, 's, Motion> for Velocity {
	const INSERT: InsertDerivedComponent = InsertDerivedComponent::Always;
	type TParam = Query<'w, 's, &'static GlobalTransform>;

	fn derive_from(
		entity: Entity,
		Motion { target, speed }: &Motion,
		transforms: &Query<&GlobalTransform>,
	) -> Self {
		match transforms.get(entity) {
			Ok(transform) => Velocity::linear((*target - transform.translation()) * **speed),
			Err(_) => Velocity::linear(Vec3::ZERO),
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use common::{
		tools::UnitsPerSecond,
		traits::register_derived_component::RegisterDerivedComponent,
	};
	use testing::SingleThreadedApp;

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.register_derived_component::<Motion, Velocity>();

		app
	}

	#[test]
	fn insert_velocity() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((
				GlobalTransform::from_xyz(1., 2., 3.),
				Motion::from(LinearMotion {
					target: Vec3::new(3., -1., 11.),
					speed: Speed(UnitsPerSecond::from(1.)),
				}),
			))
			.id();

		app.update();

		assert_eq!(
			Some(&Velocity::linear(Vec3::new(2., -3., 8.))),
			app.world().entity(entity).get::<Velocity>(),
		);
	}

	#[test]
	fn insert_velocity_with_speed() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((
				GlobalTransform::from_xyz(1., 2., 3.),
				Motion::from(LinearMotion {
					target: Vec3::new(3., -1., 11.),
					speed: Speed(UnitsPerSecond::from(2.)),
				}),
			))
			.id();

		app.update();

		assert_eq!(
			Some(&Velocity::linear(Vec3::new(4., -6., 16.))),
			app.world().entity(entity).get::<Velocity>(),
		);
	}

	#[test]
	fn insert_velocity_zero_when_no_transform() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((Motion::from(LinearMotion {
				target: Vec3::new(3., -1., 11.),
				speed: Speed(UnitsPerSecond::from(2.)),
			}),))
			.id();

		app.update();

		assert_eq!(
			Some(&Velocity::linear(Vec3::ZERO)),
			app.world().entity(entity).get::<Velocity>(),
		);
	}
}
