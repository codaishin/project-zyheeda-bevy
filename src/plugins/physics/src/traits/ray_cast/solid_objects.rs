use crate::{components::collider::ChildCollider, traits::ray_cast::RayCaster};
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
		let exclude = |e| {
			if only_hoverable && self.no_mouse_hovers.contains(e) {
				return false;
			}

			if exclude.contains(&e) {
				return false;
			}

			match self.interaction_child_colliders.get(e) {
				Ok(ChildCollider { root, .. }) if exclude.contains(root) => {
					return false;
				}
				_ => {}
			};

			match self.rigid_body_child_colliders.get(e) {
				Ok(ChildCollider { root, .. }) if exclude.contains(root) => {
					return false;
				}
				_ => {}
			};

			true
		};

		let (entity, time_of_impact) = ray_caster.cast_ray(
			ray.origin,
			*ray.direction,
			Real::MAX,
			true,
			QueryFilter::default().exclude_sensors().predicate(&exclude),
		)?;

		if let Ok(ChildCollider { root, .. }) = self.interaction_child_colliders.get(entity) {
			return Some(RaycastHit {
				entity: *root,
				time_of_impact,
			});
		};

		if let Ok(ChildCollider { root, .. }) = self.rigid_body_child_colliders.get(entity) {
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

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{
		PhysicsPlugin,
		components::{interaction_target::InteractionTarget, no_hover::NoMouseHover},
	};
	use bevy::{
		ecs::system::{RunSystemError, RunSystemOnce},
		mesh::MeshPlugin,
		prelude::*,
		scene::ScenePlugin,
	};
	use common::traits::handles_physics::RaycastSystemParam;
	use test_case::test_case;
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
		app.add_observer(ChildCollider::<InteractionTarget>::link);
		app.add_observer(ChildCollider::<RigidBody>::link);

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
	fn hit_object_interaction_target_root() -> Result<(), RunSystemError> {
		let mut app = setup();
		let root = app
			.world_mut()
			.spawn((
				InteractionTarget,
				children![(Transform::default(), Collider::ball(0.5))],
			))
			.id();
		app.world_mut().spawn(());
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
	fn hit_object_rigid_body_root() -> Result<(), RunSystemError> {
		let mut app = setup();
		let root = app
			.world_mut()
			.spawn((
				RigidBody::Fixed,
				children![(Transform::default(), Collider::ball(0.5))],
			))
			.id();
		app.world_mut().spawn(());
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
	fn prefer_interaction_target_root() -> Result<(), RunSystemError> {
		let mut app = setup();
		let root = app
			.world_mut()
			.spawn((
				InteractionTarget,
				children![(
					RigidBody::Fixed,
					children![(Transform::default(), Collider::ball(0.5))]
				)],
			))
			.id();
		app.world_mut().spawn(());
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

		let hit = app.world_mut().run_system_once(
			move |mut ray_caster: RaycastSystemParam<PhysicsPlugin<()>>| {
				ray_caster.raycast(SolidObjects {
					ray: Ray3d::new(Vec3::Y, Dir3::NEG_Y),
					exclude: vec![a, b],
					only_hoverable: false,
				})
			},
		)?;

		assert_eq!(None, hit);
		Ok(())
	}

	#[test_case(RigidBody::Fixed; "rigid body")]
	#[test_case(InteractionTarget; "interaction target")]
	fn ignore_child_entities<TMarker>(maker: TMarker) -> Result<(), RunSystemError>
	where
		TMarker: Component + Copy,
	{
		let mut app = setup();
		let b = app
			.world_mut()
			.spawn((
				maker,
				Transform::default(),
				children![(Transform::default(), Collider::ball(0.5))],
			))
			.id();
		let a = app
			.world_mut()
			.spawn((
				maker,
				Transform::default(),
				children![(Transform::default(), Collider::ball(0.5))],
			))
			.id();
		app.update();

		let hit = app.world_mut().run_system_once(
			move |mut ray_caster: RaycastSystemParam<PhysicsPlugin<()>>| {
				ray_caster.raycast(SolidObjects {
					ray: Ray3d::new(Vec3::Y, Dir3::NEG_Y),
					exclude: vec![a, b],
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
