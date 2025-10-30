use crate::traits::ray_cast::RayCaster;
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
		let mut filter = QueryFilter::default().exclude_sensors();
		let filter_only_hoverable = |e| !self.no_mouse_hovers.contains(e);

		if only_hoverable {
			filter = filter.predicate(&filter_only_hoverable);
		}

		for entity in exclude {
			filter = filter.exclude_rigid_body(entity);
		}

		let (entity, time_of_impact) =
			ray_caster.cast_ray(ray.origin, *ray.direction, Real::MAX, true, filter)?;

		let Ok(sub_collider) = self.interaction_colliders.get(entity) else {
			return Some(RaycastHit {
				entity,
				time_of_impact,
			});
		};

		Some(RaycastHit {
			entity: sub_collider.target(),
			time_of_impact,
		})
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{PhysicsPlugin, components::no_hover::NoMouseHover};
	use bevy::{
		ecs::system::{RunSystemError, RunSystemOnce},
		prelude::*,
		render::mesh::MeshPlugin,
		scene::ScenePlugin,
	};
	use common::{
		components::collider_relationship::ColliderOfInteractionTarget,
		traits::handles_physics::RaycastSystemParam,
	};
	use testing::SingleThreadedApp;

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_plugins((
			MinimalPlugins,
			AssetPlugin::default(),
			MeshPlugin,
			ScenePlugin,
			RapierPhysicsPlugin::<NoUserData>::default(),
		));

		app
	}

	#[test]
	fn hit_object() -> Result<(), RunSystemError> {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((RigidBody::Fixed, Transform::default(), Collider::ball(0.5)))
			.id();
		app.update();

		let hit = app.world_mut().run_system_once(
			|mut ray_caster: RaycastSystemParam<PhysicsPlugin<()>>| {
				ray_caster.raycast(SolidObjects {
					ray: Ray3d::new(Vec3::Y, Dir3::NEG_Y),
					exclude: vec![],
					only_hoverable: false,
				})
			},
		)?;

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
	fn hit_object_root() -> Result<(), RunSystemError> {
		let mut app = setup();
		let root = app.world_mut().spawn_empty().id();
		app.world_mut().spawn((
			RigidBody::Fixed,
			Transform::default(),
			Collider::ball(0.5),
			ColliderOfInteractionTarget::from_raw(root),
		));
		app.update();

		let hit = app.world_mut().run_system_once(
			|mut ray_caster: RaycastSystemParam<PhysicsPlugin<()>>| {
				ray_caster.raycast(SolidObjects {
					ray: Ray3d::new(Vec3::Y, Dir3::NEG_Y),
					exclude: vec![],
					only_hoverable: false,
				})
			},
		)?;

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
			RigidBody::Fixed,
			Transform::default(),
			Collider::ball(0.5),
			Sensor,
		));
		app.update();

		let hit = app.world_mut().run_system_once(
			|mut ray_caster: RaycastSystemParam<PhysicsPlugin<()>>| {
				ray_caster.raycast(SolidObjects {
					ray: Ray3d::new(Vec3::Y, Dir3::NEG_Y),
					exclude: vec![],
					only_hoverable: false,
				})
			},
		)?;

		assert_eq!(None, hit);
		Ok(())
	}

	#[test]
	fn ignore_object_rigid_body() -> Result<(), RunSystemError> {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((
				RigidBody::Fixed,
				Transform::default(),
				children![(Transform::default(), Collider::ball(0.5))],
			))
			.id();
		app.update();

		let hit = app.world_mut().run_system_once(
			move |mut ray_caster: RaycastSystemParam<PhysicsPlugin<()>>| {
				ray_caster.raycast(SolidObjects {
					ray: Ray3d::new(Vec3::Y, Dir3::NEG_Y),
					exclude: vec![entity],
					only_hoverable: false,
				})
			},
		)?;

		assert_eq!(None, hit);
		Ok(())
	}

	#[test]
	fn ignore_entities_with_non_hover_component() -> Result<(), RunSystemError> {
		let mut app = setup();
		app.world_mut().spawn((
			RigidBody::Fixed,
			NoMouseHover,
			Transform::default(),
			Collider::ball(0.5),
		));
		let entity = app
			.world_mut()
			.spawn((
				RigidBody::Fixed,
				Transform::from_xyz(0., -1., 0.),
				Collider::ball(0.5),
			))
			.id();
		app.update();

		let hit = app.world_mut().run_system_once(
			move |mut ray_caster: RaycastSystemParam<PhysicsPlugin<()>>| {
				ray_caster.raycast(SolidObjects {
					ray: Ray3d::new(Vec3::Y, Dir3::NEG_Y),
					exclude: vec![],
					only_hoverable: true,
				})
			},
		)?;

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
	fn do_not_ignore_entities_with_non_hover_component() -> Result<(), RunSystemError> {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((
				RigidBody::Fixed,
				NoMouseHover,
				Transform::default(),
				Collider::ball(0.5),
			))
			.id();
		app.world_mut().spawn((
			RigidBody::Fixed,
			Transform::from_xyz(0., -1., 0.),
			Collider::ball(0.5),
		));
		app.update();

		let hit = app.world_mut().run_system_once(
			move |mut ray_caster: RaycastSystemParam<PhysicsPlugin<()>>| {
				ray_caster.raycast(SolidObjects {
					ray: Ray3d::new(Vec3::Y, Dir3::NEG_Y),
					exclude: vec![],
					only_hoverable: false,
				})
			},
		)?;

		assert_eq!(
			Some(RaycastHit {
				entity,
				time_of_impact: 0.5
			}),
			hit,
		);
		Ok(())
	}
}
