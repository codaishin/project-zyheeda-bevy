pub(crate) mod skill_contact;
pub(crate) mod skill_projection;

use crate::components::{
	blockable::Blockable,
	colliders::ColliderShape,
	fix_points::{Always, Anchor, Once},
	ground_target::GroundTarget,
	interaction_target::InteractionTarget,
	set_motion_forward::SetMotionForward,
	skill_prefabs::skill_contact::CreatedFrom,
	when_traveled::WhenTraveled,
};
use bevy::prelude::*;
use bevy_rapier3d::prelude::{Collider as RapierCollider, *};
use common::{
	components::{asset_model::AssetModel, insert_asset::InsertAsset},
	errors::{ErrorData, Level, Unreachable},
	tools::Units,
	traits::{
		handles_physics::{PhysicalObject, colliders::Shape},
		handles_skill_behaviors::{ContactShape, Motion, ProjectionShape},
		prefab::PrefabEntityCommands,
	},
};
use std::{f32::consts::PI, fmt::Display};

trait SkillPrefab {
	type TExtra;
	type TError;

	fn prefab(
		&self,
		entity: &mut impl PrefabEntityCommands,
		extra: Self::TExtra,
	) -> Result<(), Self::TError>;
}

const SPHERE_MODEL: &str = "models/sphere.glb";
const BEAM_MODEL: fn() -> Mesh = || {
	Mesh::from(Cylinder {
		radius: 1.,
		half_height: 0.5,
	})
};
const HALF_FORWARD: Transform = Transform::from_translation(Vec3 {
	x: 0.,
	y: 0.,
	z: -0.5,
});

impl SkillPrefab for ContactShape {
	type TExtra = Vec3;
	type TError = FaultyColliderShape;

	fn prefab(
		&self,
		entity: &mut impl PrefabEntityCommands,
		offset: Vec3,
	) -> Result<(), FaultyColliderShape> {
		let (interaction, (model, model_transform), (collider, collider_transform)) = match self
			.clone()
		{
			Self::Sphere {
				radius,
				hollow_collider,
				destroyed_by,
			} => (
				Blockable(PhysicalObject::Fragile { destroyed_by }),
				(
					Model::Asset(AssetModel::path(SPHERE_MODEL)),
					Transform::from_scale(Vec3::splat(*radius * 2.)),
				),
				match hollow_collider {
					true => ring_collider(*radius).map(|(c, t)| (ColliderVariant::Old(c), t))?,
					false => (
						ColliderVariant::New(ColliderShape(Shape::Sphere { radius })),
						Transform::default(),
					),
				},
			),
			Self::Custom {
				model,
				collider,
				model_scale,
				destroyed_by,
			} => (
				Blockable(PhysicalObject::Fragile { destroyed_by }),
				(Model::Asset(model), Transform::from_scale(model_scale)),
				(
					ColliderVariant::New(ColliderShape(collider)),
					Transform::default(),
				),
			),
			Self::Beam {
				range,
				blocked_by,
				radius,
			} => (
				Blockable(PhysicalObject::Beam { range, blocked_by }),
				(
					Model::Proc(InsertAsset::shared::<Beam>(BEAM_MODEL)),
					HALF_FORWARD
						.with_scale(Vec3 {
							x: *radius,
							y: 1.,
							z: *radius,
						})
						.with_rotation(Quat::from_rotation_x(PI / 2.)),
				),
				(
					ColliderVariant::New(ColliderShape(Shape::Cylinder {
						half_y: Units::from(0.5),
						radius,
					})),
					HALF_FORWARD.with_rotation(Quat::from_rotation_x(PI / 2.)),
				),
			),
		};

		entity.try_insert_if_new((
			Transform::from_translation(offset),
			Visibility::default(),
			InteractionTarget,
			interaction,
		));

		match model {
			Model::Asset(asset_model) => entity.with_child((asset_model, model_transform)),
			Model::Proc(insert_asset) => entity.with_child((insert_asset, model_transform)),
		};

		match collider {
			ColliderVariant::Old(collider) => entity.with_child((
				collider,
				collider_transform,
				ActiveEvents::COLLISION_EVENTS,
				ActiveCollisionTypes::default(),
				Sensor,
			)),
			ColliderVariant::New(collider_shape) => entity.with_child((
				collider_shape,
				collider_transform,
				ActiveEvents::COLLISION_EVENTS,
				ActiveCollisionTypes::default(),
				Sensor,
			)),
		};

		Ok(())
	}
}

// FIXME: Remove when using bevy rapier physics hooks to create better hollow spherical colliders.
//        Hollow colliders just need to work with spherical or capsule colliders inside of
//        hollow (half)spherical colliders (shield domes).
/// This enum is a temporary measure to allow creation of a fake hollow sphere collider.
enum ColliderVariant {
	/// Data needed for current hollow spherical colliders
	Old(RapierCollider),
	/// Newer approach using a common collider definition
	New(ColliderShape),
}

impl SkillPrefab for ProjectionShape {
	type TExtra = Vec3;
	type TError = FaultyColliderShape;

	fn prefab(
		&self,
		entity: &mut impl PrefabEntityCommands,
		offset: Vec3,
	) -> Result<(), FaultyColliderShape> {
		let ((model, model_transform), (collider, collider_transform)) = match self.clone() {
			Self::Sphere { radius } => (
				(
					Model::Asset(AssetModel::path(SPHERE_MODEL)),
					Transform::from_scale(Vec3::splat(*radius * 2.)),
				),
				(
					ColliderShape(Shape::Sphere { radius }),
					Transform::default(),
				),
			),
			Self::Custom {
				model,
				model_scale,
				collider,
			} => (
				(
					Model::Asset(model.clone()),
					Transform::from_scale(model_scale),
				),
				(ColliderShape(collider), Transform::default()),
			),
			Self::Beam { radius } => (
				(
					Model::Proc(InsertAsset::shared::<Beam>(BEAM_MODEL)),
					HALF_FORWARD
						.with_scale(Vec3 {
							x: *radius,
							y: 1.,
							z: *radius,
						})
						.with_rotation(Quat::from_rotation_x(PI / 2.)),
				),
				(
					ColliderShape(Shape::Cylinder {
						half_y: Units::from(0.5),
						radius,
					}),
					HALF_FORWARD.with_rotation(Quat::from_rotation_x(PI / 2.)),
				),
			),
		};

		entity.try_insert_if_new((
			Transform::from_translation(offset),
			Visibility::default(),
			InteractionTarget,
		));

		match model {
			Model::Asset(asset_model) => entity.with_child((asset_model, model_transform)),
			Model::Proc(insert_asset) => entity.with_child((insert_asset, model_transform)),
		};

		entity.with_child((
			collider,
			collider_transform,
			ActiveEvents::COLLISION_EVENTS,
			ActiveCollisionTypes::default(),
			Sensor,
		));

		Ok(())
	}
}

struct Beam;

enum Model {
	Asset(AssetModel),
	Proc(InsertAsset<Mesh>),
}

impl SkillPrefab for Motion {
	type TExtra = CreatedFrom;
	type TError = Unreachable;

	fn prefab(
		&self,
		entity: &mut impl PrefabEntityCommands,
		created_from: CreatedFrom,
	) -> Result<(), Unreachable> {
		match *self {
			Motion::HeldBy { caster, spawner } => {
				entity.try_insert_if_new((
					RigidBody::Fixed,
					Anchor::<Always>::to_target(caster.0)
						.on_spawner(spawner)
						.with_target_rotation(),
				));
			}
			Motion::Stationary {
				caster,
				max_cast_range,
				target,
			} => {
				entity.try_insert_if_new((
					RigidBody::Fixed,
					GroundTarget {
						caster,
						max_cast_range,
						target,
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
					WhenTraveled::distance(range).destroy(),
				));

				if created_from == CreatedFrom::Save {
					return Ok(());
				}

				entity.try_insert_if_new((
					Anchor::<Once>::to_target(caster.0)
						.on_spawner(spawner)
						.with_target_rotation(),
					SetMotionForward(speed),
				));
			}
		}
		Ok(())
	}
}

fn ring_collider(radius: f32) -> Result<(RapierCollider, Transform), FaultyColliderShape> {
	let transform = Transform::default().with_rotation(Quat::from_axis_angle(Vec3::X, PI / 2.));
	let ring = Annulus::new(radius * 0.9, radius);
	let torus = Mesh::from(Extrusion::new(ring, radius * 2.));
	let shape = ComputedColliderShape::TriMesh(TriMeshFlags::MERGE_DUPLICATE_VERTICES);
	let collider = RapierCollider::from_bevy_mesh(&torus, &shape);

	let Some(collider) = collider else {
		return Err(FaultyColliderShape { shape });
	};

	Ok((collider, transform))
}

pub struct FaultyColliderShape {
	shape: ComputedColliderShape,
}

impl Display for FaultyColliderShape {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "Faulty collider shape ({:?})", self.shape)
	}
}

impl ErrorData for FaultyColliderShape {
	fn level(&self) -> Level {
		Level::Error
	}

	fn label() -> impl Display {
		"Construction error"
	}

	fn into_details(self) -> impl Display {
		self
	}
}
