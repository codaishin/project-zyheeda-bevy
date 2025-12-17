pub(crate) mod skill_contact;
pub(crate) mod skill_projection;

use crate::components::{
	blockable::Blockable,
	colliders::ColliderShape,
	fix_points::{Always, Anchor, Once},
	ground_target::GroundTarget,
	hollow::Hollow,
	interaction_target::InteractionTarget,
	set_motion_forward::SetMotionForward,
	skill_prefabs::skill_contact::CreatedFrom,
	when_traveled::WhenTraveled,
};
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
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
use std::{f32::consts::PI, fmt::Display, sync::LazyLock};

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
static HOLLOW_OUTER_THICKNESS: LazyLock<Units> = LazyLock::new(|| Units::from(0.3));

impl SkillPrefab for ContactShape {
	type TExtra = Vec3;
	type TError = FaultyColliderShape;

	fn prefab(
		&self,
		entity: &mut impl PrefabEntityCommands,
		offset: Vec3,
	) -> Result<(), FaultyColliderShape> {
		let (interaction, (model, model_transform), (collider, hollow, collider_transform)) =
			match self.clone() {
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
						true => (
							ColliderShape(Shape::Sphere { radius }),
							Some(Hollow {
								radius: Units::from(*radius - **HOLLOW_OUTER_THICKNESS),
							}),
							Transform::default(),
						),
						false => (
							ColliderShape(Shape::Sphere { radius }),
							None,
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
					(ColliderShape(collider), None, Transform::default()),
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
						ColliderShape(Shape::Cylinder {
							half_y: Units::from(0.5),
							radius,
						}),
						None,
						HALF_FORWARD.with_rotation(Quat::from_rotation_x(PI / 2.)),
					),
				),
			};

		entity
			.try_insert_if_new((
				Transform::from_translation(offset),
				Visibility::default(),
				InteractionTarget,
				interaction,
			))
			.with_children(|parent| {
				match model {
					Model::Asset(asset_model) => parent.spawn((asset_model, model_transform)),
					Model::Proc(insert_asset) => parent.spawn((insert_asset, model_transform)),
				};

				let mut child = parent.spawn((
					collider,
					collider_transform,
					ActiveEvents::COLLISION_EVENTS,
					ActiveCollisionTypes::default(),
					Sensor,
				));
				let Some(hollow) = hollow else {
					return;
				};
				child.insert(hollow);
			});

		Ok(())
	}
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
