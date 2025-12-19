use crate::components::{
	colliders::ColliderShape,
	interaction_target::InteractionTarget,
	skill::{
		BEAM_MODEL,
		Beam,
		HALF_FORWARD,
		Model,
		SPHERE_MODEL,
		Skill,
		SkillProjection,
		insert_effect,
	},
	skill_transform::SkillTransformOf,
};
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use common::{
	components::{asset_model::AssetModel, insert_asset::InsertAsset},
	tools::Units,
	traits::{
		handles_physics::colliders::Shape,
		handles_skill_physics::{ProjectionOffset, ProjectionShape},
	},
	zyheeda_commands::ZyheedaEntityCommands,
};
use std::f32::consts::PI;

impl Skill {
	pub(crate) fn projection(&self, entity: &mut ZyheedaEntityCommands, root: Entity) {
		let ((model, model_transform), (collider, collider_transform)) =
			match self.projection.shape.clone() {
				ProjectionShape::Sphere { radius } => (
					(
						Model::Asset(AssetModel::path(SPHERE_MODEL)),
						Transform::from_scale(Vec3::splat(*radius * 2.)),
					),
					(
						ColliderShape(Shape::Sphere { radius }),
						Transform::default(),
					),
				),
				ProjectionShape::Custom {
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
				ProjectionShape::Beam { radius } => (
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

		let offset = self
			.projection
			.offset
			.map(|ProjectionOffset(offset)| offset)
			.unwrap_or_default();

		entity.try_insert_if_new((
			SkillProjection,
			Transform::from_translation(offset),
			Visibility::default(),
			InteractionTarget,
		));

		match model {
			Model::Asset(asset_model) => {
				entity.with_child((SkillTransformOf(root), asset_model, model_transform))
			}
			Model::Proc(insert_asset) => {
				entity.with_child((SkillTransformOf(root), insert_asset, model_transform))
			}
		};

		entity.with_child((
			SkillTransformOf(root),
			collider,
			collider_transform,
			ActiveEvents::COLLISION_EVENTS,
			ActiveCollisionTypes::default(),
			Sensor,
		));

		for effect in &self.projection_effects {
			insert_effect(entity, *effect);
		}
	}
}
