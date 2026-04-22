use crate::{
	components::{
		async_collider::ColliderType,
		collider::ColliderShape,
		effects::Effects,
		skill::{
			BEAM_CONTACT_RADIUS,
			BEAM_MODEL,
			HALF_FORWARD,
			PROJECTILE_CONTACT_RADIUS,
			SHIELD_CONTACT_COLLIDER,
			SHIELD_MODEL,
			SHIELD_SCALE,
			SPHERE_MODEL,
			Skill,
		},
	},
	observers::skill_prefab::{ContactCollider, GetContactPrefab, SubModel},
};
use bevy::prelude::*;
use bevy_rapier3d::prelude::ColliderScale;
use common::{
	components::{asset_model::AssetModel, insert_asset::InsertAsset, model::Model},
	tools::Units,
	traits::{
		handles_physics::{PhysicalObject, physical_bodies::Blocker},
		handles_skill_physics::{
			SkillShape,
			beam::Beam,
			ground_target::SphereAoE,
			projectile::Projectile,
			shield::Shield,
		},
	},
};
use std::f32::consts::PI;

impl GetContactPrefab for Skill {
	fn get_contact_prefab(&self) -> (PhysicalObject, SubModel, ContactCollider, Effects) {
		let (obj, model, collider) = match &self.shape {
			SkillShape::SphereAoE(SphereAoE { radius, .. }) => (
				PhysicalObject::Fragile {
					destroyed_by: Blocker::none(),
				},
				SubModel {
					model: Model::Asset(AssetModel::path(SPHERE_MODEL)),
					transform: Transform::from_scale(Vec3::splat(**radius * 2.)),
				},
				ContactCollider {
					shape: ColliderShape::Sphere {
						radius: *radius,
						hollow: true,
					},
					transform: Transform::default(),
				},
			),
			SkillShape::Projectile(Projectile { destroyed_by }) => (
				PhysicalObject::Fragile {
					destroyed_by: destroyed_by.clone().into(),
				},
				SubModel {
					model: Model::Asset(AssetModel::path(SPHERE_MODEL)),
					transform: Transform::from_scale(Vec3::splat(PROJECTILE_CONTACT_RADIUS * 2.)),
				},
				ContactCollider {
					shape: ColliderShape::Sphere {
						radius: Units::from(PROJECTILE_CONTACT_RADIUS),
						hollow: false,
					},
					transform: Transform::default(),
				},
			),
			SkillShape::Beam(Beam { range, blocked_by }) => (
				PhysicalObject::Beam {
					range: *range,
					blocked_by: blocked_by.clone().into(),
				},
				SubModel {
					model: Model::Procedural(InsertAsset::shared::<Beam>(BEAM_MODEL)),
					transform: HALF_FORWARD
						.with_scale(Vec3 {
							x: BEAM_CONTACT_RADIUS * 2.,
							y: 1.,
							z: BEAM_CONTACT_RADIUS * 2.,
						})
						.with_rotation(Quat::from_rotation_x(PI / 2.)),
				},
				ContactCollider {
					shape: ColliderShape::Cylinder {
						half_y: Units::from(1.),
						radius: Units::from(BEAM_CONTACT_RADIUS),
					},
					transform: HALF_FORWARD.with_rotation(Quat::from_rotation_x(PI / 2.)),
				},
			),
			SkillShape::Shield(Shield) => (
				PhysicalObject::Fragile {
					destroyed_by: Blocker::none(),
				},
				SubModel {
					model: Model::Asset(AssetModel::path(SHIELD_MODEL)),
					transform: Transform::from_scale(SHIELD_SCALE),
				},
				ContactCollider {
					shape: ColliderShape::CustomAsset {
						mesh: SHIELD_CONTACT_COLLIDER,
						scale: ColliderScale::Relative(SHIELD_SCALE),
						collider_type: ColliderType::Concave,
					},
					transform: Transform::default(),
				},
			),
		};

		(obj, model, collider, Effects(self.contact_effects.clone()))
	}
}
