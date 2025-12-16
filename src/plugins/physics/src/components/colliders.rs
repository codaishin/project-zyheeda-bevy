use bevy::{ecs::entity::EntityHashSet, prelude::*};
use bevy_rapier3d::prelude::Collider as RapierCollider;
use common::{
	errors::Unreachable,
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

#[derive(Component, Debug, PartialEq)]
#[require(Transform)]
pub(crate) struct ColliderShape(pub(crate) Shape);

impl Prefab<()> for ColliderShape {
	type TError = Unreachable;

	fn insert_prefab_components(
		&self,
		entity: &mut impl PrefabEntityCommands,
		_: &mut impl LoadAsset,
	) -> Result<(), Self::TError> {
		match self.0 {
			Shape::Sphere { radius } => {
				entity.try_insert_if_new(RapierCollider::ball(*radius));
			}
			Shape::Cuboid {
				half_x,
				half_y,
				half_z,
			} => {
				entity.try_insert_if_new(RapierCollider::cuboid(*half_x, *half_y, *half_z));
			}
			Shape::Cylinder { half_y, radius } => {
				entity.try_insert_if_new(RapierCollider::cylinder(*half_y, *radius));
			}
			Shape::Capsule { half_y, radius } => {
				entity.try_insert_if_new(RapierCollider::capsule_y(*half_y, *radius));
			}
		}

		Ok(())
	}
}
