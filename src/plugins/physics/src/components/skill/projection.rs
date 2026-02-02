use crate::{
	components::{
		collider::ColliderShape,
		effects::Effects,
		skill::{
			BEAM_MODEL,
			BEAM_PROJECTION_RADIUS,
			HALF_FORWARD,
			PROJECTILE_PROJECTION_RADIUS,
			SHIELD_PROJECTION_RADIUS,
			SHIELD_PROJECTION_TRANSFORM,
			SPHERE_MODEL,
			Skill,
		},
	},
	observers::skill_prefab::{GetProjectionPrefab, ProjectionCollider, SubModel},
};
use bevy::prelude::*;
use common::{
	components::{asset_model::AssetModel, insert_asset::InsertAsset, model::Model},
	tools::Units,
	traits::handles_skill_physics::{
		SkillShape,
		beam::Beam,
		ground_target::SphereAoE,
		shield::Shield,
	},
};
use std::f32::consts::PI;

impl GetProjectionPrefab for Skill {
	fn get_projection_prefab(&self) -> (SubModel, ProjectionCollider, Effects) {
		let (model, collider) = match &self.shape {
			SkillShape::SphereAoE(SphereAoE { radius, .. }) => (
				SubModel {
					model: Model::Asset(AssetModel::path(SPHERE_MODEL)),
					transform: Transform::from_scale(Vec3::splat(**radius * 2.)),
				},
				ProjectionCollider {
					shape: ColliderShape::Sphere {
						radius: *radius,
						hollow_radius: None,
					},
					transform: Transform::default(),
				},
			),
			SkillShape::Projectile(..) => (
				SubModel {
					model: Model::Asset(AssetModel::path(SPHERE_MODEL)),
					transform: Transform::from_scale(Vec3::splat(
						PROJECTILE_PROJECTION_RADIUS * 2.,
					)),
				},
				ProjectionCollider {
					shape: ColliderShape::Sphere {
						radius: Units::from(PROJECTILE_PROJECTION_RADIUS),
						hollow_radius: None,
					},
					transform: Transform::default(),
				},
			),
			SkillShape::Beam(Beam { .. }) => (
				SubModel {
					model: Model::Procedural(InsertAsset::shared::<Beam>(BEAM_MODEL)),
					transform: HALF_FORWARD
						.with_scale(Vec3 {
							x: BEAM_PROJECTION_RADIUS * 2.,
							y: 1.,
							z: BEAM_PROJECTION_RADIUS * 2.,
						})
						.with_rotation(Quat::from_rotation_x(PI / 2.)),
				},
				ProjectionCollider {
					shape: ColliderShape::Cylinder {
						half_y: Units::from(1.),
						radius: Units::from(BEAM_PROJECTION_RADIUS),
					},
					transform: HALF_FORWARD.with_rotation(Quat::from_rotation_x(PI / 2.)),
				},
			),
			SkillShape::Shield(Shield) => (
				SubModel {
					model: Model::Asset(AssetModel::path(SPHERE_MODEL)),
					transform: Transform::from_scale(Vec3::splat(SHIELD_PROJECTION_RADIUS * 2.)),
				},
				ProjectionCollider {
					shape: ColliderShape::Sphere {
						radius: Units::from(SHIELD_PROJECTION_RADIUS),
						hollow_radius: None,
					},
					transform: SHIELD_PROJECTION_TRANSFORM,
				},
			),
		};

		(model, collider, Effects(self.projection_effects.clone()))
	}
}
