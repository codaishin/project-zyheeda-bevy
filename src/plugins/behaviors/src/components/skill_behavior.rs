pub mod skill_contact;
pub mod skill_projection;

use super::{
	ground_target::GroundTarget,
	set_position_and_rotation::SetPositionAndRotation,
	set_to_move_forward::SetVelocityForward,
	when_traveled_insert::WhenTraveled,
	Always,
	Once,
};
use bevy::{ecs::system::EntityCommands, prelude::*};
use bevy_rapier3d::prelude::*;
use common::{
	components::{AssetModel, ColliderRoot},
	errors::{Error, Level},
	traits::{
		handles_destruction::HandlesDestruction,
		handles_interactions::HandlesInteractions,
		handles_skill_behaviors::{Integrity, Motion, Shape},
	},
};
use std::f32::consts::PI;

trait SimplePrefab {
	type TExtra;

	fn prefab<TInteractions, TLifeCycles>(
		&self,
		entity: &mut EntityCommands,
		extra: Self::TExtra,
	) -> Result<(), Error>
	where
		TInteractions: HandlesInteractions,
		TLifeCycles: HandlesDestruction;
}

const SPHERE_MODEL: &str = "models/sphere.glb";

impl SimplePrefab for Shape {
	type TExtra = Vec3;

	fn prefab<TInteractions, TLifeCycles>(
		&self,
		entity: &mut EntityCommands,
		offset: Vec3,
	) -> Result<(), Error> {
		let ((model, model_transform), (collider, collider_transform)) = match self {
			Shape::Sphere {
				radius,
				hollow_collider,
			} => (
				(
					AssetModel::path(SPHERE_MODEL),
					Transform::from_scale(Vec3::splat(**radius * 2.)),
				),
				match hollow_collider {
					true => ring_collider(**radius)?,
					false => sphere_collider(**radius),
				},
			),
			Shape::Custom {
				model,
				collider,
				scale,
			} => (
				(model.clone(), Transform::from_scale(*scale)),
				custom_collider(collider, *scale),
			),
		};

		entity
			.try_insert((Transform::from_translation(offset), Visibility::default()))
			.with_children(|parent| {
				parent.spawn((model, model_transform));
				parent.spawn((
					ColliderRoot(parent.parent_entity()),
					collider,
					collider_transform,
					ActiveEvents::COLLISION_EVENTS,
					Sensor,
				));
			});

		Ok(())
	}
}

fn sphere_collider(radius: f32) -> (Collider, Transform) {
	(Collider::ball(radius), Transform::default())
}

fn ring_collider(radius: f32) -> Result<(Collider, Transform), Error> {
	let transform = Transform::default().with_rotation(Quat::from_axis_angle(Vec3::X, PI / 2.));
	let ring = Annulus::new(radius * 0.9, radius);
	let torus = Mesh::from(Extrusion::new(ring, radius * 2.));
	let collider = Collider::from_bevy_mesh(
		&torus,
		&ComputedColliderShape::TriMesh(TriMeshFlags::MERGE_DUPLICATE_VERTICES),
	);

	let Some(collider) = collider else {
		return Err(Error {
			msg: "Cannot create spherical contact collider".to_owned(),
			lvl: Level::Error,
		});
	};

	Ok((collider, transform))
}

fn custom_collider(collider: &Collider, scale: Vec3) -> (Collider, Transform) {
	(collider.clone(), Transform::from_scale(scale))
}

impl SimplePrefab for Integrity {
	type TExtra = ();

	fn prefab<TInteractions, TLifeCycles>(
		&self,
		entity: &mut EntityCommands,
		_: (),
	) -> Result<(), Error>
	where
		TInteractions: HandlesInteractions,
	{
		match self {
			Integrity::Solid => {}
			Integrity::Fragile { destroyed_by } => {
				entity.try_insert(TInteractions::is_fragile_when_colliding_with(destroyed_by));
			}
		};

		Ok(())
	}
}

impl SimplePrefab for Motion {
	type TExtra = ();

	fn prefab<TInteractions, TLifeCycles>(
		&self,
		entity: &mut EntityCommands,
		_: (),
	) -> Result<(), Error>
	where
		TLifeCycles: HandlesDestruction,
	{
		match *self {
			Motion::HeldBy { spawner } => {
				entity.try_insert((
					RigidBody::Fixed,
					SetPositionAndRotation::<Always>::to(spawner),
				));
			}
			Motion::Stationary {
				caster,
				max_cast_range,
				target_ray,
			} => {
				entity.try_insert((
					RigidBody::Fixed,
					GroundTarget {
						caster,
						max_cast_range,
						target_ray,
					},
				));
			}
			Motion::Projectile {
				caster,
				spawner,
				speed,
				max_range,
			} => {
				entity.try_insert((
					RigidBody::Dynamic,
					GravityScale(0.),
					Ccd::enabled(),
					SetPositionAndRotation::<Once>::to(spawner),
					SetVelocityForward {
						rotation: caster,
						speed,
					},
					WhenTraveled::via::<Velocity>()
						.distance(max_range)
						.insert::<TLifeCycles::TDestroy>(),
				));
			}
		}
		Ok(())
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::when_traveled_insert::InsertAfterDistanceTraveled;
	use bevy::ecs::system::{RunSystemError, RunSystemOnce};
	use bevy_rapier3d::prelude::ActiveCollisionTypes;
	use common::{
		blocker::Blocker,
		components::{AssetModel, ColliderRoot},
		tools::{Units, UnitsPerSecond},
		traits::{
			clamp_zero_positive::ClampZeroPositive,
			handles_destruction::HandlesDestruction,
			handles_interactions::{BeamParameters, HandlesInteractions},
		},
	};

	struct _Interactions;

	impl HandlesInteractions for _Interactions {
		fn is_fragile_when_colliding_with(blockers: &[Blocker]) -> impl Bundle {
			_IsFragile(Vec::from(blockers))
		}

		fn is_ray_interrupted_by(_: &[Blocker]) -> impl Bundle {}

		fn beam_from<T>(_: &T) -> impl Bundle
		where
			T: BeamParameters,
		{
		}
	}

	struct _LifeCycles;

	impl HandlesDestruction for _LifeCycles {
		type TDestroy = _Destroy;
	}

	#[derive(Component, Debug, PartialEq, Default)]
	struct _Destroy;

	#[derive(Component, Debug, PartialEq)]
	struct _IsFragile(Vec<Blocker>);

	fn test_system<T>(
		exec: impl Fn(&mut EntityCommands) -> T,
	) -> impl Fn(Commands, Query<Entity>) -> T {
		move |mut commands, query| {
			let entity = query.single();
			let mut entity = commands.entity(entity);

			exec(&mut entity)
		}
	}

	fn setup() -> (App, Entity) {
		let mut app = App::new();
		let entity = app.world_mut().spawn_empty().id();

		(app, entity)
	}

	#[test]
	fn held_components() -> Result<(), RunSystemError> {
		let (mut app, entity) = setup();
		let motion = Motion::HeldBy {
			spawner: Entity::from_raw(11),
		};

		_ = app.world_mut().run_system_once(test_system(move |entity| {
			motion.prefab::<_Interactions, _LifeCycles>(entity, ())
		}))?;

		assert_eq!(
			(
				Some(&RigidBody::Fixed),
				None,
				None,
				None,
				Some(&SetPositionAndRotation::<Always>::to(Entity::from_raw(11))),
				None,
				None,
				None,
			),
			(
				app.world().entity(entity).get::<RigidBody>(),
				app.world().entity(entity).get::<GravityScale>(),
				app.world().entity(entity).get::<Ccd>(),
				app.world().entity(entity).get::<GroundTarget>(),
				app.world()
					.entity(entity)
					.get::<SetPositionAndRotation<Always>>(),
				app.world()
					.entity(entity)
					.get::<SetPositionAndRotation<Once>>(),
				app.world().entity(entity).get::<SetVelocityForward>(),
				app.world()
					.entity(entity)
					.get::<InsertAfterDistanceTraveled<_Destroy, Velocity>>(),
			)
		);
		Ok(())
	}

	#[test]
	fn stationary_components() -> Result<(), RunSystemError> {
		let (mut app, entity) = setup();
		let motion = Motion::Stationary {
			caster: Entity::from_raw(42),
			max_cast_range: Units::new(42.),
			target_ray: Ray3d {
				origin: Vec3::new(11., 12., 13.),
				direction: Dir3::from_xyz(2., 1., 4.).unwrap(),
			},
		};

		_ = app.world_mut().run_system_once(test_system(move |entity| {
			motion.prefab::<_Interactions, _LifeCycles>(entity, ())
		}))?;

		assert_eq!(
			(
				Some(&RigidBody::Fixed),
				None,
				None,
				Some(&GroundTarget {
					caster: Entity::from_raw(42),
					max_cast_range: Units::new(42.),
					target_ray: Ray3d {
						origin: Vec3::new(11., 12., 13.),
						direction: Dir3::from_xyz(2., 1., 4.).unwrap(),
					},
				}),
				None,
				None,
				None,
				None,
			),
			(
				app.world().entity(entity).get::<RigidBody>(),
				app.world().entity(entity).get::<GravityScale>(),
				app.world().entity(entity).get::<Ccd>(),
				app.world().entity(entity).get::<GroundTarget>(),
				app.world()
					.entity(entity)
					.get::<SetPositionAndRotation<Always>>(),
				app.world()
					.entity(entity)
					.get::<SetPositionAndRotation<Once>>(),
				app.world().entity(entity).get::<SetVelocityForward>(),
				app.world()
					.entity(entity)
					.get::<InsertAfterDistanceTraveled<_Destroy, Velocity>>(),
			)
		);
		Ok(())
	}

	#[test]
	fn projectile_components() -> Result<(), RunSystemError> {
		let (mut app, entity) = setup();
		let motion = Motion::Projectile {
			caster: Entity::from_raw(55),
			spawner: Entity::from_raw(66),
			speed: UnitsPerSecond::new(11.),
			max_range: Units::new(1111.),
		};

		_ = app.world_mut().run_system_once(test_system(move |entity| {
			motion.prefab::<_Interactions, _LifeCycles>(entity, ())
		}))?;

		assert_eq!(
			(
				Some(&RigidBody::Dynamic),
				Some(&GravityScale(0.)),
				Some(&Ccd::enabled()),
				None,
				None,
				Some(&SetPositionAndRotation::<Once>::to(Entity::from_raw(66))),
				Some(&SetVelocityForward {
					rotation: Entity::from_raw(55),
					speed: UnitsPerSecond::new(11.),
				}),
				Some(
					&WhenTraveled::via::<Velocity>()
						.distance(Units::new(1111.))
						.insert::<_Destroy>()
				),
			),
			(
				app.world().entity(entity).get::<RigidBody>(),
				app.world().entity(entity).get::<GravityScale>(),
				app.world().entity(entity).get::<Ccd>(),
				app.world().entity(entity).get::<GroundTarget>(),
				app.world()
					.entity(entity)
					.get::<SetPositionAndRotation<Always>>(),
				app.world()
					.entity(entity)
					.get::<SetPositionAndRotation<Once>>(),
				app.world().entity(entity).get::<SetVelocityForward>(),
				app.world()
					.entity(entity)
					.get::<InsertAfterDistanceTraveled<_Destroy, Velocity>>(),
			)
		);
		Ok(())
	}

	#[test]
	fn fragile_components() -> Result<(), RunSystemError> {
		let (mut app, entity) = setup();
		let integrity = Integrity::Fragile {
			destroyed_by: vec![Blocker::Physical],
		};

		_ = app.world_mut().run_system_once(test_system(move |entity| {
			integrity.prefab::<_Interactions, _LifeCycles>(entity, ())
		}))?;

		assert_eq!(
			Some(&_IsFragile(vec![Blocker::Physical])),
			app.world().entity(entity).get::<_IsFragile>(),
		);
		Ok(())
	}

	fn children_of(app: &App, entity: Entity) -> impl Iterator<Item = EntityRef> {
		app.world().iter_entities().filter(move |e| {
			e.get::<Parent>()
				.map(|p| p.get() == entity)
				.unwrap_or(false)
		})
	}

	#[test]
	fn transform_and_visibility_sphere() -> Result<(), RunSystemError> {
		let (mut app, entity) = setup();
		let shape = Shape::Sphere {
			radius: Units::new(42.),
			hollow_collider: false,
		};

		_ = app.world_mut().run_system_once(test_system(move |entity| {
			shape.prefab::<_Interactions, _LifeCycles>(entity, Vec3::new(1., 2., 3.))
		}))?;

		assert_eq!(
			(
				Some(&Transform::from_xyz(1., 2., 3.)),
				Some(&Visibility::Inherited)
			),
			(
				app.world().entity(entity).get::<Transform>(),
				app.world().entity(entity).get::<Visibility>(),
			)
		);
		Ok(())
	}

	#[test]
	fn transform_and_visibility_custom_shape() -> Result<(), RunSystemError> {
		let (mut app, entity) = setup();
		let shape = Shape::Custom {
			model: AssetModel::path(""),
			collider: Collider::default(),
			scale: Vec3::new(1., 2., 3.),
		};

		_ = app.world_mut().run_system_once(test_system(move |entity| {
			shape.prefab::<_Interactions, _LifeCycles>(entity, Vec3::new(1., 2., 3.))
		}))?;

		assert_eq!(
			(
				Some(&Transform::from_xyz(1., 2., 3.)),
				Some(&Visibility::Inherited)
			),
			(
				app.world().entity(entity).get::<Transform>(),
				app.world().entity(entity).get::<Visibility>(),
			)
		);
		Ok(())
	}

	#[test]
	fn shape_sphere() -> Result<(), RunSystemError> {
		let (mut app, entity) = setup();
		let shape = Shape::Sphere {
			radius: Units::new(42.),
			hollow_collider: false,
		};

		_ = app.world_mut().run_system_once(test_system(move |entity| {
			shape.prefab::<_Interactions, _LifeCycles>(entity, Vec3::ZERO)
		}))?;

		let child = children_of(&app, entity)
			.next()
			.expect("no entity children");
		assert_eq!(
			(
				Some(&Transform::from_scale(Vec3::splat(42. * 2.))),
				Some(&AssetModel::path(SPHERE_MODEL)),
			),
			(child.get::<Transform>(), child.get::<AssetModel>(),)
		);
		Ok(())
	}

	#[test]
	fn collider_sphere_sphere() -> Result<(), RunSystemError> {
		let (mut app, entity) = setup();
		let shape = Shape::Sphere {
			radius: Units::new(42.),
			hollow_collider: false,
		};

		_ = app.world_mut().run_system_once(test_system(move |entity| {
			shape.prefab::<_Interactions, _LifeCycles>(entity, Vec3::ZERO)
		}))?;

		let child = children_of(&app, entity)
			.nth(1)
			.expect("no second entity children");
		assert_eq!(
			(
				Some(&Transform::default()),
				Some(Collider::ball(42.).as_ball().map(|b| b.raw)),
				Some(&ActiveEvents::COLLISION_EVENTS),
				Some(&ActiveCollisionTypes::default()),
			),
			(
				child.get::<Transform>(),
				child.get::<Collider>().map(|c| c.as_ball().map(|c| c.raw)),
				child.get::<ActiveEvents>(),
				child.get::<ActiveCollisionTypes>(),
			),
		);
		Ok(())
	}

	#[test]
	fn collider_sphere_hollow_as_ring() -> Result<(), RunSystemError> {
		let (mut app, entity) = setup();
		let shape = Shape::Sphere {
			radius: Units::new(42.),
			hollow_collider: true,
		};

		_ = app.world_mut().run_system_once(test_system(move |entity| {
			shape.prefab::<_Interactions, _LifeCycles>(entity, Vec3::ZERO)
		}))?;

		let child = children_of(&app, entity)
			.nth(1)
			.expect("no second entity children");
		let (_expected_collider, expected_transform) = ring_collider(42.).unwrap();
		assert_eq!(
			(
				Some(&expected_transform),
				/* FIXME: Can't compare tri meshes, but in principle this is the shape we'd expect:
				 * Some(_expected_collider.as_trimesh().map(|c| c.raw)),
				 */
				Some(true),
				Some(&ActiveEvents::COLLISION_EVENTS),
				Some(&ActiveCollisionTypes::default()),
			),
			(
				child.get::<Transform>(),
				/* FIXME: Can't compare tri meshes, but in principle this is the shape we spawned:
				 * child.get::<Collider>().map(|c| c.as_trimesh().map(|c| c.raw)),
				 */
				child.get::<Collider>().map(|c| c.as_trimesh().is_some()),
				child.get::<ActiveEvents>(),
				child.get::<ActiveCollisionTypes>(),
			),
		);
		Ok(())
	}

	#[test]
	fn collider_sphere_sensor() -> Result<(), RunSystemError> {
		let (mut app, entity) = setup();
		let shape = Shape::Sphere {
			radius: Units::new(42.),
			hollow_collider: false,
		};

		_ = app.world_mut().run_system_once(test_system(move |entity| {
			shape.prefab::<_Interactions, _LifeCycles>(entity, Vec3::ZERO)
		}))?;

		let child = children_of(&app, entity)
			.nth(1)
			.expect("no second entity children");
		assert!(child.contains::<Sensor>());
		Ok(())
	}

	#[test]
	fn collider_sphere_root() -> Result<(), RunSystemError> {
		let (mut app, entity) = setup();
		let shape = Shape::Sphere {
			radius: Units::new(42.),
			hollow_collider: false,
		};

		_ = app.world_mut().run_system_once(test_system(move |entity| {
			shape.prefab::<_Interactions, _LifeCycles>(entity, Vec3::ZERO)
		}))?;

		let child = children_of(&app, entity)
			.nth(1)
			.expect("no second entity children");
		assert_eq!(Some(&ColliderRoot(entity)), child.get::<ColliderRoot>());
		Ok(())
	}

	#[test]
	fn shape_custom() -> Result<(), RunSystemError> {
		let (mut app, entity) = setup();
		let shape = Shape::Custom {
			model: AssetModel::path("custom"),
			collider: Collider::cuboid(1., 2., 3.),
			scale: Vec3::new(3., 2., 1.),
		};

		_ = app.world_mut().run_system_once(test_system(move |entity| {
			shape.prefab::<_Interactions, _LifeCycles>(entity, Vec3::ZERO)
		}))?;

		let child = children_of(&app, entity)
			.next()
			.expect("no entity children");
		assert_eq!(
			(
				Some(&AssetModel::path("custom")),
				Some(&Transform::from_scale(Vec3::new(3., 2., 1.))),
			),
			(child.get::<AssetModel>(), child.get::<Transform>()),
		);
		Ok(())
	}

	#[test]
	fn collider_custom() -> Result<(), RunSystemError> {
		let (mut app, entity) = setup();
		let shape = Shape::Custom {
			model: AssetModel::path("custom"),
			collider: Collider::cuboid(1., 2., 3.),
			scale: Vec3::new(3., 2., 1.),
		};

		_ = app.world_mut().run_system_once(test_system(move |entity| {
			shape.prefab::<_Interactions, _LifeCycles>(entity, Vec3::ZERO)
		}))?;

		let child = children_of(&app, entity)
			.nth(1)
			.expect("no second entity children");
		assert_eq!(
			(
				Some(&Transform::from_scale(Vec3::new(3., 2., 1.))),
				Some(Collider::cuboid(1., 2., 3.).as_cuboid().map(|b| b.raw)),
				Some(&ActiveEvents::COLLISION_EVENTS),
				Some(&ActiveCollisionTypes::default()),
			),
			(
				child.get::<Transform>(),
				child
					.get::<Collider>()
					.map(|c| c.as_cuboid().map(|c| c.raw)),
				child.get::<ActiveEvents>(),
				child.get::<ActiveCollisionTypes>(),
			),
		);
		Ok(())
	}

	#[test]
	fn collider_custom_sensor() -> Result<(), RunSystemError> {
		let (mut app, entity) = setup();
		let shape = Shape::Custom {
			model: AssetModel::path("custom"),
			collider: Collider::cuboid(1., 2., 3.),
			scale: Vec3::default(),
		};

		_ = app.world_mut().run_system_once(test_system(move |entity| {
			shape.prefab::<_Interactions, _LifeCycles>(entity, Vec3::ZERO)
		}))?;

		let child = children_of(&app, entity)
			.nth(1)
			.expect("no second entity children");
		assert!(child.contains::<Sensor>());
		Ok(())
	}

	#[test]
	fn collider_custom_root() -> Result<(), RunSystemError> {
		let (mut app, entity) = setup();
		let shape = Shape::Custom {
			model: AssetModel::path("custom"),
			collider: Collider::cuboid(1., 2., 3.),
			scale: Vec3::default(),
		};

		_ = app.world_mut().run_system_once(test_system(move |entity| {
			shape.prefab::<_Interactions, _LifeCycles>(entity, Vec3::ZERO)
		}))?;

		let child = children_of(&app, entity)
			.nth(1)
			.expect("no second entity children");
		assert_eq!(Some(&ColliderRoot(entity)), child.get::<ColliderRoot>());
		Ok(())
	}
}
