use crate::{
	components::collider::{ChildColliderOf, MOUSE_HOVERABLE_GROUP, RAY_GROUP},
	system_params::ray_caster::RayCaster,
};
use bevy::prelude::*;
use bevy_rapier3d::prelude::{Real, *};
use common::traits::handles_physics::{Raycast, RaycastHit, SolidObjects};

impl Raycast<SolidObjects> for RayCaster<'_, '_> {
	fn raycast(
		&mut self,
		SolidObjects {
			ray,
			exclude,
			only_hoverable,
		}: SolidObjects,
	) -> Option<RaycastHit> {
		let ray_caster = self.context.single().ok()?;
		let not_excluded = self.not_excluded(exclude);
		let filter = QueryFilter {
			flags: QueryFilterFlags::EXCLUDE_SENSORS,
			predicate: Some(&not_excluded),
			groups: Some(CollisionGroups {
				memberships: RAY_GROUP,
				filters: match only_hoverable {
					true => MOUSE_HOVERABLE_GROUP,
					false => Group::all(),
				},
			}),
			..default()
		};

		let (entity, time_of_impact) =
			ray_caster.cast_ray(ray.origin, *ray.direction, Real::MAX, true, filter)?;

		if let Ok(ChildColliderOf(root)) = self.child_colliders.get(entity) {
			return Some(RaycastHit {
				entity: *root,
				time_of_impact,
			});
		};

		Some(RaycastHit {
			entity,
			time_of_impact,
		})
	}
}

impl RayCaster<'_, '_> {
	fn not_excluded(&self, exclude: Vec<Entity>) -> impl Fn(Entity) -> bool {
		move |entity| {
			if exclude.contains(&entity) {
				return false;
			}

			!matches!(
				self.child_colliders.get(entity),
				Ok(ChildColliderOf(root)) if exclude.contains(root),
			)
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{
		components::{
			collider::{ColliderRoot, MOUSE_HOVERABLE_GROUP},
			collision_domains::Physical,
		},
		tests::TestCollisionsPlugin,
	};
	use bevy::ecs::system::{RunSystemError, RunSystemOnce};
	use test_case::test_case;
	use testing::SingleThreadedApp;

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_plugins(TestCollisionsPlugin);
		app.add_observer(ColliderRoot::link_children);

		app
	}

	#[test]
	fn hit_object() -> Result<(), RunSystemError> {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((ColliderRoot, Transform::default(), Collider::ball(0.5)))
			.id();
		app.update();

		let hit = app
			.world_mut()
			.run_system_once(|mut ray_caster: RayCaster| {
				ray_caster.raycast(SolidObjects {
					ray: Ray3d::new(Vec3::Y, Dir3::NEG_Y),
					exclude: vec![],
					only_hoverable: false,
				})
			})?;

		assert_eq!(
			Some(RaycastHit {
				entity,
				time_of_impact: 0.5
			}),
			hit,
		);
		Ok(())
	}

	#[test]
	fn hit_object_interaction_target_root() -> Result<(), RunSystemError> {
		let mut app = setup();
		let root = app
			.world_mut()
			.spawn((
				ColliderRoot,
				children![(Transform::default(), Physical::Contact, Collider::ball(0.5))],
			))
			.id();
		app.world_mut().spawn(());
		app.update();

		let hit = app
			.world_mut()
			.run_system_once(|mut ray_caster: RayCaster| {
				ray_caster.raycast(SolidObjects {
					ray: Ray3d::new(Vec3::Y, Dir3::NEG_Y),
					exclude: vec![],
					only_hoverable: false,
				})
			})?;

		assert_eq!(
			Some(RaycastHit {
				entity: root,
				time_of_impact: 0.5,
			}),
			hit,
		);
		Ok(())
	}

	#[test]
	fn prefer_interaction_target_root() -> Result<(), RunSystemError> {
		let mut app = setup();
		let root = app
			.world_mut()
			.spawn((
				ColliderRoot,
				children![children![(
					Physical::Contact,
					Transform::default(),
					Collider::ball(0.5)
				)]],
			))
			.id();
		app.world_mut().spawn(());
		app.update();

		let hit = app
			.world_mut()
			.run_system_once(|mut ray_caster: RayCaster| {
				ray_caster.raycast(SolidObjects {
					ray: Ray3d::new(Vec3::Y, Dir3::NEG_Y),
					exclude: vec![],
					only_hoverable: false,
				})
			})?;

		assert_eq!(
			Some(RaycastHit {
				entity: root,
				time_of_impact: 0.5,
			}),
			hit,
		);
		Ok(())
	}

	#[test]
	fn ignore_sensor() -> Result<(), RunSystemError> {
		let mut app = setup();
		app.world_mut().spawn((
			ColliderRoot,
			Transform::default(),
			Collider::ball(0.5),
			Sensor,
		));
		app.update();

		let hit = app
			.world_mut()
			.run_system_once(|mut ray_caster: RayCaster| {
				ray_caster.raycast(SolidObjects {
					ray: Ray3d::new(Vec3::Y, Dir3::NEG_Y),
					exclude: vec![],
					only_hoverable: false,
				})
			})?;

		assert_eq!(None, hit);
		Ok(())
	}

	#[test]
	fn ignore_entity() -> Result<(), RunSystemError> {
		let mut app = setup();
		let b = app
			.world_mut()
			.spawn((Transform::default(), Collider::ball(0.5)))
			.id();
		let a = app
			.world_mut()
			.spawn((Transform::default(), Collider::ball(0.5)))
			.id();
		app.update();

		let hit = app
			.world_mut()
			.run_system_once(move |mut ray_caster: RayCaster| {
				ray_caster.raycast(SolidObjects {
					ray: Ray3d::new(Vec3::Y, Dir3::NEG_Y),
					exclude: vec![a, b],
					only_hoverable: false,
				})
			})?;

		assert_eq!(None, hit);
		Ok(())
	}

	#[test]
	fn ignore_child_entities() -> Result<(), RunSystemError> {
		let mut app = setup();
		let a = app
			.world_mut()
			.spawn((
				ColliderRoot,
				Transform::default(),
				children![(Transform::default(), Physical::Contact, Collider::ball(0.5))],
			))
			.id();
		let b = app
			.world_mut()
			.spawn((
				ColliderRoot,
				Transform::from_xyz(0., -10., 0.),
				children![(Transform::default(), Physical::Contact, Collider::ball(0.5))],
			))
			.id();
		app.update();

		let hit = app
			.world_mut()
			.run_system_once(move |mut ray_caster: RayCaster| {
				ray_caster.raycast(SolidObjects {
					ray: Ray3d::new(Vec3::Y, Dir3::NEG_Y),
					exclude: vec![a, b],
					only_hoverable: false,
				})
			})?;

		assert_eq!(None, hit);
		Ok(())
	}

	#[test]
	fn apply_only_hoverable_filter_true() -> Result<(), RunSystemError> {
		let mut app = setup();
		app.world_mut().spawn((
			ColliderRoot,
			CollisionGroups::new(Group::all() & !MOUSE_HOVERABLE_GROUP, RAY_GROUP),
			Transform::default(),
			Collider::ball(0.5),
		));
		let entity = app
			.world_mut()
			.spawn((
				ColliderRoot,
				Transform::from_xyz(0., -1., 0.),
				Collider::ball(0.5),
			))
			.id();
		app.update();

		let hit = app
			.world_mut()
			.run_system_once(move |mut ray_caster: RayCaster| {
				ray_caster.raycast(SolidObjects {
					ray: Ray3d::new(Vec3::Y, Dir3::NEG_Y),
					exclude: vec![],
					only_hoverable: true,
				})
			})?;

		assert_eq!(
			Some(RaycastHit {
				entity,
				time_of_impact: 1.5
			}),
			hit,
		);
		Ok(())
	}

	#[test]
	fn apply_only_hoverable_filter_false() -> Result<(), RunSystemError> {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((
				ColliderRoot,
				CollisionGroups::new(Group::all() & !MOUSE_HOVERABLE_GROUP, RAY_GROUP),
				Transform::default(),
				Collider::ball(0.5),
			))
			.id();
		app.world_mut().spawn((
			ColliderRoot,
			Transform::from_xyz(0., -1., 0.),
			Collider::ball(0.5),
		));
		app.update();

		let hit = app
			.world_mut()
			.run_system_once(move |mut ray_caster: RayCaster| {
				ray_caster.raycast(SolidObjects {
					ray: Ray3d::new(Vec3::Y, Dir3::NEG_Y),
					exclude: vec![],
					only_hoverable: false,
				})
			})?;

		assert_eq!(
			Some(RaycastHit {
				entity,
				time_of_impact: 0.5
			}),
			hit,
		);
		Ok(())
	}

	#[test_case(true; "only hoverable")]
	#[test_case(false; "also non hoverable")]
	fn ignore_bodies_that_do_not_filter_rays(only_hoverable: bool) -> Result<(), RunSystemError> {
		let mut app = setup();
		app.world_mut().spawn((
			ColliderRoot,
			CollisionGroups::new(Group::all(), Group::all() & !RAY_GROUP),
			Transform::default(),
			Collider::ball(0.5),
		));
		let entity = app
			.world_mut()
			.spawn((
				ColliderRoot,
				Transform::from_xyz(0., -1., 0.),
				Collider::ball(0.5),
			))
			.id();
		app.update();

		let hit = app
			.world_mut()
			.run_system_once(move |mut ray_caster: RayCaster| {
				ray_caster.raycast(SolidObjects {
					ray: Ray3d::new(Vec3::Y, Dir3::NEG_Y),
					exclude: vec![],
					only_hoverable,
				})
			})?;

		assert_eq!(
			Some(RaycastHit {
				entity,
				time_of_impact: 1.5
			}),
			hit,
		);
		Ok(())
	}
}
