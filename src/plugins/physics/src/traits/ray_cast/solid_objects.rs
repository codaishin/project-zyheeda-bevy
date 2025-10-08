use crate::traits::ray_cast::RayCaster;
use bevy::prelude::*;
use bevy_rapier3d::prelude::{Real, *};
use common::traits::handles_physics::{Raycast, RaycastHit, SolidObjects};

impl Raycast<SolidObjects> for RayCaster<'_, '_> {
	fn raycast(&self, ray: Ray3d, SolidObjects { exclude }: SolidObjects) -> Option<RaycastHit> {
		let ray_caster = self.context.single().ok()?;
		let mut filter = QueryFilter::default().exclude_sensors();

		for entity in exclude {
			filter = filter.exclude_rigid_body(entity);
		}

		let (entity, time_of_impact) =
			ray_caster.cast_ray(ray.origin, *ray.direction, Real::MAX, true, filter)?;

		Some(RaycastHit {
			entity,
			time_of_impact,
		})
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::{
		ecs::system::{RunSystemError, RunSystemOnce},
		render::mesh::MeshPlugin,
		scene::ScenePlugin,
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

		let hit = app.world_mut().run_system_once(|ray_caster: RayCaster| {
			ray_caster.raycast(Ray3d::new(Vec3::Y, Dir3::NEG_Y), SolidObjects::default())
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
	fn ignore_sensor() -> Result<(), RunSystemError> {
		let mut app = setup();
		app.world_mut().spawn((
			RigidBody::Fixed,
			Transform::default(),
			Collider::ball(0.5),
			Sensor,
		));
		app.update();

		let hit = app.world_mut().run_system_once(|ray_caster: RayCaster| {
			ray_caster.raycast(Ray3d::new(Vec3::Y, Dir3::NEG_Y), SolidObjects::default())
		})?;

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

		let hit = app
			.world_mut()
			.run_system_once(move |ray_caster: RayCaster| {
				ray_caster.raycast(
					Ray3d::new(Vec3::Y, Dir3::NEG_Y),
					SolidObjects {
						exclude: vec![entity],
					},
				)
			})?;

		assert_eq!(None, hit);
		Ok(())
	}
}
