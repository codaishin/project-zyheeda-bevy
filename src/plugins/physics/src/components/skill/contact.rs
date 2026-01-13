use crate::components::{
	blockable::Blockable,
	colliders::ColliderShape,
	hollow::Hollow,
	interaction_target::InteractionTarget,
	skill::{
		BEAM_MODEL,
		Beam,
		HALF_FORWARD,
		HOLLOW_OUTER_THICKNESS,
		Model,
		SPHERE_MODEL,
		Skill,
		SkillContact,
		insert_effect,
	},
};
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use common::{
	components::{asset_model::AssetModel, insert_asset::InsertAsset},
	tools::Units,
	traits::{
		handles_physics::{PhysicalObject, colliders::Shape},
		handles_skill_physics::ContactShape,
	},
	zyheeda_commands::ZyheedaEntityCommands,
};
use std::f32::consts::PI;

impl Skill {
	pub(crate) fn contact(&self, entity: &mut ZyheedaEntityCommands) {
		let (interaction, (model, model_transform), (collider, hollow, collider_transform)) =
			match self.contact.shape.clone() {
				ContactShape::Sphere {
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
				ContactShape::Custom {
					model,
					collider,
					model_scale,
					destroyed_by,
				} => (
					Blockable(PhysicalObject::Fragile { destroyed_by }),
					(Model::Asset(model), Transform::from_scale(model_scale)),
					(ColliderShape(collider), None, Transform::default()),
				),
				ContactShape::Beam {
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
				SkillContact,
				Transform::default(),
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

		for effect in &self.contact_effects {
			insert_effect(entity, *effect);
		}
	}
}
