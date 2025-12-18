use crate::components::{colliders::ColliderShape, hollow::Hollow};
use bevy::{ecs::system::SystemParam, prelude::*};
use bevy_rapier3d::prelude::*;
use common::tools::Units;

#[derive(SystemParam)]
pub(crate) struct CheckHollowColliders<'w, 's, TCollider = ColliderShape>
where
	TCollider: Component + SimpleOuterRadius,
{
	colliders: Query<'w, 's, (&'static TCollider, &'static GlobalTransform)>,
	hollows: Query<'w, 's, (&'static Hollow, &'static GlobalTransform)>,
}

impl<TCollider> CheckHollowColliders<'_, '_, TCollider>
where
	TCollider: Component + SimpleOuterRadius,
{
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
		let (collider, collider_offset) = match self.colliders.get(collider) {
			Ok((collider, t)) => (collider, t.translation()),
			_ => return false,
		};
		let Some(collider_radius) = collider.simple_outer_radius() else {
			return false;
		};
		let offset = (collider_offset - hollow_offset).length();

		offset + *collider_radius <= hollow_radius
	}
}

impl BevyPhysicsHooks for CheckHollowColliders<'_, '_> {
	fn filter_intersection_pair(&self, context: PairFilterContextView) -> bool {
		self.may_collide(context.collider1(), context.collider2())
	}
}

pub(crate) trait SimpleOuterRadius {
	fn simple_outer_radius(&self) -> Option<Units>;
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::ecs::system::{RunSystemError, RunSystemOnce};
	use common::tools::Units;
	use test_case::test_case;
	use testing::SingleThreadedApp;

	#[derive(Component)]
	#[require(Transform)]
	struct _Collider {
		radius: Units,
	}

	impl SimpleOuterRadius for _Collider {
		fn simple_outer_radius(&self) -> Option<Units> {
			Some(self.radius)
		}
	}

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
			.spawn(_Collider {
				radius: Units::from(11.),
			})
			.id();
		let b = app
			.world_mut()
			.spawn(Hollow {
				radius: Units::from(42.),
			})
			.id();

		let may_collide =
			app.world_mut()
				.run_system_once(move |c: CheckHollowColliders<_Collider>| {
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
			.spawn(_Collider {
				radius: Units::from(42.),
			})
			.id();
		let b = app
			.world_mut()
			.spawn(Hollow {
				radius: Units::from(11.),
			})
			.id();

		let may_collide =
			app.world_mut()
				.run_system_once(move |c: CheckHollowColliders<_Collider>| {
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
				_Collider {
					radius: Units::from(5.),
				},
			))
			.id();
		let b = app
			.world_mut()
			.spawn(Hollow {
				radius: Units::from(10.),
			})
			.id();

		let may_collide =
			app.world_mut()
				.run_system_once(move |c: CheckHollowColliders<_Collider>| {
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
			.spawn(_Collider {
				radius: Units::from(5.),
			})
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

		let may_collide =
			app.world_mut()
				.run_system_once(move |c: CheckHollowColliders<_Collider>| {
					let (a, b) = sort(a, b);
					c.may_collide(a, b)
				})?;

		assert!(may_collide);
		Ok(())
	}
}
