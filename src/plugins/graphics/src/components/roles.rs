use bevy::{ecs::system::StaticSystemParam, prelude::*};
use common::{
	errors::Unreachable,
	traits::prefab::{Prefab, PrefabEntityCommands},
};

#[derive(Component, Debug, PartialEq, Default)]
#[component(immutable)]
pub(crate) struct RoleAssigned;

#[derive(Component, Debug, PartialEq, Default)]
#[component(immutable)]
#[require(RoleAssigned)]
pub(crate) struct Player;

impl Prefab<()> for Player {
	type TError = Unreachable;
	type TSystemParam<'w, 's> = ();

	fn insert_prefab_components(
		&self,
		entity: &mut impl PrefabEntityCommands,
		_: StaticSystemParam<Self::TSystemParam<'_, '_>>,
	) -> Result<(), Self::TError> {
		entity.try_insert(PointLight {
			intensity: 10_000.,
			range: 40.,
			radius: 1.,
			..default()
		});

		Ok(())
	}
}

#[derive(Component, Debug, PartialEq, Default)]
#[component(immutable)]
#[require(RoleAssigned)]
pub(crate) struct Enemy;
