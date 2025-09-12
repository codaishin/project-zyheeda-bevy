use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use common::{
	tools::{Done, speed::Speed},
	traits::{
		handles_physics::LinearMotion,
		register_derived_component::{DerivableFrom, InsertDerivedComponent},
	},
};
use macros::SavableComponent;
use serde::{Deserialize, Serialize};

#[derive(Component, SavableComponent, Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
#[component(immutable)]
pub enum Motion {
	Ongoing(LinearMotion),
	Done(LinearMotion),
}

impl From<LinearMotion> for Motion {
	fn from(linear_motion: LinearMotion) -> Self {
		Self::Ongoing(linear_motion)
	}
}

impl From<&Motion> for LinearMotion {
	fn from(motion: &Motion) -> Self {
		match motion {
			Motion::Ongoing(linear_motion) => *linear_motion,
			Motion::Done(linear_motion) => *linear_motion,
		}
	}
}

impl From<&Motion> for Done {
	fn from(motion: &Motion) -> Self {
		Done::when(matches!(motion, Motion::Done(..)))
	}
}

impl<'w, 's> DerivableFrom<'w, 's, Motion> for Velocity {
	const INSERT: InsertDerivedComponent = InsertDerivedComponent::Always;

	type TParam = Query<'w, 's, &'static Transform>;

	fn derive_from(entity: Entity, motion: &Motion, transforms: &Query<&Transform>) -> Self {
		match motion {
			Motion::Ongoing(LinearMotion::Direction { speed, direction }) => {
				velocity_with_direction(*direction, *speed)
			}
			Motion::Ongoing(LinearMotion::ToTarget { speed, target }) => {
				velocity_to_target(*target, *speed, transforms.get(entity).ok())
			}
			Motion::Ongoing(LinearMotion::Stop) | Motion::Done(..) => Velocity::zero(),
		}
	}
}

fn velocity_with_direction(direction: Dir3, speed: Speed) -> Velocity {
	Velocity::linear(*direction * *speed)
}

fn velocity_to_target(target: Vec3, speed: Speed, transform: Option<&Transform>) -> Velocity {
	match transform {
		Some(transform) => {
			let direction = target - transform.translation;
			let direction = direction.try_normalize().unwrap_or_default();
			Velocity::linear(direction * *speed)
		}
		None => Velocity::zero(),
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

	mod to_target {
		use super::*;

		#[test]
		fn insert_velocity() {
			let mut app = setup();
			let entity = app
				.world_mut()
				.spawn((
					Transform::from_xyz(1., 2., 3.),
					Motion::Ongoing(LinearMotion::ToTarget {
						speed: Speed(UnitsPerSecond::from(1.)),
						target: Vec3::new(3., -1., 11.),
					}),
				))
				.id();

			app.update();

			assert_eq!(
				Some(&Velocity::linear(Vec3::new(2., -3., 8.).normalize())),
				app.world().entity(entity).get::<Velocity>(),
			);
		}

		#[test]
		fn insert_velocity_with_speed() {
			let mut app = setup();
			let entity = app
				.world_mut()
				.spawn((
					Transform::from_xyz(1., 2., 3.),
					Motion::Ongoing(LinearMotion::ToTarget {
						speed: Speed(UnitsPerSecond::from(2.)),
						target: Vec3::new(3., -1., 11.),
					}),
				))
				.id();

			app.update();

			assert_eq!(
				Some(&Velocity::linear(Vec3::new(2., -3., 8.).normalize() * 2.)),
				app.world().entity(entity).get::<Velocity>(),
			);
		}

		#[test]
		fn insert_velocity_zero_when_no_transform() {
			let mut app = setup();
			let entity = app
				.world_mut()
				.spawn((Motion::Ongoing(LinearMotion::ToTarget {
					speed: Speed(UnitsPerSecond::from(2.)),
					target: Vec3::new(3., -1., 11.),
				}),))
				.id();

			app.update();

			assert_eq!(
				Some(&Velocity::zero()),
				app.world().entity(entity).get::<Velocity>(),
			);
		}

		#[test]
		fn insert_velocity_zero_when_direction_zero() {
			let mut app = setup();
			let entity = app
				.world_mut()
				.spawn((
					Transform::from_xyz(1., 2., 3.),
					Motion::Ongoing(LinearMotion::ToTarget {
						speed: Speed(UnitsPerSecond::from(1.)),
						target: Vec3::new(1., 2., 3.),
					}),
				))
				.id();

			app.update();

			assert_eq!(
				Some(&Velocity::zero()),
				app.world().entity(entity).get::<Velocity>(),
			);
		}
	}

	mod direction {
		use super::*;

		#[test]
		fn insert_velocity() {
			let mut app = setup();
			let entity = app
				.world_mut()
				.spawn(Motion::Ongoing(LinearMotion::Direction {
					speed: Speed(UnitsPerSecond::from(1.)),
					direction: Dir3::NEG_Y,
				}))
				.id();

			app.update();

			assert_eq!(
				Some(&Velocity::linear(Vec3::new(0., -1., 0.))),
				app.world().entity(entity).get::<Velocity>(),
			);
		}

		#[test]
		fn insert_velocity_with_speed() {
			let mut app = setup();
			let entity = app
				.world_mut()
				.spawn((
					Transform::from_xyz(1., 2., 3.),
					Motion::Ongoing(LinearMotion::Direction {
						speed: Speed(UnitsPerSecond::from(2.)),
						direction: Dir3::NEG_Y,
					}),
				))
				.id();

			app.update();

			assert_eq!(
				Some(&Velocity::linear(Vec3::new(0., -2., 0.))),
				app.world().entity(entity).get::<Velocity>(),
			);
		}
	}

	mod stop {
		use super::*;

		#[test]
		fn insert_velocity_zero() {
			let mut app = setup();
			let entity = app
				.world_mut()
				.spawn(Motion::Ongoing(LinearMotion::Stop))
				.id();

			app.update();

			assert_eq!(
				Some(&Velocity::zero()),
				app.world().entity(entity).get::<Velocity>(),
			);
		}
	}

	mod done {
		use super::*;

		#[test]
		fn insert_velocity_zero() {
			let mut app = setup();
			let entity = app
				.world_mut()
				.spawn((
					Transform::from_xyz(1., 2., 3.),
					Motion::Done(LinearMotion::ToTarget {
						speed: Speed::default(),
						target: Vec3::default(),
					}),
				))
				.id();

			app.update();

			assert_eq!(
				Some(&Velocity::zero()),
				app.world().entity(entity).get::<Velocity>(),
			);
		}
	}
}
