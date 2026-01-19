use crate::{
	components::{
		collider::ColliderShape,
		effects::Effects,
		skill::{BEAM_MODEL, Beam, HALF_FORWARD, SPHERE_MODEL, Skill},
	},
	observers::skill_prefab::{GetProjectionPrefab, ProjectionCollider, SubModel},
};
use bevy::prelude::*;
use common::{
	components::{asset_model::AssetModel, insert_asset::InsertAsset, model::Model},
	tools::Units,
	traits::{
		handles_physics::physical_bodies::Shape,
		handles_skill_physics::{ProjectionOffset, ProjectionShape},
	},
};
use std::f32::consts::PI;

impl GetProjectionPrefab for Skill {
	fn get_projection_prefab(
		&self,
	) -> (
		Option<ProjectionOffset>,
		SubModel,
		ProjectionCollider,
		Effects,
	) {
		let (model, collider) = match self.projection.shape.clone() {
			ProjectionShape::Sphere { radius } => (
				SubModel {
					model: Model::Asset(AssetModel::path(SPHERE_MODEL)),
					transform: Transform::from_scale(Vec3::splat(*radius * 2.)),
				},
				ProjectionCollider {
					shape: ColliderShape::from(Shape::Sphere { radius }),
					transform: Transform::default(),
				},
			),
			ProjectionShape::Custom {
				model,
				model_scale,
				collider,
			} => (
				SubModel {
					model: Model::Asset(model.clone()),
					transform: Transform::from_scale(model_scale),
				},
				ProjectionCollider {
					shape: ColliderShape::from(collider),
					transform: Transform::default(),
				},
			),
			ProjectionShape::Beam { radius } => (
				SubModel {
					model: Model::Procedural(InsertAsset::shared::<Beam>(BEAM_MODEL)),
					transform: HALF_FORWARD
						.with_scale(Vec3 {
							x: *radius,
							y: 1.,
							z: *radius,
						})
						.with_rotation(Quat::from_rotation_x(PI / 2.)),
				},
				ProjectionCollider {
					shape: ColliderShape::from(Shape::Cylinder {
						half_y: Units::from(0.5),
						radius,
					}),
					transform: HALF_FORWARD.with_rotation(Quat::from_rotation_x(PI / 2.)),
				},
			),
		};

		(
			self.projection.offset,
			model,
			collider,
			Effects(self.projection_effects.clone()),
		)
	}
}
