use crate::{
	components::{
		collider::ColliderShape,
		effects::Effects,
		skill::{BEAM_MODEL, Beam, HALF_FORWARD, HOLLOW_OUTER_THICKNESS, SPHERE_MODEL, Skill},
	},
	observers::skill_prefab::{ContactCollider, GetContactPrefab, SubModel},
};
use bevy::prelude::*;
use common::{
	components::{asset_model::AssetModel, insert_asset::InsertAsset, model::Model},
	tools::Units,
	traits::{
		handles_physics::{PhysicalObject, physical_bodies::Shape},
		handles_skill_physics::ContactShape,
	},
};
use std::f32::consts::PI;

impl GetContactPrefab for Skill {
	fn get_contact_prefab(&self) -> (PhysicalObject, SubModel, ContactCollider, Effects) {
		let (blockable, model, collider) = match self.contact.shape.clone() {
			ContactShape::Sphere {
				radius,
				hollow_collider,
				destroyed_by,
			} => (
				PhysicalObject::Fragile { destroyed_by },
				SubModel {
					model: Model::Asset(AssetModel::path(SPHERE_MODEL)),
					transform: Transform::from_scale(Vec3::splat(*radius * 2.)),
				},
				ContactCollider {
					shape: ColliderShape::Sphere {
						radius,
						hollow_radius: match hollow_collider {
							true => Some(Units::from(*radius - **HOLLOW_OUTER_THICKNESS)),
							false => None,
						},
					},
					transform: Transform::default(),
				},
			),
			ContactShape::Custom {
				model,
				collider,
				model_scale,
				destroyed_by,
			} => (
				PhysicalObject::Fragile { destroyed_by },
				SubModel {
					model: Model::Asset(model),
					transform: Transform::from_scale(model_scale),
				},
				ContactCollider {
					shape: ColliderShape::from(collider),
					transform: Transform::default(),
				},
			),
			ContactShape::Beam {
				range,
				blocked_by,
				radius,
			} => (
				PhysicalObject::Beam { range, blocked_by },
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
				ContactCollider {
					shape: ColliderShape::from(Shape::Cylinder {
						half_y: Units::from(0.5),
						radius,
					}),
					transform: HALF_FORWARD.with_rotation(Quat::from_rotation_x(PI / 2.)),
				},
			),
		};

		(
			blockable,
			model,
			collider,
			Effects(self.contact_effects.clone()),
		)
	}
}
