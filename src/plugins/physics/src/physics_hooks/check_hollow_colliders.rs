use crate::components::{colliders::ColliderShape, hollow::Hollow};
use bevy::{ecs::system::SystemParam, prelude::*};
use bevy_rapier3d::prelude::*;
use common::traits::handles_physics::colliders::Shape;

#[derive(SystemParam)]
pub(crate) struct CheckHollowColliders<'w, 's> {
	colliders: Query<'w, 's, (&'static ColliderShape, &'static GlobalTransform)>,
	hollows: Query<'w, 's, (&'static Hollow, &'static GlobalTransform)>,
}

impl CheckHollowColliders<'_, '_> {
	fn may_collide(&self, a: Entity, b: Entity) -> bool {
		if self.fully_inside_hollow(a, b) || self.fully_inside_hollow(b, a) {
			return false;
		}

		true
	}

	fn fully_inside_hollow(&self, collider: Entity, hollow: Entity) -> bool {
		let (hollow_radius, hollow_offset) = match self.hollows.get(hollow) {
			Ok((Hollow { radius }, t)) => (**radius, t.translation()),
			_ => return false,
		};
		let (collider_radius, collider_offset) = match self.colliders.get(collider) {
			Ok((ColliderShape(Shape::Sphere { radius }), t)) => (**radius, t.translation()),
			_ => return false,
		};
		let offset = (collider_offset - hollow_offset).length();

		offset + collider_radius <= hollow_radius
	}
}

impl BevyPhysicsHooks for CheckHollowColliders<'_, '_> {
	fn filter_intersection_pair(&self, context: PairFilterContextView) -> bool {
		self.may_collide(context.collider1(), context.collider2())
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::ecs::system::{RunSystemError, RunSystemOnce};
	use common::tools::Units;
	use test_case::test_case;
	use testing::SingleThreadedApp;

	fn setup() -> App {
		App::new().single_threaded(Update)
	}

	#[test_case(|a, b| (a, b); "forward")]
	#[test_case(|a, b| (b, a); "reversed")]
	fn smaller_sphere_inside_hollow(
		sort: fn(Entity, Entity) -> (Entity, Entity),
	) -> Result<(), RunSystemError> {
		let mut app = setup();
		let a = app
			.world_mut()
			.spawn(ColliderShape(Shape::Sphere {
				radius: Units::from(11.),
			}))
			.id();
		let b = app
			.world_mut()
			.spawn(Hollow {
				radius: Units::from(42.),
			})
			.id();

		let may_collide = app
			.world_mut()
			.run_system_once(move |c: CheckHollowColliders| {
				let (a, b) = sort(a, b);
				c.may_collide(a, b)
			})?;

		assert!(!may_collide);
		Ok(())
	}

	#[test_case(|a, b| (a, b); "forward")]
	#[test_case(|a, b| (b, a); "reversed")]
	fn larger_sphere_inside_hollow(
		sort: fn(Entity, Entity) -> (Entity, Entity),
	) -> Result<(), RunSystemError> {
		let mut app = setup();
		let a = app
			.world_mut()
			.spawn(ColliderShape(Shape::Sphere {
				radius: Units::from(42.),
			}))
			.id();
		let b = app
			.world_mut()
			.spawn(Hollow {
				radius: Units::from(11.),
			})
			.id();

		let may_collide = app
			.world_mut()
			.run_system_once(move |c: CheckHollowColliders| {
				let (a, b) = sort(a, b);
				c.may_collide(a, b)
			})?;

		assert!(may_collide);
		Ok(())
	}

	#[test_case(|a, b| (a, b); "forward")]
	#[test_case(|a, b| (b, a); "reversed")]
	fn smaller_offset_sphere_partially_inside_hollow(
		sort: fn(Entity, Entity) -> (Entity, Entity),
	) -> Result<(), RunSystemError> {
		let mut app = setup();
		let a = app
			.world_mut()
			.spawn((
				GlobalTransform::from_xyz(8., 0., 0.),
				ColliderShape(Shape::Sphere {
					radius: Units::from(5.),
				}),
			))
			.id();
		let b = app
			.world_mut()
			.spawn(Hollow {
				radius: Units::from(10.),
			})
			.id();

		let may_collide = app
			.world_mut()
			.run_system_once(move |c: CheckHollowColliders| {
				let (a, b) = sort(a, b);
				c.may_collide(a, b)
			})?;

		assert!(may_collide);
		Ok(())
	}

	#[test_case(|a, b| (a, b); "forward")]
	#[test_case(|a, b| (b, a); "reversed")]
	fn smaller_sphere_partially_inside_offset_hollow(
		sort: fn(Entity, Entity) -> (Entity, Entity),
	) -> Result<(), RunSystemError> {
		let mut app = setup();
		let a = app
			.world_mut()
			.spawn(ColliderShape(Shape::Sphere {
				radius: Units::from(5.),
			}))
			.id();
		let b = app
			.world_mut()
			.spawn((
				GlobalTransform::from_xyz(8., 0., 0.),
				Hollow {
					radius: Units::from(10.),
				},
			))
			.id();

		let may_collide = app
			.world_mut()
			.run_system_once(move |c: CheckHollowColliders| {
				let (a, b) = sort(a, b);
				c.may_collide(a, b)
			})?;

		assert!(may_collide);
		Ok(())
	}
}
