use crate::{components::hollow::Hollow, physics_hooks::check_hollow_colliders::SimpleOuterRadius};
use bevy::{ecs::entity::EntityHashSet, prelude::*};
use bevy_rapier3d::prelude::Collider as RapierCollider;
use common::{
	errors::Unreachable,
	tools::Units,
	traits::{
		handles_physics::colliders::{Collider, Shape},
		load_asset::LoadAsset,
		prefab::{Prefab, PrefabEntityCommands},
	},
};

#[derive(Component, Default)]
#[relationship_target(relationship = ColliderOf)]
pub(crate) struct Colliders(EntityHashSet);

#[derive(Component)]
#[relationship(relationship_target= Colliders)]
pub(crate) struct ColliderOf(pub(crate) Entity);

#[derive(Component, Debug, PartialEq)]
#[require(Colliders)]
pub struct ColliderDefinition(pub(crate) Collider);

impl From<Collider> for ColliderDefinition {
	fn from(collider: Collider) -> Self {
		Self(collider)
	}
}

#[derive(Component, Debug, PartialEq, Clone, Copy)]
#[require(Transform)]
pub(crate) enum ColliderShape {
	Sphere {
		radius: Units,
		hollow_radius: Option<Units>,
	},
	Cuboid {
		half_x: Units,
		half_y: Units,
		half_z: Units,
	},
	Capsule {
		half_y: Units,
		radius: Units,
	},
	Cylinder {
		half_y: Units,
		radius: Units,
	},
}

impl From<Shape> for ColliderShape {
	fn from(value: Shape) -> Self {
		match value {
			Shape::Sphere { radius } => Self::Sphere {
				radius,
				hollow_radius: None,
			},
			Shape::Cuboid {
				half_x,
				half_y,
				half_z,
			} => Self::Cuboid {
				half_x,
				half_y,
				half_z,
			},
			Shape::Capsule { half_y, radius } => Self::Capsule { half_y, radius },
			Shape::Cylinder { half_y, radius } => Self::Cylinder { half_y, radius },
		}
	}
}

impl SimpleOuterRadius for ColliderShape {
	fn simple_outer_radius(&self) -> Option<Units> {
		match *self {
			Self::Sphere { radius, .. } => Some(radius),
			Self::Capsule { half_y, radius } => Some(Units::from(*half_y + *radius)),
			Self::Cylinder { .. } | Self::Cuboid { .. } => None,
		}
	}
}

impl Prefab<()> for ColliderShape {
	type TError = Unreachable;

	fn insert_prefab_components(
		&self,
		entity: &mut impl PrefabEntityCommands,
		_: &mut impl LoadAsset,
	) -> Result<(), Self::TError> {
		match *self {
			Self::Sphere {
				radius,
				hollow_radius,
			} => {
				entity.try_insert_if_new(RapierCollider::ball(*radius));
				let Some(hollow_radius) = hollow_radius else {
					return Ok(());
				};
				entity.try_insert_if_new(Hollow {
					radius: hollow_radius,
				});
			}
			Self::Cuboid {
				half_x,
				half_y,
				half_z,
			} => {
				entity.try_insert_if_new(RapierCollider::cuboid(*half_x, *half_y, *half_z));
			}
			Self::Cylinder { half_y, radius } => {
				entity.try_insert_if_new(RapierCollider::cylinder(*half_y, *radius));
			}
			Self::Capsule { half_y, radius } => {
				entity.try_insert_if_new(RapierCollider::capsule_y(*half_y, *radius));
			}
		}

		Ok(())
	}
}
