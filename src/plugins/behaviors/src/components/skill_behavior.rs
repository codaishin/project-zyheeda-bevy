pub mod skill_contact;
pub mod skill_projection;

use super::{
	Always,
	Once,
	fix_points::Anchor,
	ground_target::GroundTarget,
	set_to_move_forward::SetVelocityForward,
	when_traveled_insert::WhenTraveled,
};
use crate::components::{
	fix_points::fix_point::FixPoint,
	skill_behavior::skill_contact::CreatedFrom,
};
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use common::{
	components::{asset_model::AssetModel, collider_relationship::InteractionTarget},
	errors::{Error, Level},
	traits::{
		handles_interactions::{HandlesInteractions, InteractAble},
		handles_skill_behaviors::{Integrity, Motion, Shape, SkillSpawner},
		prefab::PrefabEntityCommands,
	},
};
use std::f32::consts::PI;

trait SimplePrefab {
	type TExtra;

	fn prefab<TInteractions>(
		&self,
		entity: &mut impl PrefabEntityCommands,
		extra: Self::TExtra,
	) -> Result<(), Error>
	where
		TInteractions: HandlesInteractions;
}

const SPHERE_MODEL: &str = "models/sphere.glb";

impl SimplePrefab for Shape {
	type TExtra = Vec3;

	fn prefab<TInteractions>(
		&self,
		entity: &mut impl PrefabEntityCommands,
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
			.try_insert_if_new((
				Transform::from_translation(offset),
				Visibility::default(),
				InteractionTarget,
			))
			.with_child((model, model_transform))
			.with_child((
				collider,
				collider_transform,
				ActiveEvents::COLLISION_EVENTS,
				ActiveCollisionTypes::default(),
				Sensor,
			));

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
		return Err(Error::Single {
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

	fn prefab<TInteractions>(
		&self,
		entity: &mut impl PrefabEntityCommands,
		_: (),
	) -> Result<(), Error>
	where
		TInteractions: HandlesInteractions,
	{
		match self {
			Integrity::Solid => {}
			Integrity::Fragile { destroyed_by } => {
				entity.try_insert_if_new(TInteractions::TInteraction::from(
					InteractAble::Fragile {
						destroyed_by: destroyed_by.clone(),
					},
				));
			}
		};

		Ok(())
	}
}

impl SimplePrefab for Motion {
	type TExtra = CreatedFrom;

	fn prefab<TInteractions>(
		&self,
		entity: &mut impl PrefabEntityCommands,
		created_from: CreatedFrom,
	) -> Result<(), Error> {
		match *self {
			Motion::HeldBy { caster } => {
				entity.try_insert_if_new((
					RigidBody::Fixed,
					Anchor::<Always>::to_target(caster)
						.on_fix_point(FixPoint(SkillSpawner::Center))
						.with_target_rotation(),
				));
			}
			Motion::Stationary {
				caster,
				max_cast_range,
				target_ray,
			} => {
				entity.try_insert_if_new((
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
				range,
			} => {
				entity.try_insert_if_new((
					RigidBody::Dynamic,
					GravityScale(0.),
					Ccd::enabled(),
					WhenTraveled::via::<Velocity>().distance(range).destroy(),
				));

				if created_from == CreatedFrom::Save {
					return Ok(());
				}

				entity.try_insert_if_new((
					Anchor::<Once>::to_target(caster)
						.on_fix_point(FixPoint(spawner))
						.with_target_rotation(),
					SetVelocityForward(speed),
				));
			}
		}
		Ok(())
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::when_traveled_insert::DestroyAfterDistanceTraveled;
	use bevy::ecs::system::{RunSystemError, RunSystemOnce};
	use bevy_rapier3d::prelude::ActiveCollisionTypes;
	use common::{
		components::{is_blocker::Blocker, persistent_entity::PersistentEntity},
		tools::{
			Units,
			UnitsPerSecond,
			action_key::slot::{Side, SlotKey},
		},
		traits::{
			clamp_zero_positive::ClampZeroPositive,
			handles_interactions::{HandlesInteractions, InteractAble},
		},
	};
	use std::collections::HashSet;

	struct _Interactions;

	#[derive(SystemSet, Debug, PartialEq, Eq, Hash, Clone)]
	struct _Systems;

	#[derive(Component, Debug, PartialEq)]
	enum _Interaction {
		Beam,
		Fragile(HashSet<Blocker>),
	}

	impl From<InteractAble> for _Interaction {
		fn from(interaction: InteractAble) -> Self {
			match interaction {
				InteractAble::Beam { .. } => Self::Beam,
				InteractAble::Fragile { destroyed_by } => Self::Fragile(destroyed_by),
			}
		}
	}

	impl HandlesInteractions for _Interactions {
		type TSystems = _Systems;
		type TInteraction = _Interaction;

		const SYSTEMS: Self::TSystems = _Systems;
	}

	fn test_system<T>(
		exec: impl Fn(&mut EntityCommands) -> T,
	) -> impl Fn(Commands, Query<Entity>) -> T {
		move |mut commands, query| {
			let entity = query
				.single()
				.expect("U FOOL, AN ENTITY CANNOT BE FOUND HERE");
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
		let caster = PersistentEntity::default();
		let motion = Motion::HeldBy { caster };

		_ = app.world_mut().run_system_once(test_system(move |entity| {
			motion.prefab::<_Interactions>(entity, CreatedFrom::Contact)
		}))?;

		let entity = app.world().entity(entity);
		assert_eq!(
			(
				Some(&RigidBody::Fixed),
				None,
				None,
				None,
				Some(
					&Anchor::<Always>::to_target(caster)
						.on_fix_point(FixPoint(SkillSpawner::Center))
						.with_target_rotation()
				),
				None,
				None,
				None,
				None,
			),
			(
				entity.get::<RigidBody>(),
				entity.get::<GravityScale>(),
				entity.get::<Ccd>(),
				entity.get::<GroundTarget>(),
				entity.get::<Anchor<Always>>(),
				entity.get::<Anchor<Once>>(),
				entity.get::<SetVelocityForward>(),
				entity.get::<DestroyAfterDistanceTraveled<Velocity>>(),
				entity.get::<_Interaction>(),
			)
		);
		Ok(())
	}

	#[test]
	fn stationary_components() -> Result<(), RunSystemError> {
		let (mut app, entity) = setup();
		let caster = PersistentEntity::default();
		let motion = Motion::Stationary {
			caster,
			max_cast_range: Units::new(42.),
			target_ray: Ray3d {
				origin: Vec3::new(11., 12., 13.),
				direction: Dir3::from_xyz(2., 1., 4.).unwrap(),
			},
		};

		_ = app.world_mut().run_system_once(test_system(move |entity| {
			motion.prefab::<_Interactions>(entity, CreatedFrom::Contact)
		}))?;

		assert_eq!(
			(
				Some(&RigidBody::Fixed),
				None,
				None,
				Some(&GroundTarget {
					caster,
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
				app.world().entity(entity).get::<Anchor<Always>>(),
				app.world().entity(entity).get::<Anchor<Once>>(),
				app.world().entity(entity).get::<SetVelocityForward>(),
				app.world()
					.entity(entity)
					.get::<DestroyAfterDistanceTraveled<Velocity>>(),
			)
		);
		Ok(())
	}

	#[test]
	fn projectile_components_when_created_from_contact() -> Result<(), RunSystemError> {
		let (mut app, entity) = setup();
		let caster = PersistentEntity::default();
		let motion = Motion::Projectile {
			caster,
			spawner: SkillSpawner::Slot(SlotKey::TopHand(Side::Left)),
			speed: UnitsPerSecond::new(11.),
			range: Units::new(1111.),
		};

		_ = app.world_mut().run_system_once(test_system(move |entity| {
			motion.prefab::<_Interactions>(entity, CreatedFrom::Contact)
		}))?;

		let entity = app.world().entity(entity);
		assert_eq!(
			(
				Some(&RigidBody::Dynamic),
				Some(&GravityScale(0.)),
				Some(&Ccd::enabled()),
				None,
				None,
				Some(
					&Anchor::<Once>::to_target(caster)
						.on_fix_point(FixPoint(SkillSpawner::Slot(SlotKey::TopHand(Side::Left))))
						.with_target_rotation()
				),
				Some(&SetVelocityForward(UnitsPerSecond::new(11.))),
				Some(
					&WhenTraveled::via::<Velocity>()
						.distance(Units::new(1111.))
						.destroy()
				),
			),
			(
				entity.get::<RigidBody>(),
				entity.get::<GravityScale>(),
				entity.get::<Ccd>(),
				entity.get::<GroundTarget>(),
				entity.get::<Anchor<Always>>(),
				entity.get::<Anchor<Once>>(),
				entity.get::<SetVelocityForward>(),
				entity.get::<DestroyAfterDistanceTraveled<Velocity>>(),
			)
		);
		Ok(())
	}

	#[test]
	fn projectile_components_when_created_from_save() -> Result<(), RunSystemError> {
		let (mut app, entity) = setup();
		let caster = PersistentEntity::default();
		let motion = Motion::Projectile {
			caster,
			spawner: SkillSpawner::Slot(SlotKey::TopHand(Side::Left)),
			speed: UnitsPerSecond::new(11.),
			range: Units::new(1111.),
		};

		_ = app.world_mut().run_system_once(test_system(move |entity| {
			motion.prefab::<_Interactions>(entity, CreatedFrom::Save)
		}))?;

		assert_eq!(
			(
				Some(&RigidBody::Dynamic),
				Some(&GravityScale(0.)),
				Some(&Ccd::enabled()),
				None,
				None,
				None,
				None,
				Some(
					&WhenTraveled::via::<Velocity>()
						.distance(Units::new(1111.))
						.destroy()
				),
			),
			(
				app.world().entity(entity).get::<RigidBody>(),
				app.world().entity(entity).get::<GravityScale>(),
				app.world().entity(entity).get::<Ccd>(),
				app.world().entity(entity).get::<GroundTarget>(),
				app.world().entity(entity).get::<Anchor<Always>>(),
				app.world().entity(entity).get::<Anchor<Once>>(),
				app.world().entity(entity).get::<SetVelocityForward>(),
				app.world()
					.entity(entity)
					.get::<DestroyAfterDistanceTraveled<Velocity>>(),
			)
		);
		Ok(())
	}

	#[test]
	fn fragile_components() -> Result<(), RunSystemError> {
		let (mut app, entity) = setup();
		let integrity = Integrity::Fragile {
			destroyed_by: HashSet::from([Blocker::Physical]),
		};

		_ = app.world_mut().run_system_once(test_system(move |entity| {
			integrity.prefab::<_Interactions>(entity, ())
		}))?;

		assert_eq!(
			Some(&_Interaction::Fragile([Blocker::Physical].into())),
			app.world().entity(entity).get::<_Interaction>(),
		);
		Ok(())
	}

	fn children_of(app: &App, entity: Entity) -> impl Iterator<Item = EntityRef<'_>> {
		app.world().iter_entities().filter(move |e| {
			e.get::<ChildOf>()
				.map(|c| c.parent() == entity)
				.unwrap_or(false)
		})
	}

	#[test]
	fn collider_root_transform_and_visibility_sphere() -> Result<(), RunSystemError> {
		let (mut app, entity) = setup();
		let shape = Shape::Sphere {
			radius: Units::new(42.),
			hollow_collider: false,
		};

		_ = app.world_mut().run_system_once(test_system(move |entity| {
			shape.prefab::<_Interactions>(entity, Vec3::new(1., 2., 3.))
		}))?;

		assert_eq!(
			(
				Some(&InteractionTarget),
				Some(&Transform::from_xyz(1., 2., 3.)),
				Some(&Visibility::Inherited)
			),
			(
				app.world().entity(entity).get::<InteractionTarget>(),
				app.world().entity(entity).get::<Transform>(),
				app.world().entity(entity).get::<Visibility>(),
			)
		);
		Ok(())
	}

	#[test]
	fn collider_root_transform_and_visibility_custom_shape() -> Result<(), RunSystemError> {
		let (mut app, entity) = setup();
		let shape = Shape::Custom {
			model: AssetModel::path(""),
			collider: Collider::default(),
			scale: Vec3::new(1., 2., 3.),
		};

		_ = app.world_mut().run_system_once(test_system(move |entity| {
			shape.prefab::<_Interactions>(entity, Vec3::new(1., 2., 3.))
		}))?;

		assert_eq!(
			(
				Some(&InteractionTarget),
				Some(&Transform::from_xyz(1., 2., 3.)),
				Some(&Visibility::Inherited)
			),
			(
				app.world().entity(entity).get::<InteractionTarget>(),
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
			shape.prefab::<_Interactions>(entity, Vec3::ZERO)
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
			shape.prefab::<_Interactions>(entity, Vec3::ZERO)
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
			shape.prefab::<_Interactions>(entity, Vec3::ZERO)
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
			shape.prefab::<_Interactions>(entity, Vec3::ZERO)
		}))?;

		let child = children_of(&app, entity)
			.nth(1)
			.expect("no second entity children");
		assert!(child.contains::<Sensor>());
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
			shape.prefab::<_Interactions>(entity, Vec3::ZERO)
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
			shape.prefab::<_Interactions>(entity, Vec3::ZERO)
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
			shape.prefab::<_Interactions>(entity, Vec3::ZERO)
		}))?;

		let child = children_of(&app, entity)
			.nth(1)
			.expect("no second entity children");
		assert!(child.contains::<Sensor>());
		Ok(())
	}
}
