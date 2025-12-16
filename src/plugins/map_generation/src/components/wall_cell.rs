use bevy::prelude::*;
use common::{
	errors::Unreachable,
	tools::Units,
	traits::{
		handles_physics::colliders::{Blocker, Collider, ColliderType, HandlesColliders, Shape},
		load_asset::LoadAsset,
		prefab::{Prefab, PrefabEntityCommands},
	},
};

#[derive(Component, Debug, PartialEq)]
#[require(Transform)]
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
			half_x: Units::from(0.5),
			half_y: Units::from(0.5),
			half_z: Units::from(0.5),
		};
		let collider = Collider::from_shape(shape)
			.with_collider_type(ColliderType::Terrain)
			.with_blocker_types([Blocker::Physical]);

		entity.try_insert_if_new(TPhysics::TCollider::from(collider));

		Ok(())
	}
}
