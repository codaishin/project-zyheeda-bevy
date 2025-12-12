use bevy::prelude::*;
use common::{
	components::is_blocker::{Blocker, IsBlocker},
	errors::Unreachable,
	traits::{
		handles_physics::colliders::{Collider, ColliderType, HandlesColliders, Shape},
		load_asset::LoadAsset,
		prefab::{Prefab, PrefabEntityCommands},
	},
};

#[derive(Component, Debug, PartialEq)]
#[require(
	Transform,
	IsBlocker = [Blocker::Physical],
)]
pub(crate) struct WallCell;

impl<TPhysics> Prefab<TPhysics> for WallCell
where
	TPhysics: HandlesColliders,
{
	type TError = Unreachable;

	fn insert_prefab_components(
		&self,
		entity: &mut impl PrefabEntityCommands,
		_: &mut impl LoadAsset,
	) -> Result<(), Self::TError> {
		let shape = Shape::Cuboid {
			half_x: 0.5,
			half_y: 0.5,
			half_z: 0.5,
		};
		let collider = Collider::from_shape(shape).with_collider_type(ColliderType::Terrain);

		entity.try_insert_if_new(TPhysics::TCollider::from(collider));

		Ok(())
	}
}
