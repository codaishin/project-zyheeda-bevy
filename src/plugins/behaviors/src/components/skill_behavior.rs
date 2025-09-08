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
	components::{
		asset_model::AssetModel,
		collider_relationship::InteractionTarget,
		insert_asset::InsertAsset,
	},
	errors::{Error, Level},
	traits::{
		handles_physics::{HandlesPhysics, PhysicalObject},
		handles_skill_behaviors::{ContactShape, Motion, ProjectionShape},
		prefab::PrefabEntityCommands,
	},
};
use std::f32::consts::PI;

trait SimplePrefab {
	type TExtra;

	fn prefab<TPhysics>(
		&self,
		entity: &mut impl PrefabEntityCommands,
		extra: Self::TExtra,
	) -> Result<(), Error>
	where
		TPhysics: HandlesPhysics;
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

impl SimplePrefab for ContactShape {
	type TExtra = Vec3;

	fn prefab<TPhysics>(
		&self,
		entity: &mut impl PrefabEntityCommands,
		offset: Vec3,
	) -> Result<(), Error>
	where
		TPhysics: HandlesPhysics,
	{
		let (interaction, (model, model_transform), (collider, collider_transform)) = match self {
			Self::Sphere {
				radius,
				hollow_collider,
				destroyed_by,
			} => (
				TPhysics::TPhysicalObjectComponent::from(PhysicalObject::Fragile {
					destroyed_by: destroyed_by.clone(),
				}),
				(
					Model::Asset(AssetModel::path(SPHERE_MODEL)),
					Transform::from_scale(Vec3::splat(**radius * 2.)),
				),
				match hollow_collider {
					true => ring_collider(**radius)?,
					false => sphere_collider(**radius),
				},
			),
			Self::Custom {
				model,
				collider,
				scale,
				destroyed_by,
			} => (
				TPhysics::TPhysicalObjectComponent::from(PhysicalObject::Fragile {
					destroyed_by: destroyed_by.clone(),
				}),
				(Model::Asset(model.clone()), Transform::from_scale(*scale)),
				custom_collider(collider, *scale),
			),
			Self::Beam {
				range,
				blocked_by,
				radius,
			} => (
				TPhysics::TPhysicalObjectComponent::from(PhysicalObject::Beam {
					range: *range,
					blocked_by: blocked_by.clone(),
				}),
				(
					Model::Proc(InsertAsset::shared::<Beam>(BEAM_MODEL)),
					HALF_FORWARD
						.with_scale(Vec3 {
							x: **radius,
							y: 1.,
							z: **radius,
						})
						.with_rotation(Quat::from_rotation_x(PI / 2.)),
				),
				(
					Collider::cylinder(0.5, **radius),
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

impl SimplePrefab for ProjectionShape {
	type TExtra = Vec3;

	fn prefab<TPhysics>(
		&self,
		entity: &mut impl PrefabEntityCommands,
		offset: Vec3,
	) -> Result<(), Error>
	where
		TPhysics: HandlesPhysics,
	{
		let ((model, model_transform), (collider, collider_transform)) = match self {
			Self::Sphere { radius } => (
				(
					Model::Asset(AssetModel::path(SPHERE_MODEL)),
					Transform::from_scale(Vec3::splat(**radius * 2.)),
				),
				sphere_collider(**radius),
			),
			Self::Custom {
				model,
				collider,
				scale,
			} => (
				(Model::Asset(model.clone()), Transform::from_scale(*scale)),
				custom_collider(collider, *scale),
			),
			Self::Beam { radius } => (
				(
					Model::Proc(InsertAsset::shared::<Beam>(BEAM_MODEL)),
					HALF_FORWARD
						.with_scale(Vec3 {
							x: **radius,
							y: 1.,
							z: **radius,
						})
						.with_rotation(Quat::from_rotation_x(PI / 2.)),
				),
				(
					Collider::cylinder(0.5, **radius),
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

impl SimplePrefab for Motion {
	type TExtra = CreatedFrom;

	fn prefab<TPhysics>(
		&self,
		entity: &mut impl PrefabEntityCommands,
		created_from: CreatedFrom,
	) -> Result<(), Error>
	where
		TPhysics: HandlesPhysics,
	{
		match self.clone() {
			Motion::HeldBy { caster, spawner } => {
				entity.try_insert_if_new((
					RigidBody::Fixed,
					Anchor::<Always>::to_target(caster)
						.on_fix_point(FixPoint(spawner))
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
