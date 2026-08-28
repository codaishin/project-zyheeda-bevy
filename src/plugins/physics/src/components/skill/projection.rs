use crate::{
	components::{
		async_collider::ColliderType,
		collider::ColliderShape,
		effects::Effects,
		self_skill_scale::SelfSkillScale,
		skill::{
			BEAM_MODEL,
			BEAM_PROJECTION_RADIUS,
			HALF_FORWARD,
			PROJECTILE_PROJECTION_RADIUS,
			SHIELD_PROJECTION_COLLIDER,
			SHIELD_PROJECTION_MODEL,
			SPHERE_MODEL,
			Skill,
		},
	},
	observers::skill_prefab::{GetProjectionPrefab, ProjectionCollider, SubModel},
};
use bevy::prelude::*;
use bevy_rapier3d::prelude::ColliderScale;
use common::prelude::*;
use std::f32::consts::PI;

impl GetProjectionPrefab for Skill {
	fn get_projection_prefab(
		&self,
		SelfSkillScale(self_skill_scale): &SelfSkillScale,
	) -> (SubModel, ProjectionCollider, Effects) {
		let (model, collider) = match &self.shape {
			SkillShape::SphereAoE(SphereAoE { radius, .. }) => (
				SubModel {
					model: Model::scene(SPHERE_MODEL),
					transform: Transform::from_scale(Vec3::splat(**radius * 2.)),
				},
				ProjectionCollider {
					shape: ColliderShape::Sphere {
						radius: *radius,
						hollow: false,
					},
					transform: Transform::default(),
				},
			),
			SkillShape::Projectile(..) => (
				SubModel {
					model: Model::scene(SPHERE_MODEL),
					transform: Transform::from_scale(Vec3::splat(
						PROJECTILE_PROJECTION_RADIUS * 2.,
					)),
				},
				ProjectionCollider {
					shape: ColliderShape::Sphere {
						radius: Units::from(PROJECTILE_PROJECTION_RADIUS),
						hollow: false,
					},
					transform: Transform::default(),
				},
			),
			SkillShape::Beam(Beam { .. }) => (
				SubModel {
					model: Model::Mesh(InsertAsset::shared::<Beam>(BEAM_MODEL)),
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
					model: Model::scene(SHIELD_PROJECTION_MODEL),
					transform: Transform::from_scale(Vec3::from(*self_skill_scale)),
				},
				ProjectionCollider {
					shape: ColliderShape::CustomAsset {
						mesh: SHIELD_PROJECTION_COLLIDER,
						scale: ColliderScale::Relative(Vec3::from(*self_skill_scale)),
						collider_type: ColliderType::Convex,
					},
					transform: Transform::default(),
				},
			),
		};

		(model, collider, Effects(self.projection_effects.clone()))
	}
}
