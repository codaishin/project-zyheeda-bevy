use crate::{
	components::collider::{RAY_GROUP, TERRAIN_GROUP},
	system_params::ray_caster::RayCasterMut,
};
use bevy::prelude::*;
use bevy_rapier3d::{math::Real, prelude::*};
use common::traits::handles_physics::{Raycast, Terrain, TimeOfImpact};

impl Raycast<Terrain> for RayCasterMut<'_, '_> {
	fn raycast(&mut self, Terrain { ray }: Terrain) -> Option<TimeOfImpact> {
		let ray_caster = self.context.single().ok()?;

		let (_, toi) = ray_caster.cast_ray(
			ray.origin,
			*ray.direction,
			Real::MAX,
			true,
			QueryFilter {
				groups: Some(CollisionGroups::new(RAY_GROUP, TERRAIN_GROUP)),
				..default()
			},
		)?;

		TimeOfImpact::try_from_f32(toi).ok()
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::resources::world_camera::WorldCamera;
	use bevy::{
		ecs::system::{RunSystemError, RunSystemOnce},
		mesh::MeshPlugin,
		scene::ScenePlugin,
	};
	use common::tools::Units;
	use testing::SingleThreadedApp;

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_plugins((
			MinimalPlugins,
			TransformPlugin,
			AssetPlugin::default(),
			MeshPlugin,
			ScenePlugin,
			RapierPhysicsPlugin::<NoUserData>::default(),
		));
		app.init_resource::<WorldCamera>();

		app
	}

	#[test]
	fn raycast_onto_object() -> Result<(), RunSystemError> {
		let mut app = setup();
		let ray = Ray3d {
			origin: Vec3::Y,
			direction: Dir3::NEG_Y,
		};
		app.world_mut().spawn((
			CollisionGroups::new(TERRAIN_GROUP, RAY_GROUP),
			Collider::ball(0.5),
		));
		app.update();

		let hit = app
			.world_mut()
			.run_system_once(move |mut ray_caster: RayCasterMut| {
				ray_caster.raycast(Terrain { ray })
			})?;

		assert_eq!(Some(TimeOfImpact::from(Units::from(0.5))), hit);
		Ok(())
	}

	#[test]
	fn no_raycast_onto_object_if_not_terrain_member() -> Result<(), RunSystemError> {
		let mut app = setup();
		let ray = Ray3d {
			origin: Vec3::Y,
			direction: Dir3::NEG_Y,
		};
		app.world_mut().spawn((
			CollisionGroups {
				memberships: Group::all() & !TERRAIN_GROUP,
				filters: RAY_GROUP,
			},
			Collider::ball(0.5),
		));
		app.update();

		let hit = app
			.world_mut()
			.run_system_once(move |mut ray_caster: RayCasterMut| {
				ray_caster.raycast(Terrain { ray })
			})?;

		assert_eq!(None, hit);
		Ok(())
	}

	#[test]
	fn no_raycast_onto_object_if_not_filtering_rays() -> Result<(), RunSystemError> {
		let mut app = setup();
		let ray = Ray3d {
			origin: Vec3::Y,
			direction: Dir3::NEG_Y,
		};
		app.world_mut().spawn((
			CollisionGroups {
				memberships: TERRAIN_GROUP,
				filters: Group::all() & !RAY_GROUP,
			},
			Collider::ball(0.5),
		));
		app.update();

		let hit = app
			.world_mut()
			.run_system_once(move |mut ray_caster: RayCasterMut| {
				ray_caster.raycast(Terrain { ray })
			})?;

		assert_eq!(None, hit);
		Ok(())
	}
}
